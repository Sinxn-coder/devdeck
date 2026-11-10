use serde::Serialize;
use std::{collections::HashSet, fs, path::Path};
const MAX_ENV_FILE_BYTES: u64 = 1_048_576;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct EnvKey { pub key: String, pub present: bool, pub empty: bool, pub line: usize, pub error: Option<String> }

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct EnvReport { pub files: Vec<String>, pub expected: Vec<String>, pub entries: Vec<EnvKey>, pub missing: Vec<String>, pub duplicate_keys: Vec<String> }

fn valid_key(key: &str) -> bool { !key.is_empty() && key.chars().enumerate().all(|(i,c)| c == '_' || c.is_ascii_alphanumeric() && (i > 0 || c.is_ascii_alphabetic())) }

pub fn inspect_project(root: &Path) -> Result<EnvReport, String> {
    let candidates = [".env", ".env.local", ".env.development", ".env.example", ".env.sample", ".env.template"];
    let mut files = Vec::new(); let mut expected = Vec::new(); let mut values = Vec::new(); let mut duplicates = Vec::new();
    for name in candidates { let path=root.join(name); let metadata=match fs::symlink_metadata(&path){Ok(m)=>m,Err(_) => continue}; if !metadata.is_file(){continue} if metadata.len()>MAX_ENV_FILE_BYTES{return Err(format!("{name} exceeds the 1 MiB inspection limit"))} files.push(name.to_string()); let text=fs::read_to_string(&path).map_err(|_| format!("Unable to read {name}; file is missing, unreadable, or not UTF-8"))?; let is_template=name.ends_with("example")||name.ends_with("sample")||name.ends_with("template"); let mut seen=HashSet::new();
        for (idx, raw) in text.lines().enumerate() { let line=raw.trim(); if line.is_empty()||line.starts_with('#'){continue} let Some((key, value))=line.split_once('=') else { values.push(EnvKey{key:"<malformed line>".into(),present:false,empty:false,line:idx+1,error:Some("Expected KEY=VALUE".into())}); continue }; let key=key.trim().strip_prefix("export ").unwrap_or(key.trim()).to_string(); let valid=valid_key(&key); if !seen.insert(key.clone()) {duplicates.push(key.clone())} if is_template {expected.push(key.clone())} else {values.push(EnvKey{key,present:true,empty:value.trim().is_empty(),line:idx+1,error:(!valid).then_some("Invalid environment variable name".into())})} }
    }
    expected.sort(); expected.dedup(); let present:HashSet<_>=values.iter().map(|v|v.key.clone()).collect(); let missing=expected.iter().filter(|k|!present.contains(*k)).cloned().collect(); duplicates.sort(); duplicates.dedup(); Ok(EnvReport{files,expected,entries:values,missing,duplicate_keys:duplicates})
}

#[cfg(test)] mod tests { use super::*; use std::{fs, path::PathBuf}; #[test] fn reports_keys_without_values(){let root=std::env::temp_dir().join(format!("devdeck-env-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&root).unwrap();fs::write(root.join(".env.example"),"API_KEY=\nPORT=3000\n").unwrap();fs::write(root.join(".env"),"API_KEY=secret\nPORT=\nPORT=1\n").unwrap();let r=inspect_project(&PathBuf::from(&root)).unwrap();assert_eq!(r.expected,vec!["API_KEY","PORT"]);assert!(r.entries.iter().all(|x|x.key!="secret"));assert!(r.duplicate_keys.contains(&"PORT".into()));fs::remove_dir_all(root).unwrap()} #[test] fn rejects_bad_lines(){let root=std::env::temp_dir().join(format!("devdeck-env-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&root).unwrap();fs::write(root.join(".env"),"bad line\n").unwrap();assert!(inspect_project(&root).unwrap().entries[0].error.is_some());fs::remove_dir_all(root).unwrap()}}
