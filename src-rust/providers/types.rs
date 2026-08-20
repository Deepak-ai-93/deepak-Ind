use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

impl ChatRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatRole::System => "system",
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::Tool => "tool",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "toolCallId", skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn new(role: ChatRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            name: None,
            tool_call_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderCapabilities {
    pub streaming: bool,
    pub tools: bool,
    pub usage: bool,
    pub cancellation: bool,
    pub json_mode: bool,
}

impl ProviderCapabilities {
    pub fn enabled_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.streaming {
            names.push("streaming");
        }
        if self.tools {
            names.push("tools");
        }
        if self.usage {
            names.push("usage");
        }
        if self.cancellation {
            names.push("cancellation");
        }
        if self.json_mode {
            names.push("jsonMode");
        }
        names
    }
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub cached_tokens: Option<usize>,
    pub estimated: bool,
}

#[derive(Debug, Clone)]
pub struct ChatTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_output_tokens: usize,
    pub temperature: Option<f64>,
    pub tools: Option<Vec<ChatTool>>,
    pub json_mode: bool,
}

#[derive(Debug, Clone)]
pub enum ChatEvent {
    Start { provider: String, model: String },
    Delta { text: String },
    Usage { usage: Usage },
    Done { finish_reason: Option<String> },
    Error { message: String },
}

#[derive(Debug)]
pub struct ProviderError {
    pub provider: String,
    pub message: String,
    pub status: Option<u16>,
    pub retryable: bool,
}

impl ProviderError {
    pub fn new(
        provider: &str,
        message: impl Into<String>,
        status: Option<u16>,
        retryable: bool,
    ) -> Self {
        Self {
            provider: provider.to_string(),
            message: message.into(),
            status,
            retryable,
        }
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.provider, self.message)
    }
}

impl std::error::Error for ProviderError {}

pub fn error_event(provider: &str, message: String) -> ChatEvent {
    ChatEvent::Error {
        message: format!("[{provider}] {message}"),
    }
}

#[async_trait::async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;
    async fn stream(
        &self,
        request: &ChatRequest,
        emit: &mut (dyn FnMut(ChatEvent) + Send),
    ) -> Result<(), ProviderError>;
}
