use std::fs;
use std::path::{Component, Path, PathBuf};

fn safe_path(root: &Path, file_path: &str) -> Result<PathBuf, String> {
    let root_canonical = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let candidate = if Path::new(file_path).is_absolute() {
        PathBuf::from(file_path)
    } else {
        root_canonical.join(file_path)
    };
    let absolute = match fs::canonicalize(&candidate) {
        Ok(real) => real,
        Err(_) => {
            // File may not exist yet; canonicalize the parent to resolve the prefix.
            match candidate.parent().and_then(|p| fs::canonicalize(p).ok()) {
                Some(parent) => parent.join(
                    candidate
                        .file_name()
                        .ok_or_else(|| format!("Invalid path: {file_path}"))?,
                ),
                None => candidate,
            }
        }
    };
    let from_root = absolute
        .strip_prefix(&root_canonical)
        .map_err(|_| format!("Refusing path outside project: {file_path}"))?;
    if from_root
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(format!("Refusing path outside project: {file_path}"));
    }
    let from_root_str = from_root.to_string_lossy().replace('\\', "/");
    if from_root_str == ".env"
        || from_root_str.starts_with(".env.")
        || from_root_str.starts_with(".git")
    {
        return Err(format!("Refusing sensitive path: {file_path}"));
    }
    Ok(absolute)
}

pub fn read_project_file(root: &Path, file_path: &str) -> Result<String, String> {
    let absolute = safe_path(root, file_path)?;
    fs::read_to_string(&absolute)
        .map_err(|e| format!("Failed to read {}: {}", absolute.display(), e))
}

pub fn write_project_file(
    root: &Path,
    file_path: &str,
    content: &str,
    expected_content: Option<&str>,
) -> Result<PathBuf, String> {
    let absolute = safe_path(root, file_path)?;
    if let Some(expected) = expected_content {
        let current = fs::read_to_string(&absolute).ok();
        if current.as_deref() != Some(expected) {
            return Err(format!(
                "Edit precondition failed for {file_path}; file changed or does not exist."
            ));
        }
    }
    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {e}"))?;
    }
    fs::write(&absolute, content)
        .map_err(|e| format!("Failed to write {}: {}", absolute.display(), e))?;
    Ok(absolute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_paths_outside_project() {
        let root = Path::new("/tmp/project");
        assert!(safe_path(root, "../outside.txt").is_err());
        assert!(safe_path(root, "/etc/passwd").is_err());
        assert!(safe_path(root, "sub/dir/file.txt").is_ok());
    }

    #[test]
    fn refuses_sensitive_paths() {
        let root = Path::new("/tmp/project");
        assert!(safe_path(root, ".env").is_err());
        assert!(safe_path(root, ".env.local").is_err());
        assert!(safe_path(root, ".git/config").is_err());
    }

    #[test]
    fn write_checks_expected_content() {
        let dir = std::env::temp_dir().join(format!("ind-files-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("file.txt");
        fs::write(&path, "old").unwrap();

        assert!(write_project_file(&dir, "file.txt", "new", Some("old")).is_ok());
        assert_eq!(fs::read_to_string(&path).unwrap(), "new");

        let err = write_project_file(&dir, "file.txt", "x", Some("old")).unwrap_err();
        assert!(err.contains("precondition failed"));

        fs::remove_dir_all(&dir).unwrap();
    }
}
