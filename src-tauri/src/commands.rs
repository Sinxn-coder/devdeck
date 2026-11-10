use crate::storage::{canonical_dir, now, Database, Project};
use serde::Serialize;
use std::{collections::HashSet, fs, process::Command, sync::Mutex};
use tauri::State;
use crate::env::EnvReport;

#[tauri::command]
pub fn list_projects(db: State<'_, Mutex<Database>>) -> Result<Vec<Project>, String> {
    db.lock()
        .map_err(|_| "database lock failed".to_string())?
        .all()
        .map_err(|e| e.to_string())
}

fn redact_summary(s: &str) -> String {
    // very small sanitizer: for each line, if it contains a sensitive key before ':' or '=', redact the value
    let keywords = ["authorization", "token", "password", "secret", "api_key", "api-key", "api key"];
    let mut out = Vec::new();
    for line in s.lines() {
        let mut replaced = line.to_string();
        if let Some(colon_idx) = line.find(':') {
            let (k, v) = line.split_at(colon_idx);
            let klow = k.to_lowercase();
            if keywords.iter().any(|kw| klow.contains(kw)) {
                replaced = format!("{}: [redacted]", k.trim());
            }
        } else if let Some(eq_idx) = line.find('=') {
            let (k, v) = line.split_at(eq_idx);
            let klow = k.to_lowercase();
            if keywords.iter().any(|kw| klow.contains(kw)) {
                replaced = format!("{}=[redacted]", k.trim());
            }
        }
        out.push(replaced);
    }
    let joined = out.join("\n");
    if joined.len() > 240 { joined[..240].to_string() } else { joined }
}

#[tauri::command]
pub fn register_project(
    path: String,
    db: State<'_, Mutex<Database>>,
) -> Result<Project, String> {
    let start = std::time::Instant::now();
    let dir = canonical_dir(&path).map_err(|e| e.to_string())?;
    let files = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<HashSet<_>>();
    let project = Project {
        id: uuid::Uuid::new_v4().to_string(),
        name: dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        path: dir.to_string_lossy().to_string(),
        git: files.contains(".git"),
        technologies: detect(&dir, &files),
        pinned: false,
        registered_at: now(),
        last_accessed_at: now(),
    };
    db.lock()
        .map_err(|_| "database lock failed".to_string())?
        .insert(&project)
        .map_err(|e| e.to_string())?;
    let duration = start.elapsed().as_millis() as i64;
    let summary = format!("Registered project {}", project.name);
    let _ = db.lock().map_err(|_| "db lock".to_string())?.insert_history(&uuid::Uuid::new_v4().to_string(), &now(), "project.register", true, &redact_summary(&summary), Some(duration));
    Ok(project)
}

#[tauri::command]
pub fn toggle_project_pin(
    id: String,
    pinned: bool,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let start = std::time::Instant::now();
    db.lock()
        .map_err(|_| "database lock failed".to_string())?
        .pin(&id, pinned)
        .map_err(|e| e.to_string())?;
    let duration = start.elapsed().as_millis() as i64;
    let summary = format!("{} project pin", if pinned { "Pinned" } else { "Unpinned" });
    let _ = db.lock().map_err(|_| "db lock".to_string())?.insert_history(&uuid::Uuid::new_v4().to_string(), &now(), "project.pin", true, &redact_summary(&summary), Some(duration));
    Ok(())
}

fn detect(dir: &std::path::Path, files: &HashSet<String>) -> Vec<String> {
    let mut result = Vec::new();
    let package_text = fs::read_to_string(dir.join("package.json")).unwrap_or_default();
    let has_dependency = |name: &str| package_text.contains(&format!("\"{name}\""));

    if files.contains("pubspec.yaml") { result.push("Flutter".into()); }
    if files.contains("pyproject.toml")
        || files.contains("requirements.txt")
        || files.iter().any(|file| file.ends_with(".py"))
    { result.push("Python".into()); }
    if files.contains("next.config.js") || files.contains("next.config.mjs") || has_dependency("next") { result.push("Next.js".into()); }
    if files.contains("vite.config.ts") || files.contains("vite.config.js") || has_dependency("vite") { result.push("Vite".into()); }
    if has_dependency("react") { result.push("React".into()); }
    if files.contains("tsconfig.json") || has_dependency("typescript") { result.push("TypeScript".into()); }
    if files.contains("package.json") { result.push("Node.js".into()); }
    if files.contains(".git") { result.push("Git".into()); }
    if result.is_empty() { result.push("Generic project".into()); }
    result
}

#[derive(Serialize)]
pub struct GitInfo {
    pub is_repository: bool,
    pub branch: Option<String>,
    pub clean: bool,
    pub staged: Vec<String>,
    pub modified: Vec<String>,
    pub untracked: Vec<String>,
    pub recent_commits: Vec<String>,
}

