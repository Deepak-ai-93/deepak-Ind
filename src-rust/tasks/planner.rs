use chrono::{SecondsFormat, Utc};
use rand::Rng;

use crate::tasks::types::{TaskChunk, TaskPlan};

pub fn create_task_plan(task: &str, context_files: &[String]) -> Result<TaskPlan, String> {
    let normalized = task.trim();
    if normalized.is_empty() {
        return Err("Task description cannot be empty.".to_string());
    }
    let id: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    let id = format!("plan-{id}");

    let mut unique_files: Vec<String> = context_files.to_vec();
    unique_files.sort();
    unique_files.dedup();

    Ok(TaskPlan {
        id: id.clone(),
        task: normalized.to_string(),
        context_files: unique_files,
        created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        chunks: vec![
            TaskChunk {
                id: format!("{id}-01"),
                sequence: 1,
                title: "Understand the change".to_string(),
                goal: format!("Confirm the smallest change needed for: {normalized}"),
                status: "pending".to_string(),
                verification: vec![
                    "Context is selected and the task scope is explicit.".to_string(),
                ],
            },
            TaskChunk {
                id: format!("{id}-02"),
                sequence: 2,
                title: "Implement the change".to_string(),
                goal: format!("Apply the approved code change for: {normalized}"),
                status: "pending".to_string(),
                verification: vec![
                    "The requested files are changed and the diff is reviewable.".to_string(),
                ],
            },
            TaskChunk {
                id: format!("{id}-03"),
                sequence: 3,
                title: "Verify the change".to_string(),
                goal: "Run the configured verification command and report the result.".to_string(),
                status: "pending".to_string(),
                verification: vec!["The verification command exits successfully.".to_string()],
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_tasks() {
        assert!(create_task_plan("  ", &[]).is_err());
    }

    #[test]
    fn builds_three_chunk_plan() {
        let plan =
            create_task_plan("fix the bug", &["a.rs".to_string(), "a.rs".to_string()]).unwrap();
        assert_eq!(plan.chunks.len(), 3);
        assert_eq!(plan.context_files, vec!["a.rs".to_string()]);
        assert_eq!(plan.chunks[0].sequence, 1);
        assert_eq!(plan.chunks[2].title, "Verify the change");
        assert!(plan.chunks.iter().all(|c| c.status == "pending"));
    }
}
