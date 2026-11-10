#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod commands_history;
mod storage;
mod env;

use std::sync::Mutex;
use tauri::Manager;
use storage::Database;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db = Database::open(dir.join("devdeck.sqlite3"))?;
            app.manage(Mutex::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_projects,
            commands::register_project,
            commands::toggle_project_pin,
            commands::inspect_git,
            commands::discover_ports,
            commands::inspect_environment,
            commands_history::record_history,
            commands_history::read_history,
            commands_history::clear_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevDeck");
}
