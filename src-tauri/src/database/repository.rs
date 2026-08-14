use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::database::migrations::apply_migrations;
use crate::domain::{HeaderMapping, TemplateSchema};
use crate::error::AppError;
use crate::mapping::history::{HistoryHit, HistoryLookup};
use crate::mapping::normalize_header;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;
        apply_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    #[allow(dead_code)]
    pub fn memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()?;
        apply_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|_| AppError::DatabaseError("lock poisoned".into()))
    }

    pub fn save_template(
        &self,
        template: &TemplateSchema,
        embeddings: &[Vec<f32>],
        model_version: &str,
    ) -> Result<(), AppError> {
        let conn = self.lock()?;
        let now = now_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO template
            (id, file_name, sheet_name, header_start_row, header_end_row, data_start_row, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            rusqlite::params![
                template.id,
                template.file_name,
                template.sheet_name,
                template.header_start_row as i64,
                template.header_end_row as i64,
                template.data_start_row as i64,
                now,
            ],
        )?;
        conn.execute(
            "DELETE FROM template_column WHERE template_id = ?1",
            rusqlite::params![template.id],
        )?;
        for (column, embedding) in template.columns.iter().zip(embeddings.iter()) {
            let embedding_json = serde_json::to_string(embedding)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            conn.execute(
                "INSERT INTO template_column
                (id, template_id, column_index, header, normalized_header, embedding, embedding_model_version)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    template.id,
                    column.index as i64,
                    column.name,
                    column.normalized_name,
                    embedding_json,
                    model_version,
                ],
            )?;
        }
        Ok(())
    }

    pub fn load_template_embeddings(
        &self,
        template_id: &str,
        model_version: &str,
    ) -> Result<Option<Vec<Vec<f32>>>, AppError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT embedding, embedding_model_version, column_index
             FROM template_column
             WHERE template_id = ?1
             ORDER BY column_index ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![template_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        let mut embeddings = Vec::new();
        for row in rows {
            let (json, version, _) = row?;
            if version != model_version {
                return Ok(None);
            }
            let vector: Vec<f32> = serde_json::from_str(&json)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            embeddings.push(vector);
        }
        if embeddings.is_empty() {
            Ok(None)
        } else {
            Ok(Some(embeddings))
        }
    }

    pub fn record_confirmed_mappings(&self, mappings: &[HeaderMapping]) -> Result<(), AppError> {
        let conn = self.lock()?;
        let now = now_rfc3339();
        for mapping in mappings {
            let Some(target_header) = mapping.target_header.as_ref() else {
                continue;
            };
            let normalized_target = normalize_header(target_header);
            let existing: Option<(String, i64)> = conn
                .query_row(
                    "SELECT id, usage_count FROM header_mapping_history
                     WHERE normalized_source_header = ?1 AND normalized_target_header = ?2",
                    rusqlite::params![mapping.normalized_source_header, normalized_target],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;

            if let Some((id, usage_count)) = existing {
                conn.execute(
                    "UPDATE header_mapping_history
                     SET usage_count = ?1, updated_at = ?2, source_header = ?3, target_header = ?4
                     WHERE id = ?5",
                    rusqlite::params![
                        usage_count + 1,
                        now,
                        mapping.source_header,
                        target_header,
                        id
                    ],
                )?;
            } else {
                conn.execute(
                    "INSERT INTO header_mapping_history
                    (id, source_header, normalized_source_header, target_header, normalized_target_header, usage_count, created_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)",
                    rusqlite::params![
                        Uuid::new_v4().to_string(),
                        mapping.source_header,
                        mapping.normalized_source_header,
                        target_header,
                        normalized_target,
                        now,
                    ],
                )?;
            }
        }
        Ok(())
    }
}

impl HistoryLookup for Database {
    fn find(&self, normalized_source: &str) -> Option<HistoryHit> {
        let conn = self.lock().ok()?;
        conn.query_row(
            "SELECT target_header, normalized_target_header, usage_count
             FROM header_mapping_history
             WHERE normalized_source_header = ?1
             ORDER BY usage_count DESC, updated_at DESC
             LIMIT 1",
            rusqlite::params![normalized_source],
            |row| {
                Ok(HistoryHit {
                    target_header: row.get(0)?,
                    normalized_target_header: row.get(1)?,
                    usage_count: row.get(2)?,
                })
            },
        )
        .ok()
    }
}

fn now_rfc3339() -> String {
    chrono::Local::now().to_rfc3339()
}


#[cfg(test)]
mod tests {
    use super::Database;
    use crate::domain::{HeaderMapping, MappingSource};
    use crate::mapping::history::HistoryLookup;
    use crate::mapping::normalize_header;

    #[test]
    fn history_prefers_highest_usage() {
        let db = Database::memory().unwrap();
        let mapping = |source: &str, target: &str| HeaderMapping {
            source_column_index: 0,
            source_header: source.into(),
            normalized_source_header: normalize_header(source),
            target_column_index: Some(0),
            target_header: Some(target.into()),
            score: Some(1.0),
            source: MappingSource::Manual,
            candidates: vec![],
        };

        db.record_confirmed_mappings(&[mapping("组织机构", "单位名称")])
            .unwrap();
        db.record_confirmed_mappings(&[mapping("组织机构", "所属单位")])
            .unwrap();
        db.record_confirmed_mappings(&[mapping("组织机构", "所属单位")])
            .unwrap();

        let hit = db.find(&normalize_header("组织机构")).unwrap();
        assert_eq!(hit.target_header, "所属单位");
        assert_eq!(hit.usage_count, 2);
    }
}

