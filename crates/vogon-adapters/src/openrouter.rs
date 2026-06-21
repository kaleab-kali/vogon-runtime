use std::{env, fmt, time::Duration};

use vogon_core::{ModelAdapter, Result, RuntimeMetadata, Step, VogonError};

use crate::openai_compatible::{MAX_OPENAI_COMPATIBLE_RETRIES, OpenAiCompatibleModel};

/// Default OpenRouter OpenAI-compatible API base URL.
pub const DEFAULT_OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
/// Default OpenRouter model used by [`OpenRouterModel`].
pub const DEFAULT_OPENROUTER_MODEL: &str = "openrouter/free";
/// Default OpenRouter request timeout in seconds.
pub const DEFAULT_OPENROUTER_TIMEOUT_SECONDS: u64 = 30;
/// Default number of retry attempts for retryable OpenRouter failures.
pub const DEFAULT_OPENROUTER_MAX_RETRIES: u32 = 2;
/// Maximum accepted OpenRouter retry count.
pub const MAX_OPENROUTER_RETRIES: u32 = MAX_OPENAI_COMPATIBLE_RETRIES;

/// OpenRouter model adapter using OpenRouter's OpenAI-compatible endpoint.
#[derive(Clone)]
pub struct OpenRouterModel {
    inner: OpenAiCompatibleModel,
    base_url: String,
    model: String,
    timeout: Duration,
    max_retries: u32,
}

impl OpenRouterModel {
    /// Creates an OpenRouter adapter using the default model, timeout, and retry count.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_model(api_key, DEFAULT_OPENROUTER_MODEL)
    }

    /// Creates an OpenRouter adapter using a specific model and default timeout.
    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        Self::with_model_timeout_and_retries(
            api_key,
            model,
            Duration::from_secs(DEFAULT_OPENROUTER_TIMEOUT_SECONDS),
            DEFAULT_OPENROUTER_MAX_RETRIES,
        )
    }

    /// Creates an OpenRouter adapter using a specific model, timeout, and retry count.
    pub fn with_model_timeout_and_retries(
        api_key: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        Self::with_base_url(
            api_key,
            DEFAULT_OPENROUTER_BASE_URL,
            model,
            timeout,
            max_retries,
        )
    }

    /// Creates an OpenRouter adapter from the `OPENROUTER_API_KEY` environment variable.
    pub fn from_env_with_timeout_and_retries(
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
            VogonError::Adapter(
                "OPENROUTER_API_KEY must be set for the OpenRouter adapter".to_owned(),
            )
        })?;

        Self::with_model_timeout_and_retries(api_key, model, timeout, max_retries)
    }

    fn with_base_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        let base_url = base_url.into();
        let model = model.into();
        let inner = OpenAiCompatibleModel::with_base_url_model_timeout_and_retries(
            api_key,
            base_url.clone(),
            model.clone(),
            timeout,
            max_retries,
        )?;

        Ok(Self {
            inner,
            base_url,
            model,
            timeout,
            max_retries,
        })
    }
}

impl fmt::Debug for OpenRouterModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenRouterModel")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish_non_exhaustive()
    }
}

impl ModelAdapter for OpenRouterModel {
    fn complete(&self, step: &Step, input: &str) -> Result<String> {
        self.inner.complete(step, input)
    }

    fn cache_identity(&self) -> String {
        format!(
            "vogon-adapters@{}:openrouter:v1:base={}:model={}:timeout_nanos={}:max_retries={}",
            env!("CARGO_PKG_VERSION"),
            self.base_url.trim_end_matches('/'),
            self.model,
            self.timeout.as_nanos(),
            self.max_retries
        )
    }

    fn runtime_metadata(&self) -> RuntimeMetadata {
        RuntimeMetadata::new(
            "openrouter",
            "openrouter-openai-compatible-chat-completions",
            env!("CARGO_PKG_VERSION"),
            self.cache_identity(),
        )
        .with_model(self.model.clone())
        .with_parameter("base_url", self.base_url.trim_end_matches('/'))
        .with_parameter("timeout_nanos", self.timeout.as_nanos().to_string())
        .with_parameter("max_retries", self.max_retries.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{MAX_OPENROUTER_RETRIES, OpenRouterModel};
    use vogon_core::ModelAdapter;

    #[test]
    fn debug_output_redacts_api_key() {
        let model = OpenRouterModel::new("secret-key").unwrap();

        let debug = format!("{model:?}");

        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret-key"));
    }

    #[test]
    fn cache_identity_includes_model_and_omits_api_key() {
        let model = OpenRouterModel::with_model("secret-key", "openrouter-test").unwrap();

        let identity = model.cache_identity();

        assert!(identity.contains("openrouter"));
        assert!(identity.contains("openrouter-test"));
        assert!(!identity.contains("secret-key"));
    }

    #[test]
    fn runtime_metadata_describes_openrouter_provider_and_omits_api_key() {
        let model = OpenRouterModel::with_model_timeout_and_retries(
            "secret-key",
            "openrouter-test",
            Duration::from_secs(5),
            1,
        )
        .unwrap();

        let metadata = model.runtime_metadata();

        assert_eq!(metadata.provider, "openrouter");
        assert_eq!(
            metadata.adapter,
            "openrouter-openai-compatible-chat-completions"
        );
        assert_eq!(metadata.model.as_deref(), Some("openrouter-test"));
        assert_eq!(
            metadata.parameters.get("max_retries").map(String::as_str),
            Some("1")
        );
        assert!(!metadata.cache_identity.contains("secret-key"));
    }

    #[test]
    fn openrouter_model_rejects_excessive_retries() {
        let result = OpenRouterModel::with_model_timeout_and_retries(
            "secret-key",
            "openrouter/free",
            Duration::from_secs(5),
            MAX_OPENROUTER_RETRIES + 1,
        );

        let error = result.unwrap_err().to_string();
        assert!(error.contains("OpenAI-compatible max retries must be at most 20"));
    }
}
