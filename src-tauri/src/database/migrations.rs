use rusqlite::Connection;

use crate::error::AppError;

pub fn apply_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS template (
            id TEXT PRIMARY KEY,
            file_name TEXT NOT NULL,
            sheet_name TEXT NOT NULL,
            header_start_row INTEGER NOT NULL,
            header_end_row INTEGER NOT NULL,
            data_start_row INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS template_column (
            id TEXT PRIMARY KEY,
            template_id TEXT NOT NULL,
            column_index INTEGER NOT NULL,
            header TEXT NOT NULL,
            normalized_header TEXT NOT NULL,
            embedding TEXT,
            embedding_model_version TEXT,
            FOREIGN KEY(template_id) REFERENCES template(id)
        );

        CREATE TABLE IF NOT EXISTS header_mapping_history (
            id TEXT PRIMARY KEY,
            source_header TEXT NOT NULL,
            normalized_source_header TEXT NOT NULL,
            target_header TEXT NOT NULL,
            normalized_target_header TEXT NOT NULL,
            usage_count INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        ",
    )?;
    Ok(())
}
