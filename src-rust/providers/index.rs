use std::env;

use crate::config::load_config;
use crate::policy::{assert_provider_allowed, load_policy};

use super::anthropic::AnthropicAdapter;
use super::discovery::discover_local_runtimes;
use super::google::GoogleAdapter;
use super::openai_compatible::OpenAICompatibleAdapter;
use super::types::ProviderAdapter;

pub fn create_configured_provider() -> Result<Box<dyn ProviderAdapter>, String> {
    let config = load_config(None);
    let policy = load_policy(&config.project_root)?;
    assert_provider_allowed(&config.provider, &policy)?;

    match config.provider.as_str() {
        "anthropic" => Ok(Box::new(AnthropicAdapter::new(
            env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            config.base_url.clone(),
        )?)),
        "google" => Ok(Box::new(GoogleAdapter::new(
            env::var("GOOGLE_GENERATIVE_AI_API_KEY").unwrap_or_default(),
            config.base_url.clone(),
        )?)),
        "openai" => Ok(Box::new(OpenAICompatibleAdapter {
            id: "openai".to_string(),
            base_url: config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            api_key: env::var("OPENAI_API_KEY").ok().filter(|k| !k.is_empty()),
        })),
        _ => Ok(Box::new(OpenAICompatibleAdapter::new(
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434/v1".to_string()),
            env::var("OPENAI_API_KEY").ok().filter(|k| !k.is_empty()),
        ))),
    }
}

pub fn provider_summary() -> Result<Vec<String>, String> {
    let config = load_config(None);
    let policy = load_policy(&config.project_root)?;
    assert_provider_allowed(&config.provider, &policy)?;
    let capabilities = create_configured_provider()?.capabilities();
    Ok(vec![
        format!("configured: {}", config.provider),
        format!(
            "model: {}",
            if config.model.is_empty() {
                "not selected"
            } else {
                &config.model
            }
        ),
        format!(
            "endpoint: {}",
            config.base_url.as_deref().unwrap_or("provider default")
        ),
        format!("capabilities: {}", capabilities.enabled_names().join(", ")),
    ])
}

pub async fn local_runtime_summary() -> Result<Vec<String>, String> {
    let runtimes = discover_local_runtimes().await;
    if runtimes.is_empty() {
        return Ok(vec![
            "no local runtimes detected (probes: Ollama 11434, LM Studio 1234)".to_string(),
        ]);
    }
    Ok(runtimes
        .into_iter()
        .flat_map(|runtime| {
            let models = if runtime.models.is_empty() {
                "none reported".to_string()
            } else {
                runtime.models.join(", ")
            };
            vec![
                format!("{}: {}", runtime.name, runtime.base_url),
                format!("models: {models}"),
            ]
        })
        .collect())
}
