use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{SecondsFormat, Utc};

const IGNORED_DIRECTORIES: &[&str] = &[
    ".git", ".hg", ".svn", "node_modules", "dist", "build", "coverage", ".ind", ".agents",
    "output", ".next", "target", "vendor",
];

const SOURCE_EXTENSIONS: &[&str] = &[
    ".c", ".cc", ".cpp", ".css", ".go", ".h", ".hpp", ".html", ".java", ".js", ".json",
    ".jsx", ".md", ".php", ".py", ".rb", ".rs", ".sh", ".sql", ".svelte", ".toml", ".ts",
    ".tsx", ".vue", ".yaml", ".yml",
];

#[derive(Debug, Clone)]
pub struct RepositoryFile {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub extension: String,
    pub bytes: u64,
    pub modified_at_ms: u128,
    pub is_source: bool,
}

#[derive(Debug, Clone)]
pub struct RepositorySnapshot {
    pub root: PathBuf,
    pub files: Vec<RepositoryFile>,
    pub ignored_directories: Vec<String>,
    pub scanned_at: String,
}

fn extension_of(name: &str) -> String {
    match name.rfind('.') {
        Some(idx) if idx + 1 < name.len() => name[idx..].to_lowercase(),
        _ => String::new(),
    }
}

fn walk(root: &Path, current: &Path, files: &mut Vec<RepositoryFile>, ignored: &mut HashSet<String>) -> Result<(), std::io::Error> {
    let entries = fs::read_dir(current)?;
    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let name = entry.file_name().to_string_lossy().to_string();
        let absolute = entry.path();
        let relative = absolute
            .strip_prefix(root)
            .unwrap_or(&absolute)
            .to_string_lossy()
            .replace('\\', "/");

        if file_type.is_dir() {
            if IGNORED_DIRECTORIES.contains(&name.as_str()) {
                ignored.insert(relative);
                continue;
            }
            walk(root, &absolute, files, ignored)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        if name.starts_with('.') && name != ".env.example" {
            continue;
        }

        let metadata = entry.metadata()?;
        let modified_at_ms = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let extension = extension_of(&name);
        let is_source = SOURCE_EXTENSIONS.contains(&extension.as_str());

        files.push(RepositoryFile {
            relative_path: relative,
            absolute_path: absolute,
            extension,
            bytes: metadata.len(),
            modified_at_ms,
            is_source,
        });
    }
    Ok(())
}

pub fn inspect_repository(root: &Path) -> Result<RepositorySnapshot, std::io::Error> {
    let mut files = Vec::new();
    let mut ignored = HashSet::new();
    walk(root, root, &mut files, &mut ignored)?;
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    let mut ignored_directories: Vec<String> = ignored.into_iter().collect();
    ignored_directories.sort();
    Ok(RepositorySnapshot {
        root: root.to_path_buf(),
        files,
        ignored_directories,
        scanned_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
    })
}

pub fn read_repository_file(file: &RepositoryFile) -> Option<String> {
    if !file.is_source || file.bytes > 200_000 {
        return None;
    }
    let content = fs::read_to_string(&file.absolute_path).ok()?;
    if content.contains('\0') {
        return None;
    }
    Some(content)
}
