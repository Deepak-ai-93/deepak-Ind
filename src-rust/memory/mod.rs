use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const MEMORY_TEMPLATE: &str = "# Memory — IND\n\n## Project\n\n- **Goal:** Project-local memory for IND sessions.\n- **Where we are:** New project.\n\n## Standing decisions\n\n- Keep memory human-readable and append-only.\n";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeState {
    pub task: String,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub current_chunk: usize,
    pub status: String,
    pub next_step: String,
    pub updated_at: String,
}

impl std::fmt::Display for ResumeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "resume: {} — chunk {} — {} — next: {}",
            self.task, self.current_chunk, self.status, self.next_step
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryEntry {
    id: String,
    category: String,
    content: String,
    created_at: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct MemoryDb {
    entries: Vec<MemoryEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resume: Option<ResumeState>,
}

pub struct MemoryManager {
    project_root: PathBuf,
    file_path: PathBuf,
    db_path: PathBuf,
    db: MemoryDb,
}

impl MemoryManager {
    pub fn new(project_root: &Path) -> Self {
        let ind_dir = project_root.join(".ind");
        let _ = fs::create_dir_all(&ind_dir);
        let db_path = ind_dir.join("memory.json");
        let db = read_db(&db_path);
        Self {
            project_root: project_root.to_path_buf(),
            file_path: project_root.join("MEMORY.md"),
            db_path,
            db,
        }
    }

    pub fn read(&self) -> Result<String, std::io::Error> {
        self.ensure_file()?;
        if self.file_path.exists() {
            fs::read_to_string(&self.file_path)
        } else {
            Ok(String::new())
        }
    }

    fn ensure_file(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.project_root)?;
        if !self.file_path.exists() {
            fs::write(&self.file_path, MEMORY_TEMPLATE)?;
        }
        Ok(())
    }

    pub fn append(&mut self, category: &str, note: &str) -> Result<(), std::io::Error> {
        let entry = format!("\n- **{}:** {}\n", category.trim(), note);
        self.append_raw(&entry)?;
        let id = random_id();
        let entry = MemoryEntry {
            id,
            category: category.to_string(),
            content: note.to_string(),
            created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        };
        self.db.entries.push(entry);
        self.save_db();
        Ok(())
    }

    pub fn append_daily(
        &mut self,
        did: &[String],
        decided: &[String],
        blocked: &[String],
        next: &str,
    ) -> Result<(), std::io::Error> {
        self.ensure_file()?;
        let date = Utc::now().format("%Y-%m-%d");
        let mut lines = vec![
            format!("\n---\n\n## {date}\n"),
            "- **Did:**".to_string(),
        ];
        for item in did {
            lines.push(format!("  - {item}"));
        }
        lines.push("- **Decided:**".to_string());
        for item in decided {
            lines.push(format!("  - {item}"));
        }
        lines.push("- **Blocked:**".to_string());
        for item in blocked {
            lines.push(format!("  - {item}"));
        }
        lines.push(format!("- **Next:** {next}"));
        lines.push(String::new());
        self.append_raw(&lines.join("\n"))?;
        for content in did.iter().chain(decided).chain(blocked).chain(std::iter::once(&next.to_string())) {
            self.db.entries.push(MemoryEntry {
                id: random_id(),
                category: "session".to_string(),
                content: content.clone(),
                created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            });
        }
        self.save_db();
        Ok(())
    }

    fn append_raw(&self, entry: &str) -> Result<(), std::io::Error> {
        self.ensure_file()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
        file.write_all(entry.as_bytes())?;
        Ok(())
    }

    pub fn relevant(&self, task: &str, limit: usize) -> Result<Vec<String>, std::io::Error> {
        let content = self.read()?;
        let task_terms = task_terms(task);
        let mut hits: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| line.starts_with('-') || line.starts_with("## "))
            .filter(|line| {
                task_terms.is_empty()
                    || task_terms
                        .iter()
                        .any(|term| line.to_lowercase().contains(term))
            })
            .collect();
        let start = hits.len().saturating_sub(limit);
        hits.drain(..start);
        Ok(hits)
    }

    pub fn save_resume(&mut self, state: ResumeState) {
        self.db.resume = Some(state);
        self.save_db();
    }

    pub fn resume_state(&self) -> Result<Option<ResumeState>, std::io::Error> {
        Ok(self.db.resume.clone())
    }

    fn save_db(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.db) {
            let _ = fs::write(&self.db_path, json);
        }
    }
}

fn read_db(path: &Path) -> MemoryDb {
    if !path.exists() {
        return MemoryDb::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn random_id() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

fn task_terms(task: &str) -> Vec<String> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for chunk in task.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        for term in chunk.split(['/', '_', '-']) {
            if term.len() > 2 && seen.insert(term.to_string()) {
                result.push(term.to_string());
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ind-memory-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn memory_read_creates_template() {
        let root = temp_root();
        let mut mem = MemoryManager::new(&root);
        let content = mem.read().unwrap();
        assert!(content.contains("# Memory"));
        mem.append("decision", "Use Rust for the CLI").unwrap();
        assert!(mem.read().unwrap().contains("Use Rust for the CLI"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn saves_and_loads_resume_state() {
        let root = temp_root();
        let mut mem = MemoryManager::new(&root);
        assert!(mem.resume_state().unwrap().is_none());
        mem.save_resume(ResumeState {
            task: "port to rust".to_string(),
            session_id: None,
            current_chunk: 2,
            status: "active".to_string(),
            next_step: "verify the change".to_string(),
            updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        });
        let loaded = MemoryManager::new(&root);
        let state = loaded.resume_state().unwrap().unwrap();
        assert_eq!(state.task, "port to rust");
        assert_eq!(state.status, "active");
        fs::remove_dir_all(&root).unwrap();
    }
}