#[tauri::command]
pub fn inspect_git(path: String, db: State<'_, Mutex<Database>>) -> Result<GitInfo, String> {
    let start = std::time::Instant::now();
    let dir = canonical_dir(&path).map_err(|e| e.to_string())?;
    let run = |args: &[&str]| Command::new("git").args(args).current_dir(&dir).output();
    let check = run(&["rev-parse", "--is-inside-work-tree"])
        .map_err(|_| {
            let _ = db.lock().map_err(|_| "db lock".to_string())?.insert_history(&uuid::Uuid::new_v4().to_string(), &now(), "git.inspect", false, &redact_summary("Git not installed"), None);
            "Git is not installed or unavailable".to_string()
        })?;
    if !check.status.success() {
        let _ = db.lock().map_err(|_| "db lock".to_string())?.insert_history(&uuid::Uuid::new_v4().to_string(), &now(), "git.inspect", true, &redact_summary("Not a git repository"), Some(start.elapsed().as_millis() as i64));
        return Ok(GitInfo { is_repository: false, branch: None, clean: true, staged: vec![], modified: vec![], untracked: vec![], recent_commits: vec![] });
    }
    let branch = run(&["branch", "--show-current"])
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string());
    let status = run(&["status", "--porcelain=v1"]).map_err(|e| e.to_string())?;
    let mut staged = Vec::new();
    let mut modified = Vec::new();
    let mut untracked = Vec::new();
    for line in String::from_utf8_lossy(&status.stdout).lines() {
        if line.len() < 3 { continue; }
        let path = line[3..].to_string();
        let state = &line[0..2];
        if state == "??" { untracked.push(path); continue; }
        if state.chars().next().unwrap_or(' ') != ' ' { staged.push(path.clone()); }
        if state.chars().nth(1).unwrap_or(' ') != ' ' { modified.push(path); }
    }
    let recent_commits = run(&["log", "-5", "--pretty=format:%h %s"])
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).lines().map(str::to_string).collect())
        .unwrap_or_default();
    let ok = true;
    let duration = start.elapsed().as_millis() as i64;
    let summary = if branch.is_some() { format!("{} · {} staged · {} modified · {} untracked", branch.clone().unwrap_or_else(||"detached".into()), staged.len(), modified.len(), untracked.len()) } else { format!("{} staged · {} modified · {} untracked", staged.len(), modified.len(), untracked.len()) };
    let _ = db.lock().map_err(|_| "db lock".to_string())?.insert_history(&uuid::Uuid::new_v4().to_string(), &now(), "git.inspect", ok, &redact_summary(&summary), Some(duration));
    Ok(GitInfo { is_repository: true, clean: staged.is_empty() && modified.is_empty() && untracked.is_empty(), branch, staged, modified, untracked, recent_commits })
}

#[derive(Serialize, Debug, PartialEq)]
pub struct PortInfo { pub port: u16, pub protocol: String, pub process_id: Option<u32>, pub process_name: Option<String>, pub process_attribution_supported: bool }

fn parse_linux_tcp(text: &str) -> Vec<PortInfo> {
    text.lines().skip(1).filter_map(|line| {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.get(3) != Some(&"0A") { return None; }
        let port = u16::from_str_radix(fields.get(1)?.split(':').nth(1)?, 16).ok()?;
        Some(PortInfo { port, protocol: "tcp".into(), process_id: None, process_name: None, process_attribution_supported: false })
    }).collect()
}

#[cfg(target_os = "windows")]
fn discover_windows_tcp() -> Result<Vec<PortInfo>, String> {
    let output = Command::new("netstat").args(["-ano", "-p", "tcp"]).output().map_err(|_| "Windows netstat is unavailable".to_string())?;
    if !output.status.success() { return Err("Windows netstat failed".into()); }
    Ok(String::from_utf8_lossy(&output.stdout).lines().filter_map(|line| {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 5 || fields[0] != "TCP" || fields[3] != "LISTENING" { return None; }
        let port = fields[1].rsplit_once(':').and_then(|(_, value)| value.parse().ok())?;
        let process_id = fields[4].parse().ok();
        Some(PortInfo { port, protocol: "tcp".into(), process_id, process_name: None, process_attribution_supported: process_id.is_some() })
    }).collect())
}

#[tauri::command]
pub fn discover_ports() -> Result<Vec<PortInfo>, String> {
    #[cfg(target_os = "linux")] { return Ok(parse_linux_tcp(&fs::read_to_string("/proc/net/tcp").map_err(|e| e.to_string())?)); }
    #[cfg(target_os = "windows")] { return discover_windows_tcp(); }
    #[cfg(target_os = "macos")] { return Err("Port discovery is not implemented on macOS yet".into()); }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))] { Err("Port discovery is unsupported on this platform".into()) }
}

#[tauri::command]
pub fn inspect_environment(path: String) -> Result<EnvReport, String> {
    let dir = canonical_dir(&path).map_err(|e| e.to_string())?;
    crate::env::inspect_project(&dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_multiple_project_types() {
        let dir = std::env::temp_dir().join(format!("devdeck-detect-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("package.json"), r#"{"dependencies":{"react":"1","vite":"1","typescript":"1"}}"#).unwrap();
        let files = ["package.json", "vite.config.ts", "tsconfig.json", ".git"].into_iter().map(str::to_string).collect();
        let result = detect(&dir, &files);
        assert!(result.contains(&"React".into()) && result.contains(&"Vite".into()) && result.contains(&"TypeScript".into()) && result.contains(&"Git".into()));
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn parses_only_listening_tcp() { let result = parse_linux_tcp("header\n0: 0100007F:1F90 00000000:0000 0A\n1: 0100007F:0016 00000000:0000 01"); assert_eq!(result.len(), 1); assert_eq!(result[0].port, 8080); }
    #[test]
    fn empty_table_is_empty() { assert!(parse_linux_tcp("header\n").is_empty()); }
    #[test]
    fn git_error_is_actionable_for_missing_directory() { assert!(inspect_git("/definitely/not/a/project".into()).is_err()); }
}
