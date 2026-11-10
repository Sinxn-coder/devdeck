use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn record_history(operation: String, ok: bool, summary: String, duration_ms: Option<i64>, db: State<'_, Mutex<Database>>) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    let at = crate::storage::now();
    db.lock().map_err(|_| "db lock".to_string())?.insert_history(&id, &at, &operation, ok, &summary, duration_ms).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn read_history(db: State<'_, Mutex<Database>>) -> Result<Vec<(String,String,String,bool,String,Option<i64>)>, String> {
    db.lock().map_err(|_| "db lock".to_string())?.all_history().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_history(db: State<'_, Mutex<Database>>) -> Result<(), String> {
    db.lock().map_err(|_| "db lock".to_string())?.clear_history().map_err(|e| e.to_string())
}
