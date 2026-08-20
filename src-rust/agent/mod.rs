pub mod system_prompt;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use colored::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::IndConfig;
use crate::context::selector::inspect_and_select;
use crate::memory::MemoryManager;
use crate::policy::{IndPolicy, load_policy};
use crate::providers::types::{
    ChatEvent, ChatMessage, ChatRequest, ChatRole, ProviderAdapter, ProviderError,
};
use crate::tools::commands::{RunCommandOptions, run_project_command};
use crate::tools::files::{read_project_file, write_project_file};
use crate::usage::UsageLedger;

#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallPayload {
    pub tool: String,
    #[serde(default)]
    pub parameters: Value,
}

#[derive(Debug, Clone)]
pub struct AgentSession {
    pub config: IndConfig,
    pub policy: IndPolicy,
    pub history: Vec<ChatMessage>,
    pub total_input_tokens: usize,
    pub total_output_tokens: usize,
}

impl AgentSession {
    pub fn new(config: IndConfig) -> Self {
        let policy = load_policy(&config.project_root).unwrap_or_default();
        Self {
            config,
            policy,
            history: Vec::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
        }
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.total_input_tokens = 0;
        self.total_output_tokens = 0;
    }

    pub fn compact(&mut self) {
        if self.history.len() > 6 {
            let keep = self.history.split_off(self.history.len() - 6);
            self.history = keep;
        }
    }

    pub async fn run_turn<F>(
        &mut self,
        provider: &dyn ProviderAdapter,
        user_input: &str,
        mut on_token: F,
    ) -> Result<String, String>
    where
        F: FnMut(&str) + Send,
    {
        // 1. Inspect repository context
        let context_selection = inspect_and_select(
            &self.config.project_root,
            user_input,
            self.config.max_input_tokens / 2,
        )
        .ok();

        let context_file_paths: Vec<String> = context_selection
            .as_ref()
            .map(|cs| {
                cs.selected
                    .iter()
                    .map(|f| f.relative_path.clone())
                    .collect()
            })
            .unwrap_or_default();

        let memory_notes = MemoryManager::new(&self.config.project_root).read().ok();

        let system_text = system_prompt::build_system_prompt_with_context(
            &self.config.project_root.to_string_lossy(),
            &self.config.provider,
            &self.config.model,
            &context_file_paths,
            memory_notes.as_deref(),
        );

        // Append user message
        self.history
            .push(ChatMessage::new(ChatRole::User, user_input));

        let max_turns = self.config.max_tool_turns.max(1);
        let mut final_response = String::new();

        for _turn in 0..max_turns {
            let mut messages = vec![ChatMessage::new(ChatRole::System, &system_text)];
            messages.extend(self.history.clone());

            let req = ChatRequest {
                model: self.config.model.clone(),
                messages,
                max_output_tokens: self.config.max_output_tokens,
                temperature: Some(0.2),
                tools: None,
                json_mode: false,
            };

            let mut assistant_buffer = String::new();
            let mut turn_input_tokens = 0;
            let mut turn_output_tokens = 0;

            let stream_res = provider
                .stream(&req, &mut |event| match event {
                    ChatEvent::Delta { text } => {
                        assistant_buffer.push_str(&text);
                        on_token(&text);
                    }
                    ChatEvent::Usage { usage } => {
                        turn_input_tokens = usage.input_tokens;
                        turn_output_tokens = usage.output_tokens;
                    }
                    _ => {}
                })
                .await;

            if let Err(e) = stream_res {
                return Err(format!("Provider streaming error: {e}"));
            }

            self.total_input_tokens += turn_input_tokens;
            self.total_output_tokens += turn_output_tokens;

            // Record to SQLite ledger
            if let Ok(ledger) = UsageLedger::init(&self.config.project_root) {
                let _ = ledger.record(
                    &self.config.provider,
                    &self.config.model,
                    turn_input_tokens,
                    turn_output_tokens,
                );
            }

            self.history.push(ChatMessage::new(
                ChatRole::Assistant,
                assistant_buffer.clone(),
            ));

            // Check for tool call
            if let Some(tool_call) = extract_tool_call(&assistant_buffer) {
                if tool_call.tool == "finish" {
                    let msg = tool_call
                        .parameters
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Task completed.");
                    final_response = msg.to_string();
                    break;
                }

                println!(
                    "\n{} Executing tool: {}",
                    "⚙ [Agent]".bold().yellow(),
                    tool_call.tool.bold().cyan()
                );

                let tool_result = self.execute_tool(&tool_call);
                println!(
                    "{} Result: {}\n",
                    "✔ [Tool Result]".green().bold(),
                    truncate_str(&tool_result, 120)
                );

                self.history.push(ChatMessage::new(
                    ChatRole::User,
                    format!("Tool `{}` returned:\n{}", tool_call.tool, tool_result),
                ));
            } else {
                // No tool call requested — model finished direct response
                final_response = assistant_buffer;
                break;
            }
        }

        Ok(final_response)
    }

