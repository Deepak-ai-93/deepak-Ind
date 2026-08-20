use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskChunk {
    pub id: String,
    pub sequence: usize,
    pub title: String,
    pub goal: String,
    pub status: String,
    pub verification: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub id: String,
    pub task: String,
    pub context_files: Vec<String>,
    pub chunks: Vec<TaskChunk>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct FileEditAction {
    pub path: String,
    pub content: String,
    pub expected_content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommandAction {
    pub command: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum ChunkAction {
    Edit(FileEditAction),
    Command(CommandAction),
}