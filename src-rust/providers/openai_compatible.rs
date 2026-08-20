use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Serialize;

use crate::providers::types::{
    ChatEvent, ChatMessage, ChatRequest, ChatRole, ProviderAdapter, ProviderCapabilities,
    ProviderError, Usage, error_event,
};

#[derive(Debug, Clone)]
pub struct OpenAICompatibleAdapter {
    pub id: String,
    pub base_url: String,
    pub api_key: Option<String>,
}

impl OpenAICompatibleAdapter {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        if base_url.trim().is_empty() {
            panic!("OpenAI-compatible provider requires a base URL.");
        }
        Self {
            id: "openai-compatible".to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        }
    }

    fn to_message(message: &ChatMessage) -> OpenAiMessage {
        OpenAiMessage {
            role: message.role.as_str(),
            content: message.content.clone(),
            name: message.name.clone(),
            tool_call_id: message.tool_call_id.clone(),
        }
    }
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: &'static str,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "tool_call_id", skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize)]
struct OpenAiBody<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage>,
    max_tokens: usize,
    stream: bool,
    stream_options: StreamOptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct StreamOptions {
    include_usage: bool,
}

fn usage_from(value: &serde_json::Value) -> Option<Usage> {
    let prompt_tokens = value.get("prompt_tokens")?.as_u64()? as usize;
    let completion_tokens = value.get("completion_tokens")?.as_u64()? as usize;
    let cached = value
        .get("prompt_tokens_details")
        .and_then(|d| d.get("cached_tokens"))
        .and_then(|c| c.as_u64())
        .map(|c| c as usize);
    let total = value
        .get("total_tokens")
        .and_then(|t| t.as_u64())
        .map(|t| t as usize)
        .unwrap_or(prompt_tokens + completion_tokens);
    Some(Usage {
        input_tokens: prompt_tokens,
        output_tokens: completion_tokens,
        total_tokens: total,
        cached_tokens: cached,
        estimated: false,
    })
}

fn parse_sse_line(line: &str) -> Option<Option<serde_json::Value>> {
    let trimmed = line.trim_start();
    let payload = trimmed.strip_prefix("data:")?.trim();
    if payload == "[DONE]" {
        return Some(None);
    }
    if payload.is_empty() {
        return None;
    }
    serde_json::from_str(payload).ok().map(Some)
}

#[async_trait]
impl ProviderAdapter for OpenAICompatibleAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            tools: true,
            usage: true,
            cancellation: true,
            json_mode: true,
        }
    }

    async fn stream(
        &self,
        request: &ChatRequest,
        emit: &mut (dyn FnMut(ChatEvent) + Send),
    ) -> Result<(), ProviderError> {
        emit(ChatEvent::Start {
            provider: self.id.clone(),
            model: request.model.clone(),
        });

        let messages: Vec<OpenAiMessage> = request.messages.iter().map(Self::to_message).collect();
        let body = OpenAiBody {
            model: &request.model,
            messages,
            max_tokens: request.max_output_tokens,
            stream: true,
            stream_options: StreamOptions {
                include_usage: true,
            },
            temperature: request.temperature,
            response_format: request
                .json_mode
                .then(|| serde_json::json!({ "type": "json_object" })),
        };

        let client = reqwest::Client::new();
        let mut req = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("content-type", "application/json")
            .header("accept", "text/event-stream");
        if let Some(key) = &self.api_key {
            req = req.header("authorization", format!("Bearer {key}"));
        }

        let response = match req.json(&body).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Err(ProviderError::new(
                    &self.id,
                    format!("Request failed: {e}"),
                    None,
                    true,
                ));
            }
        };

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let detail = response.text().await.unwrap_or_default();
            return Err(ProviderError::new(
                &self.id,
                format!(
                    "Provider returned HTTP {status}{}",
                    if detail.is_empty() {
                        String::new()
                    } else {
                        format!(": {}", &detail[..detail.len().min(300)])
                    }
                ),
                Some(status),
                status == 408 || status == 429 || status >= 500,
            ));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut finish_reason: Option<String> = None;

        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(bytes) => bytes,
                Err(e) => {
                    emit(error_event(&self.id, format!("Stream read failed: {e}")));
                    break;
                }
            };
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            let lines: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();
            buffer = lines.last().cloned().unwrap_or_default();

            for line in lines[..lines.len().saturating_sub(1)].iter() {
                let Some(parsed) = parse_sse_line(line) else {
                    continue;
                };
                let Some(data) = parsed else {
                    emit(ChatEvent::Done {
                        finish_reason: finish_reason.clone(),
                    });
                    return Ok(());
                };
                if let Some(usage) = data.get("usage").and_then(usage_from) {
                    emit(ChatEvent::Usage { usage });
                }
                let Some(choice) = data
                    .get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                else {
                    continue;
                };
                if let Some(reason) = choice.get("finish_reason").and_then(|r| r.as_str()) {
                    finish_reason = Some(reason.to_string());
                }
                if let Some(delta) = choice
                    .get("delta")
                    .and_then(|d| d.get("content"))
                    .and_then(|c| c.as_str())
                    && !delta.is_empty()
                {
                    emit(ChatEvent::Delta {
                        text: delta.to_string(),
                    });
                }
            }
        }

        emit(ChatEvent::Done {
            finish_reason: finish_reason.clone(),
        });
        Ok(())
    }
}

pub fn usage_message(provider: &str, usage: &Usage) -> String {
    format!(
        "[{provider}] {input} input + {output} output = {total} tokens",
        input = usage.input_tokens,
        output = usage.output_tokens,
        total = usage.total_tokens
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sse_lines() {
        assert!(parse_sse_line("event: ping").is_none());
        assert_eq!(parse_sse_line("data: [DONE]"), Some(None));
        assert_eq!(
            parse_sse_line("data: {\"x\":1}"),
            Some(Some(serde_json::json!({"x": 1})))
        );
        assert!(parse_sse_line("data: not-json").is_none());
    }

    #[test]
    fn parses_openai_usage() {
        let value = serde_json::json!({
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15,
            "prompt_tokens_details": {"cached_tokens": 3}
        });
        let usage = usage_from(&value).unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 5);
        assert_eq!(usage.total_tokens, 15);
        assert_eq!(usage.cached_tokens, Some(3));
    }

    #[test]
    fn message_roles_serialize_lowercase() {
        let msg = ChatMessage::new(ChatRole::Assistant, "hi");
        assert_eq!(msg.role.as_str(), "assistant");
    }
}
