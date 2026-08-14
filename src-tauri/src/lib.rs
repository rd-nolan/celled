mod app_state;
mod commands;
mod database;
mod domain;
mod embedding;
mod error;
mod excel;
mod mapping;

use std::path::PathBuf;

use tauri::Manager;

use crate::app_state::AppState;
use crate::database::Database;
use crate::embedding::create_provider;
use crate::mapping::AliasDictionary;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
            let db = Database::open(&app_dir.join("celld.db")).map_err(|e| e.to_string())?;

            let resource_dir = app.path().resource_dir().ok();
            let model_dir = resolve_resource(&resource_dir, "models/header-embedding");
            let alias_path = resolve_resource(&resource_dir, "aliases.json");
            let provider = create_provider(model_dir.as_deref());
            let alias = AliasDictionary::load_or_builtin(alias_path.as_deref());

            app.manage(AppState::new(provider, db, alias));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::analyze_template,
            commands::update_template_header_row,
            commands::confirm_template,
            commands::analyze_data_excel,
            commands::update_import_header_row,
            commands::update_import_sheet,
            commands::update_mapping,
            commands::confirm_mapping,
            commands::convert_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn resolve_resource(resource_dir: &Option<PathBuf>, relative: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(dir) = resource_dir {
        candidates.push(dir.join(relative));
        candidates.push(dir.join("resources").join(relative));
    }
    candidates.push(PathBuf::from("src-tauri/resources").join(relative));
    candidates.push(PathBuf::from("resources").join(relative));
    candidates.into_iter().find(|path| path.exists())
}
