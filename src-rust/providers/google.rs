use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Serialize;

use crate::providers::types::{
    error_event, ChatEvent, ChatMessage, ChatRequest, ChatRole, ProviderAdapter,
    ProviderCapabilities, ProviderError, Usage,
};

#[derive(Debug, Clone)]
pub struct GoogleAdapter {
    pub api_key: String,
    pub base_url: String,
}

impl GoogleAdapter {
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self, String> {
        if api_key.is_empty() {
            return Err("Google requires GOOGLE_GENERATIVE_AI_API_KEY.".to_string());
        }
        Ok(Self {
            api_key,
            base_url: base_url
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string())
                .trim_end_matches('/')
                .to_string(),
        })
    }
}

#[derive(Serialize)]
struct GooglePart {
    text: String,
}

#[derive(Serialize)]
struct GoogleContent {
    role: &'static str,
    parts: Vec<GooglePart>,
}

#[derive(Serialize)]
struct GoogleGenerationConfig {
    max_output_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
}

#[derive(Serialize)]
struct GoogleBody {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<serde_json::Value>,
    generation_config: GoogleGenerationConfig,
}

#[async_trait]
impl ProviderAdapter for GoogleAdapter {
    fn id(&self) -> &str {
        "google"
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
            provider: "google".to_string(),
            model: request.model.clone(),
        });

        let system = request
            .messages
            .iter()
            .filter(|m| m.role == ChatRole::System)
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        let contents: Vec<GoogleContent> = request
            .messages
            .iter()
            .filter(|m| m.role != ChatRole::System)
            .map(|m| GoogleContent {
                role: if m.role == ChatRole::Assistant { "model" } else { "user" },
                parts: vec![GooglePart {
                    text: m.content.clone(),
                }],
            })
            .collect();

        let body = GoogleBody {
            contents,
            system_instruction: (!system.is_empty()).then(|| {
                serde_json::json!({ "parts": [{ "text": system }] })
            }),
            generation_config: GoogleGenerationConfig {
                max_output_tokens: request.max_output_tokens,
                temperature: request.temperature,
            },
        };

        let encoded_model = urlencoding_light(&request.model);
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse",
            self.base_url, encoded_model
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("content-type", "application/json")
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::new("google", format!("Request failed: {e}"), None, true))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let detail = response.text().await.unwrap_or_default();
            return Err(ProviderError::new(
                "google",
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
        let mut usage: Option<Usage> = None;

        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(bytes) => bytes,
                Err(e) => {
                    emit(error_event("google", format!("Stream read failed: {e}")));
                    break;
                }
            };
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            let lines: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();
            buffer = lines.last().cloned().unwrap_or_default();

            for line in lines[..lines.len().saturating_sub(1)].iter() {
                let Some(payload) = line.trim_start().strip_prefix("data:") else {
                    continue;
                };
                let Ok(data) = serde_json::from_str::<serde_json::Value>(payload.trim()) else {
                    continue;
                };
                if let Some(text) = data
                    .get("candidates")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                    .and_then(|c| c.get("content"))
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.as_array())
                    .and_then(|p| p.first())
                    .and_then(|p| p.get("text"))
                    .and_then(|t| t.as_str())
                {
                    if !text.is_empty() {
                        emit(ChatEvent::Delta {
                            text: text.to_string(),
                        });
                    }
                }
                if let Some(metadata) = data.get("usageMetadata") {
                    let prompt = metadata.get("promptTokenCount").and_then(|v| v.as_u64());
                    let candidates = metadata.get("candidatesTokenCount").and_then(|v| v.as_u64());
                    if let (Some(prompt), Some(candidates)) = (prompt, candidates) {
                        let total = metadata
                            .get("totalTokenCount")
                            .and_then(|v| v.as_u64())
                            .map(|t| t as usize)
                            .unwrap_or((prompt + candidates) as usize);
                        usage = Some(Usage {
                            input_tokens: prompt as usize,
                            output_tokens: candidates as usize,
                            total_tokens: total,
                            cached_tokens: None,
                            estimated: false,
                        });
                    }
                }
            }
        }

        if let Some(usage_state) = usage {
            emit(ChatEvent::Usage { usage: usage_state });
        }
        emit(ChatEvent::Done { finish_reason: None });
        Ok(())
    }
}

fn urlencoding_light(input: &str) -> String {
    let mut out = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::anthropic::AnthropicAdapter;

    #[test]
    fn encodes_model_names() {
        assert_eq!(urlencoding_light("gemini-2.0-flash"), "gemini-2.0-flash");
        assert_eq!(urlencoding_light("models/a:b"), "models%2Fa%3Ab");
    }

    #[test]
    fn requires_api_key() {
        assert!(GoogleAdapter::new(String::new(), None).is_err());
        assert!(GoogleAdapter::new("key".to_string(), None).is_ok());
        assert!(AnthropicAdapter::new(String::new(), None).is_err());
        assert!(AnthropicAdapter::new("key".to_string(), None).is_ok());
    }
}
