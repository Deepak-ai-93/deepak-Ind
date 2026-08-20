use crate::tasks::types::{ChunkAction, FileEditAction, TaskPlan};
use crate::tools::commands::{run_project_command, CommandResult, RunCommandOptions};
use crate::tools::files::write_project_file;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum TaskRunEvent {
    ApprovalRequired { chunk_id: String, message: String },
    ChunkStart { chunk_id: String, message: String },
    EditApplied { chunk_id: String, message: String },
    CommandFinished { chunk_id: String, message: String },
    ChunkPassed { chunk_id: String, message: String },
    ChunkFailed { chunk_id: String, message: String },
    PlanComplete { message: String },
}

impl TaskRunEvent {
    pub fn message(&self) -> &str {
        match self {
            TaskRunEvent::ApprovalRequired { message, .. }
            | TaskRunEvent::ChunkStart { message, .. }
            | TaskRunEvent::EditApplied { message, .. }
            | TaskRunEvent::CommandFinished { message, .. }
            | TaskRunEvent::ChunkPassed { message, .. }
            | TaskRunEvent::ChunkFailed { message, .. }
            | TaskRunEvent::PlanComplete { message } => message,
        }
    }
}

pub struct RunPlanOptions<'a> {
    pub project_root: &'a Path,
    pub approved_chunks: &'a [String],
    pub policy: Option<crate::policy::IndPolicy>,
}

pub fn run_task_plan(
    plan: &mut TaskPlan,
    actions: &HashMap<String, Vec<ChunkAction>>,
    options: &RunPlanOptions,
    on_event: &mut dyn FnMut(TaskRunEvent),
) -> Result<(), String> {
    for chunk in &mut plan.chunks {
        chunk.status = "waiting-approval".to_string();
        on_event(TaskRunEvent::ApprovalRequired {
            chunk_id: chunk.id.clone(),
            message: format!(
                "Approval required for chunk {}: {}",
                chunk.sequence, chunk.title
            ),
        });

        if !options.approved_chunks.iter().any(|id| *id == chunk.id) {
            chunk.status = "blocked".to_string();
            return Err(format!("Chunk {} is blocked until approved.", chunk.id));
        }

        chunk.status = "running".to_string();
        on_event(TaskRunEvent::ChunkStart {
            chunk_id: chunk.id.clone(),
            message: format!("Starting chunk {}: {}", chunk.sequence, chunk.title),
        });

        let run_result = (|| -> Result<(), String> {
            for action in actions.get(&chunk.id).cloned().unwrap_or_default() {
                match action {
                    ChunkAction::Edit(FileEditAction {
                        path,
                        content,
                        expected_content,
                    }) => {
                        write_project_file(
                            options.project_root,
                            &path,
                            &content,
                            expected_content.as_deref(),
                        )
                        .map_err(|e| format!("Failed to apply edit {}: {e}", path))?;
                        on_event(TaskRunEvent::EditApplied {
                            chunk_id: chunk.id.clone(),
                            message: format!("Applied edit: {path}"),
                        });
                    }
                    ChunkAction::Command(cmd) => {
                        let result: CommandResult = run_project_command(
                            &cmd.command,
                            options.project_root,
                            &RunCommandOptions {
                                timeout_ms: cmd.timeout_ms,
                                approved: true,
                                policy: options.policy.clone(),
                            },
                        )
                        .map_err(|e| format!("Command failed to run: {e}"))?;
                        on_event(TaskRunEvent::CommandFinished {
                            chunk_id: chunk.id.clone(),
                            message: format!(
                                "{} exited {}",
                                cmd.command,
                                result
                                    .exit_code
                                    .map(|c| c.to_string())
                                    .unwrap_or_else(|| "unknown".to_string())
                            ),
                        });
                        if result.timed_out || result.exit_code != Some(0) {
                            return Err(format!(
                                "Verification failed for '{}': {}",
                                cmd.command,
                                if result.stderr.is_empty() {
                                    result.stdout
                                } else {
                                    result.stderr
                                }
                            ));
                        }
                    }
                }
            }
            Ok(())
        })();

        match run_result {
            Ok(()) => {
                chunk.status = "passed".to_string();
                on_event(TaskRunEvent::ChunkPassed {
                    chunk_id: chunk.id.clone(),
                    message: format!("Chunk {} passed.", chunk.sequence),
                });
            }
            Err(error) => {
                chunk.status = "failed".to_string();
                on_event(TaskRunEvent::ChunkFailed {
                    chunk_id: chunk.id.clone(),
                    message: format!("Chunk {} failed: {error}", chunk.sequence),
                });
                return Err(error);
            }
        }
    }

    on_event(TaskRunEvent::PlanComplete {
        message: format!("Task plan {} completed.", plan.id),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::types::CommandAction;

    fn chunk_id(plan: &TaskPlan) -> String {
        plan.chunks[0].id.clone()
    }

    #[test]
    fn blocked_chunk_fails_without_approval() {
        let mut plan = create_plan();
        let mut events = Vec::new();
        let err = run_task_plan(
            &mut plan,
            &HashMap::new(),
            &RunPlanOptions {
                project_root: Path::new("."),
                approved_chunks: &[],
                policy: None,
            },
            &mut |e| events.push(e),
        )
        .unwrap_err();
        assert!(err.contains("blocked until approved"));
        assert_eq!(plan.chunks[0].status, "blocked");
        assert!(events
            .iter()
            .any(|e| matches!(e, TaskRunEvent::ApprovalRequired { .. })));
    }

    #[test]
    fn passes_approved_empty_chunks() {
        let mut plan = create_plan();
        let approved = plan
            .chunks
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let mut events = Vec::new();
        run_task_plan(
            &mut plan,
            &HashMap::new(),
            &RunPlanOptions {
                project_root: Path::new("."),
                approved_chunks: &approved,
                policy: None,
            },
            &mut |e| events.push(e),
        )
        .unwrap();
        assert!(plan.chunks.iter().all(|c| c.status == "passed"));
        assert!(events
            .iter()
            .any(|e| matches!(e, TaskRunEvent::PlanComplete { .. })));
    }

    #[test]
    fn failing_command_marks_chunk_failed() {
        let dir = std::env::temp_dir().join(format!("ind-runner-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let mut plan = create_plan();
        let approved = plan
            .chunks
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let mut actions = HashMap::new();
        actions.insert(
            chunk_id(&plan),
            vec![ChunkAction::Command(CommandAction {
                command: if cfg!(target_os = "windows") {
                    "Write-Error boom; exit 1".to_string()
                } else {
                    "exit 1".to_string()
                },
                timeout_ms: None,
            })],
        );

        let result = run_task_plan(
            &mut plan,
            &actions,
            &RunPlanOptions {
                project_root: &dir,
                approved_chunks: &approved,
                policy: None,
            },
            &mut |_| {},
        );
        assert!(result.is_err());
        assert_eq!(plan.chunks[0].status, "failed");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    fn create_plan() -> TaskPlan {
        crate::tasks::planner::create_task_plan("run tests", &[]).unwrap()
    }
}