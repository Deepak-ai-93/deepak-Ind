use async_trait::async_trait;

pub mod anthropic;
pub mod discovery;
pub mod google;
pub mod index;
pub mod openai_compatible;
pub mod types;

pub use anthropic::AnthropicAdapter;
pub use discovery::{LocalRuntime, discover_local_runtimes};
pub use google::GoogleAdapter;
pub use index::{create_configured_provider, local_runtime_summary, provider_summary};
pub use openai_compatible::OpenAICompatibleAdapter;
pub use types::{
    ChatEvent, ChatMessage, ChatRequest, ChatRole, ChatTool, ProviderAdapter, ProviderCapabilities,
    ProviderError, Usage, error_event,
};

#[async_trait]
impl ProviderAdapter for Box<dyn ProviderAdapter> {
    fn id(&self) -> &str {
        (**self).id()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        (**self).capabilities()
    }

    async fn stream(
        &self,
        request: &ChatRequest,
        emit: &mut (dyn FnMut(ChatEvent) + Send),
    ) -> Result<(), ProviderError> {
        (**self).stream(request, emit).await
    }
}
