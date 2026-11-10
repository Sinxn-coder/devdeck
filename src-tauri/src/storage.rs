use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError { #[error("database error: {0}")] Sql(#[from] rusqlite::Error), #[error("project path is not an accessible directory")] InvalidPath, #[error("project is already registered")] Duplicate }
pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Serialize, Clone)]
pub struct Project { pub id:String, pub name:String, pub path:String, #[serde(rename="tech")] pub technologies:Vec<String>, pub git:bool, pub pinned:bool, #[serde(rename="registeredAt")] pub registered_at:String, #[serde(rename="updatedAt")] pub last_accessed_at:String }

pub struct Database { conn: Connection }
impl Database {
 pub fn open(path:PathBuf)->Result<Self>{let c=Connection::open(path)?;c.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL); INSERT INTO schema_version(version) SELECT 0 WHERE NOT EXISTS(SELECT 1 FROM schema_version); CREATE TABLE IF NOT EXISTS projects (id TEXT PRIMARY KEY,name TEXT NOT NULL,path TEXT NOT NULL UNIQUE,technologies TEXT NOT NULL,git INTEGER NOT NULL DEFAULT 0,pinned INTEGER NOT NULL DEFAULT 0,registered_at TEXT NOT NULL,last_accessed_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS history (id TEXT PRIMARY KEY, at TEXT NOT NULL, operation TEXT NOT NULL, ok INTEGER NOT NULL, summary TEXT NOT NULL, duration_ms INTEGER); ")?;Ok(Self{conn:c})}
 pub fn all(&self)->Result<Vec<Project>>{let mut s=self.conn.prepare("SELECT id,name,path,technologies,git,pinned,registered_at,last_accessed_at FROM projects ORDER BY pinned DESC,last_accessed_at DESC")?;let rows=s.query_map([],|r|{let t:String=r.get(3)?;Ok(Project{id:r.get(0)?,name:r.get(1)?,path:r.get(2)?,technologies:serde_json::from_str(&t).unwrap_or_default(),git:r.get::<_,i64>(4)?!=0,pinned:r.get::<_,i64>(5)?!=0,registered_at:r.get(6)?,last_accessed_at:r.get(7)?})})?;rows.collect::<std::result::Result<Vec<_>,_>>().map_err(StorageError::Sql)}
 pub fn insert(&self,p:&Project)->Result<()> {let tech=serde_json::to_string(&p.technologies).unwrap_or_else(|_|"[]".into());self.conn.execute("INSERT INTO projects (id,name,path,technologies,git,pinned,registered_at,last_accessed_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",params![p.id,p.name,p.path,tech,p.git as i64,p.pinned as i64,p.registered_at,p.last_accessed_at]).map_err(|e|if matches!(e,rusqlite::Error::SqliteFailure(ref x,_) if x.extended_code==2067){StorageError::Duplicate}else{StorageError::Sql(e)})?;Ok(())}
 pub fn pin(&self,id:&str,pinned:bool)->Result<()>{self.conn.execute("UPDATE projects SET pinned=?1 WHERE id=?2",params![pinned as i64,id])?;Ok(())}
 pub fn insert_history(&self, id:&str, at:&str, operation:&str, ok:bool, summary:&str, duration_ms:Option<i64>) -> Result<()> {
    self.conn.execute("INSERT INTO history (id, at, operation, ok, summary, duration_ms) VALUES (?1,?2,?3,?4,?5,?6)", params![id, at, operation, ok as i64, summary, duration_ms])?;
    // enforce retention of last 200 entries
    self.conn.execute("DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY at DESC LIMIT 200)", params![])?;
    Ok(())
 }
 pub fn all_history(&self)->Result<Vec<(String,String,String,bool,String,Option<i64>)>>{
    let mut s = self.conn.prepare("SELECT id,at,operation,ok,summary,duration_ms FROM history ORDER BY at DESC LIMIT 200")?;
    let rows = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_,i64>(3)?!=0, r.get(4)?, r.get(5)?)))?;
    rows.collect::<std::result::Result<Vec<_>,_>>().map_err(StorageError::Sql)
 }
 pub fn clear_history(&self)->Result<()> { self.conn.execute("DELETE FROM history", params![])?; Ok(()) }
}

pub fn canonical_dir(input:&str)->Result<PathBuf>{let p=Path::new(input);if !p.is_dir(){return Err(StorageError::InvalidPath)};p.canonicalize().map_err(|_|StorageError::InvalidPath)}
pub fn now()->String{Utc::now().to_rfc3339()}

#[cfg(test)]
mod tests {
 use super::*;
 fn db()->Database { let p=std::env::temp_dir().join(format!("devdeck-test-{}.sqlite",uuid::Uuid::new_v4())); Database::open(p).unwrap() }
 fn project(path:&str)->Project { Project{id:"one".into(),name:"demo".into(),path:path.into(),technologies:vec!["Rust".into()],git:false,pinned:false,registered_at:now(),last_accessed_at:now()} }
 #[test] fn crud_and_pin(){let d=db();d.insert(&project("/tmp/demo")).unwrap();assert_eq!(d.all().unwrap().len(),1);d.pin("one",true).unwrap();assert!(d.all().unwrap()[0].pinned)}
 #[test] fn duplicate_path_is_rejected(){let d=db();d.insert(&project("/tmp/demo")).unwrap();let mut p=project("/tmp/demo");p.id="two".into();assert!(matches!(d.insert(&p),Err(StorageError::Duplicate)))}
 #[test] fn invalid_path_is_rejected(){assert!(matches!(canonical_dir("/definitely/not/a/project"),Err(StorageError::InvalidPath)))}
}
