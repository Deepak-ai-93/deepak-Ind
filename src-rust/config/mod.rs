use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalMode {
    Chunk,
    Command,
    Never,
}

impl Default for ApprovalMode {
    fn default() -> Self {
        ApprovalMode::Chunk
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutingMode {
    Auto,
    Off,
}

impl Default for RoutingMode {
    fn default() -> Self {
        RoutingMode::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndConfig {
    pub project_root: PathBuf,
    pub provider: String,
    pub model: String,
    pub cheap_model: String,
    pub strong_model: String,
    pub routing: RoutingMode,
    pub base_url: Option<String>,
    pub approval: ApprovalMode,
    pub max_input_tokens: usize,
    pub max_output_tokens: usize,
    pub max_tool_turns: usize,
}

#[derive(Debug, Default, Deserialize)]
struct RawConfigFile {
    provider: Option<String>,
    model: Option<String>,
    #[serde(rename = "cheapModel")]
    cheap_model: Option<String>,
    #[serde(rename = "strongModel")]
    strong_model: Option<String>,
    routing: Option<String>,
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
    approval: Option<String>,
    #[serde(rename = "maxInputTokens")]
    max_input_tokens: Option<usize>,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: Option<usize>,
    #[serde(rename = "maxToolTurns")]
    max_tool_turns: Option<usize>,
}

pub fn find_project_root(start: Option<&Path>) -> PathBuf {
    let current_dir = start
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut current = current_dir.clone();
    loop {
        if current.join(".git").exists() || current.join(".ind").exists() || current.join("Cargo.toml").exists() || current.join("package.json").exists() {
            return current;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    current_dir
}

pub fn load_config(start: Option<&Path>) -> IndConfig {
    let project_root = find_project_root(start);
    let cfg_path = env::var("IND_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| project_root.join(".ind").join("config.json"));

    let file_cfg: RawConfigFile = if cfg_path.exists() {
        fs::read_to_string(&cfg_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        RawConfigFile::default()
    };

    let provider = env::var("IND_PROVIDER")
        .ok()
        .or(file_cfg.provider)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "openai-compatible".to_string());

    let model = env::var("IND_MODEL")
        .ok()
        .or(file_cfg.model)
        .unwrap_or_default();

    let cheap_model = env::var("IND_CHEAP_MODEL")
        .ok()
        .or(file_cfg.cheap_model)
        .unwrap_or_default();

    let strong_model = env::var("IND_STRONG_MODEL")
        .ok()
        .or(file_cfg.strong_model)
        .unwrap_or_default();

    let base_url = env::var("IND_BASE_URL")
        .ok()
        .or(file_cfg.base_url)
        .filter(|s| !s.trim().is_empty());

    let approval = match env::var("IND_APPROVAL").ok().or(file_cfg.approval).as_deref() {
        Some("command") => ApprovalMode::Command,
        Some("never") => ApprovalMode::Never,
        _ => ApprovalMode::Chunk,
    };

    let routing = match env::var("IND_ROUTING").ok().or(file_cfg.routing).as_deref() {
        Some("off") => RoutingMode::Off,
        _ => RoutingMode::Auto,
    };

    let max_input_tokens = env::var("IND_MAX_INPUT_TOKENS")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.max_input_tokens)
        .unwrap_or(12_000);

    let max_output_tokens = env::var("IND_MAX_OUTPUT_TOKENS")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.max_output_tokens)
        .unwrap_or(4_000);

    let max_tool_turns = env::var("IND_MAX_TOOL_TURNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.max_tool_turns)
        .unwrap_or(8);

    IndConfig {
        project_root,
        provider,
        model,
        cheap_model,
        strong_model,
        routing,
        base_url,
        approval,
        max_input_tokens,
        max_output_tokens,
        max_tool_turns,
    }
}
