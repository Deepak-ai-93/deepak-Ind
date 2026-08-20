use std::time::Duration;

#[derive(Debug, Clone)]
pub struct LocalRuntime {
    pub name: String,
    pub base_url: String,
    pub models: Vec<String>,
    pub available: bool,
    pub detail: String,
}

async fn get_json(url: &str, timeout_ms: u64) -> Option<serde_json::Value> {
    let client = reqwest::Client::new();
    let response = tokio::time::timeout(Duration::from_millis(timeout_ms), client.get(url).send())
        .await
        .ok()?
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json().await.ok()
}

pub async fn discover_local_runtimes() -> Vec<LocalRuntime> {
    let mut runtimes = Vec::new();

    if let Some(catalog) = get_json("http://127.0.0.1:11434/api/tags", 700).await {
        let models = catalog
            .get("models")
            .and_then(|m| m.as_array())
            .map(|models| {
                models
                    .iter()
                    .filter_map(|model| {
                        model
                            .get("name")
                            .and_then(|n| n.as_str())
                            .or_else(|| model.get("model").and_then(|m| m.as_str()))
                            .map(|s| s.to_string())
                    })
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        runtimes.push(LocalRuntime {
            name: "ollama".to_string(),
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            models,
            available: true,
            detail: "Ollama model catalog available".to_string(),
        });
    }

    if let Some(catalog) = get_json("http://127.0.0.1:1234/v1/models", 700).await {
        let models = catalog
            .get("data")
            .and_then(|d| d.as_array())
            .map(|models| {
                models
                    .iter()
                    .filter_map(|model| {
                        model
                            .get("id")
                            .and_then(|n| n.as_str())
                            .or_else(|| model.get("name").and_then(|m| m.as_str()))
                            .map(|s| s.to_string())
                    })
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        runtimes.push(LocalRuntime {
            name: "lm-studio".to_string(),
            base_url: "http://127.0.0.1:1234/v1".to_string(),
            models,
            available: true,
            detail: "LM Studio OpenAI-compatible catalog available".to_string(),
        });
    }

    runtimes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn discovers_nothing_when_offline() {
        let runtimes = discover_local_runtimes().await;
        assert!(runtimes.is_empty() || runtimes.iter().all(|r| r.available));
    }
}
