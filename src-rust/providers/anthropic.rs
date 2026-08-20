use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Serialize;

use crate::providers::types::{
    ChatEvent, ChatMessage, ChatRequest, ChatRole, ProviderAdapter, ProviderCapabilities,
    ProviderError, Usage, error_event,
};

#[derive(Debug, Clone)]
pub struct AnthropicAdapter {
    pub api_key: String,
    pub base_url: String,
}

impl AnthropicAdapter {
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self, String> {
        if api_key.is_empty() {
            return Err("Anthropic requires ANTHROPIC_API_KEY.".to_string());
        }
        Ok(Self {
            api_key,
            base_url: base_url
                .unwrap_or_else(|| "https://api.anthropic.com".to_string())
                .trim_end_matches('/')
                .to_string(),
        })
    }
}

#[derive(Serialize)]
struct AnthropicBody<'a> {
    model: &'a str,
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: &'static str,
    content: String,
}

#[async_trait]
impl ProviderAdapter for AnthropicAdapter {
    fn id(&self) -> &str {
        "anthropic"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            tools: true,
            usage: true,
            cancellation: true,
            json_mode: false,
        }
    }

    async fn stream(
        &self,
        request: &ChatRequest,
        emit: &mut (dyn FnMut(ChatEvent) + Send),
    ) -> Result<(), ProviderError> {
        emit(ChatEvent::Start {
            provider: "anthropic".to_string(),
            model: request.model.clone(),
        });

        let system = request
            .messages
            .iter()
            .filter(|m| m.role == ChatRole::System)
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        let messages: Vec<AnthropicMessage> = request
            .messages
            .iter()
            .filter(|m| m.role != ChatRole::System)
            .map(|m| AnthropicMessage {
                role: if m.role == ChatRole::Assistant {
                    "assistant"
                } else {
                    "user"
                },
                content: m.content.clone(),
            })
            .collect();

        let body = AnthropicBody {
            model: &request.model,
            max_tokens: request.max_output_tokens,
            messages,
            stream: true,
            system: (!system.is_empty()).then_some(system),
        };

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/v1/messages", self.base_url))
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                ProviderError::new("anthropic", format!("Request failed: {e}"), None, true)
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let detail = response.text().await.unwrap_or_default();
            return Err(ProviderError::new(
                "anthropic",
                format!(
                    "Provider returned HTTP {status}: {}",
                    &detail[..detail.len().min(300)]
                ),
                Some(status),
                status == 429 || status >= 500,
            ));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut event_name = String::new();
        let mut finish_reason: Option<String> = None;
        let mut usage: Option<Usage> = None;

        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(bytes) => bytes,
                Err(e) => {
                    emit(error_event("anthropic", format!("Stream read failed: {e}")));
                    break;
                }
            };
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            let lines: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();
            buffer = lines.last().cloned().unwrap_or_default();

            for line in lines[..lines.len().saturating_sub(1)].iter() {
                if let Some(name) = line.strip_prefix("event:") {
                    event_name = name.trim().to_string();
                    continue;
                }
                let Some(payload) = line.trim_start().strip_prefix("data:") else {
                    continue;
                };
                let Ok(data) = serde_json::from_str::<serde_json::Value>(payload.trim()) else {
                    continue;
                };

                if event_name == "message_start" {
                    if let Some(input) = data
                        .get("message")
                        .and_then(|m| m.get("usage"))
                        .and_then(|u| u.get("input_tokens"))
                        .and_then(|v| v.as_u64())
                    {
                        let input = input as usize;
                        usage = Some(Usage {
                            input_tokens: input,
                            output_tokens: 0,
                            total_tokens: input,
                            cached_tokens: None,
                            estimated: false,
                        });
                    }
                }
                if event_name == "content_block_delta" {
                    if let Some(text) = data
                        .get("delta")
                        .and_then(|d| d.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        emit(ChatEvent::Delta {
                            text: text.to_string(),
                        });
                    }
                }
                if event_name == "message_delta" {
                    if let Some(reason) = data
                        .get("delta")
                        .and_then(|d| d.get("stop_reason"))
                        .and_then(|r| r.as_str())
                    {
                        finish_reason = Some(reason.to_string());
                    }
                    if let Some(output) = data
                        .get("usage")
                        .and_then(|u| u.get("output_tokens"))
                        .and_then(|v| v.as_u64())
                    {
                        if let Some(usage_state) = usage.as_mut() {
                            usage_state.output_tokens = output as usize;
                            usage_state.total_tokens = usage_state.input_tokens + output as usize;
                        }
                    }
                    if let Some(usage_state) = usage.clone() {
                        emit(ChatEvent::Usage { usage: usage_state });
                    }
                }
                if event_name == "message_stop" {
                    emit(ChatEvent::Done {
                        finish_reason: finish_reason.clone(),
                    });
                    return Ok(());
                }
            }
        }

        if let Some(usage_state) = usage.clone() {
            emit(ChatEvent::Usage { usage: usage_state });
        }
        emit(ChatEvent::Done {
            finish_reason: finish_reason.clone(),
        });
        Ok(())
    }
}