    fn execute_tool(&self, call: &ToolCallPayload) -> String {
        match call.tool.as_str() {
            "read_file" => {
                let path = call
                    .parameters
                    .get("path")
                    .and_then(|p| p.as_str())
                    .unwrap_or_default();
                match read_project_file(&self.config.project_root, path) {
                    Ok(content) => content,
                    Err(e) => format!("Error reading {path}: {e}"),
                }
            }
            "write_file" => {
                let path = call
                    .parameters
                    .get("path")
                    .and_then(|p| p.as_str())
                    .unwrap_or_default();
                let content = call
                    .parameters
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or_default();
                match write_project_file(&self.config.project_root, path, content, None) {
                    Ok(written) => format!("Successfully wrote to {}", written.display()),
                    Err(e) => format!("Error writing {path}: {e}"),
                }
            }
            "list_files" => {
                let rel_path = call
                    .parameters
                    .get("path")
                    .and_then(|p| p.as_str())
                    .unwrap_or(".");
                let dir = self.config.project_root.join(rel_path);
                match fs::read_dir(&dir) {
                    Ok(entries) => {
                        let mut items = Vec::new();
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let is_dir = entry.path().is_dir();
                            items.push(format!("{}{}", name, if is_dir { "/" } else { "" }));
                        }
                        items.join("\n")
                    }
                    Err(e) => format!("Error listing {rel_path}: {e}"),
                }
            }
            "run_command" => {
                let cmd = call
                    .parameters
                    .get("command")
                    .and_then(|c| c.as_str())
                    .unwrap_or_default();
                let options = RunCommandOptions {
                    timeout_ms: Some(60_000),
                    approved: true,
                    policy: Some(self.policy.clone()),
                };
                match run_project_command(cmd, &self.config.project_root, &options) {
                    Ok(res) => {
                        let mut out = format!("Exit Code: {:?}\n", res.exit_code);
                        if !res.stdout.is_empty() {
                            out.push_str(&format!("STDOUT:\n{}\n", res.stdout));
                        }
                        if !res.stderr.is_empty() {
                            out.push_str(&format!("STDERR:\n{}\n", res.stderr));
                        }
                        out
                    }
                    Err(e) => format!("Command error: {e}"),
                }
            }
            unknown => format!("Unknown tool `{unknown}`. Please use available tools."),
        }
    }
}

fn extract_tool_call(text: &str) -> Option<ToolCallPayload> {
    let re = regex::Regex::new(r"(?s)```tool_call\s*(\{.*?\})\s*```").ok()?;
    let caps = re.captures(text)?;
    let json_str = caps.get(1)?.as_str();
    serde_json::from_str::<ToolCallPayload>(json_str).ok()
}

fn truncate_str(s: &str, max: usize) -> String {
    let trimmed = s.trim();
    if trimmed.len() > max {
        format!("{}...", &trimmed[..max])
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_tool_call() {
        let raw = "Let me read the file.\n```tool_call\n{\n  \"tool\": \"read_file\",\n  \"parameters\": {\"path\": \"Cargo.toml\"}\n}\n```";
        let parsed = extract_tool_call(raw).expect("Should parse tool call");
        assert_eq!(parsed.tool, "read_file");
        assert_eq!(parsed.parameters.get("path").unwrap(), "Cargo.toml");
    }

    #[test]
    fn returns_none_on_regular_text() {
        let raw = "Here is how to solve the bug without calling any tool.";
        assert!(extract_tool_call(raw).is_none());
    }
}
