use std::collections::HashSet;

use crate::context::repository::{inspect_repository, read_repository_file, RepositoryFile, RepositorySnapshot};

const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "in", "is", "it", "of",
    "on", "or", "that", "the", "to", "with",
];

#[derive(Debug, Clone)]
pub struct SelectedContextFile {
    pub relative_path: String,
    pub content: String,
    pub estimated_tokens: usize,
    pub score: i64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct OmittedFile {
    pub relative_path: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ContextSelection {
    pub task: String,
    pub budget_tokens: usize,
    pub estimated_tokens: usize,
    pub selected: Vec<SelectedContextFile>,
    pub omitted: Vec<OmittedFile>,
}

pub fn task_terms(task: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for chunk in task.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        for term in chunk.split(['/', '_', '-']) {
            if term.len() > 2 && !STOP_WORDS.contains(&term) && seen.insert(term.to_string()) {
                result.push(term.to_string());
            }
        }
    }
    result
}

pub fn estimate_tokens(content: &str) -> usize {
    content.chars().count().div_ceil(4).max(1)
}

fn score_file(file: &RepositoryFile, task_terms: &[String]) -> (i64, String) {
    let path = file.relative_path.to_lowercase();
    let basename = path.rsplit('/').next().unwrap_or(&path).to_string();
    let documentation = file.extension == ".md" || file.extension == ".txt";
    let mut score = if file.is_source && !documentation { 2 } else { 0 };
    let matches: Vec<&String> = task_terms.iter().filter(|term| path.contains(*term)).collect();
    score += (matches.len() * 12) as i64;
    if basename.contains("test") || basename.contains("spec") {
        score += 1;
    }
    if path == "package.json" || path == "tsconfig.json" || path.ends_with("/README.md") {
        score += 5;
    }
    if path == "package-lock.json"
        || path.ends_with("/package-lock.json")
        || path == "pack-plan.json"
    {
        score -= 20;
    }
    if file.bytes > 100_000 {
        score -= 8;
    }
    let reason = if matches.is_empty() {
        if file.is_source {
            "source/config file".to_string()
        } else {
            "non-source metadata".to_string()
        }
    } else {
        format!(
            "path matches: {}",
            matches
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    (score, reason)
}

pub fn select_context(snapshot: &RepositorySnapshot, task: &str, budget_tokens: usize) -> Result<ContextSelection, String> {
    if budget_tokens == 0 {
        return Err("Context budget must be a positive integer.".to_string());
    }
    let terms = task_terms(task);
    let mut ranked: Vec<(&RepositoryFile, i64, String)> = snapshot
        .files
        .iter()
        .map(|file| {
            let (score, reason) = score_file(file, &terms);
            (file, score, reason)
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.1.cmp(&a.1).then_with(|| a.0.relative_path.cmp(&b.0.relative_path))
    });

    let mut selected = Vec::new();
    let mut omitted = Vec::new();
    let mut total = 0usize;

    for (file, score, reason) in ranked {
        if score <= 0 {
            omitted.push(OmittedFile {
                relative_path: file.relative_path.clone(),
                reason: "not relevant to task or supported source".to_string(),
            });
            continue;
        }
        let Some(content) = read_repository_file(file) else {
            omitted.push(OmittedFile {
                relative_path: file.relative_path.clone(),
                reason: if file.is_source {
                    "binary, unreadable, or too large".to_string()
                } else {
                    "not source content".to_string()
                },
            });
            continue;
        };
        let estimated = estimate_tokens(&content);
        if total + estimated > budget_tokens {
            omitted.push(OmittedFile {
                relative_path: file.relative_path.clone(),
                reason: format!("budget exceeded ({estimated} estimated tokens)"),
            });
            continue;
        }
        selected.push(SelectedContextFile {
            relative_path: file.relative_path.clone(),
            content,
            estimated_tokens: estimated,
            score,
            reason,
        });
        total += estimated;
    }

    Ok(ContextSelection {
        task: task.to_string(),
        budget_tokens,
        estimated_tokens: total,
        selected,
        omitted,
    })
}

pub fn inspect_and_select(project_root: &std::path::Path, task: &str, budget_tokens: usize) -> Result<ContextSelection, String> {
    let snapshot = inspect_repository(project_root).map_err(|e| e.to_string())?;
    select_context(&snapshot, task, budget_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn make_tree() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ind-sel-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src").join("config.rs"), "pub fn load() {}").unwrap();
        fs::write(dir.join("src").join("other.rs"), "pub fn other() {}").unwrap();
        fs::write(dir.join("readme.md"), "# Project readme").unwrap();
        fs::write(dir.join("package-lock.json"), "{}").unwrap();
        fs::write(dir.join("unrelated.rs"), "x".repeat(400_000)).unwrap();
        fs::write(dir.join("a.rs"), "// aaaaaaaaaaaaaaaaaaaa").unwrap();
        fs::write(dir.join("b.rs"), "// bbbbbbbbbbbbbbbbbbbb").unwrap();
        dir
    }

    fn snapshot_for(dir: &std::path::Path) -> RepositorySnapshot {
        inspect_repository(dir).unwrap()
    }

    #[test]
    fn estimates_tokens_like_typescript() {
        assert_eq!(estimate_tokens(""), 1);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
    }

    #[test]
    fn extracts_terms_with_stop_words_removed() {
        let terms = task_terms("Fix the bug in src/config and update README.md");
        assert!(terms.contains(&"fix".to_string()));
        assert!(terms.contains(&"config".to_string()));
        assert!(terms.contains(&"update".to_string()));
        assert!(terms.contains(&"readme".to_string()));
        assert!(!terms.contains(&"the".to_string()));
        assert!(!terms.contains(&"in".to_string()));
    }

    #[test]
    fn ranks_path_matches_first() {
        let dir = make_tree();
        let snapshot = snapshot_for(&dir);
        let selection = select_context(&snapshot, "config", 10_000).unwrap();
        assert_eq!(selection.selected[0].relative_path, "src/config.rs");
        assert_eq!(selection.selected[0].reason, "path matches: config");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn omits_zero_score_and_over_budget_files() {
        let dir = make_tree();
        let snapshot = snapshot_for(&dir);
        let selection = select_context(&snapshot, "unrelated-task", 100).unwrap();
        assert!(selection
            .omitted
            .iter()
            .any(|o| o.relative_path == "package-lock.json"
                && o.reason == "not relevant to task or supported source"));
        assert!(selection
            .omitted
            .iter()
            .any(|o| o.relative_path == "unrelated.rs"
                && o.reason == "binary, unreadable, or too large"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn enforces_token_budget() {
        let dir = make_tree();
        let snapshot = snapshot_for(&dir);
        let selection = select_context(&snapshot, "rust", 8).unwrap();
        assert_eq!(selection.selected.len(), 1);
        assert_eq!(selection.omitted[0].reason, "budget exceeded (6 estimated tokens)");
        fs::remove_dir_all(&dir).unwrap();
    }
}
