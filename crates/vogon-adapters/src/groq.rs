use std::{env, fmt, time::Duration};

use vogon_core::{ModelAdapter, Result, RuntimeMetadata, Step, VogonError};

use crate::openai_compatible::{MAX_OPENAI_COMPATIBLE_RETRIES, OpenAiCompatibleModel};

/// Default Groq OpenAI-compatible API base URL.
pub const DEFAULT_GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
/// Default Groq model used by [`GroqModel`].
pub const DEFAULT_GROQ_MODEL: &str = "llama-3.1-8b-instant";
/// Default Groq request timeout in seconds.
pub const DEFAULT_GROQ_TIMEOUT_SECONDS: u64 = 30;
/// Default number of retry attempts for retryable Groq failures.
pub const DEFAULT_GROQ_MAX_RETRIES: u32 = 2;
/// Maximum accepted Groq retry count.
pub const MAX_GROQ_RETRIES: u32 = MAX_OPENAI_COMPATIBLE_RETRIES;

/// Groq model adapter using Groq's OpenAI-compatible chat-completions endpoint.
#[derive(Clone)]
pub struct GroqModel {
    inner: OpenAiCompatibleModel,
    base_url: String,
    model: String,
    timeout: Duration,
    max_retries: u32,
}

impl GroqModel {
    /// Creates a Groq adapter using the default model, timeout, and retry count.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_model(api_key, DEFAULT_GROQ_MODEL)
    }

    /// Creates a Groq adapter using a specific model and default timeout.
    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        Self::with_model_timeout_and_retries(
            api_key,
            model,
            Duration::from_secs(DEFAULT_GROQ_TIMEOUT_SECONDS),
            DEFAULT_GROQ_MAX_RETRIES,
        )
    }

    /// Creates a Groq adapter using a specific model, timeout, and retry count.
    pub fn with_model_timeout_and_retries(
        api_key: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        Self::with_base_url(api_key, DEFAULT_GROQ_BASE_URL, model, timeout, max_retries)
    }

    /// Creates a Groq adapter from the `GROQ_API_KEY` environment variable.
    pub fn from_env_with_timeout_and_retries(
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        let api_key = env::var("GROQ_API_KEY").map_err(|_| {
            VogonError::Adapter("GROQ_API_KEY must be set for the Groq adapter".to_owned())
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

impl fmt::Debug for GroqModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GroqModel")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish_non_exhaustive()
    }
}

impl ModelAdapter for GroqModel {
    fn complete(&self, step: &Step, input: &str) -> Result<String> {
        self.inner.complete(step, input)
    }

    fn cache_identity(&self) -> String {
        format!(
            "vogon-adapters@{}:groq:v1:base={}:model={}:timeout_nanos={}:max_retries={}",
            env!("CARGO_PKG_VERSION"),
            self.base_url.trim_end_matches('/'),
            self.model,
            self.timeout.as_nanos(),
            self.max_retries
        )
    }

    fn runtime_metadata(&self) -> RuntimeMetadata {
        RuntimeMetadata::new(
            "groq",
            "groq-openai-compatible-chat-completions",
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

    use super::{GroqModel, MAX_GROQ_RETRIES};
    use vogon_core::ModelAdapter;

    #[test]
    fn debug_output_redacts_api_key() {
        let model = GroqModel::new("secret-key").unwrap();

        let debug = format!("{model:?}");

        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret-key"));
    }

    #[test]
    fn cache_identity_includes_model_and_omits_api_key() {
        let model = GroqModel::with_model("secret-key", "groq-test").unwrap();

        let identity = model.cache_identity();

        assert!(identity.contains("groq"));
        assert!(identity.contains("groq-test"));
        assert!(!identity.contains("secret-key"));
    }

    #[test]
    fn runtime_metadata_describes_groq_provider_and_omits_api_key() {
        let model = GroqModel::with_model_timeout_and_retries(
            "secret-key",
            "groq-test",
            Duration::from_secs(5),
            1,
        )
        .unwrap();

        let metadata = model.runtime_metadata();

        assert_eq!(metadata.provider, "groq");
        assert_eq!(metadata.adapter, "groq-openai-compatible-chat-completions");
        assert_eq!(metadata.model.as_deref(), Some("groq-test"));
        assert_eq!(
            metadata.parameters.get("max_retries").map(String::as_str),
            Some("1")
        );
        assert!(!metadata.cache_identity.contains("secret-key"));
    }

    #[test]
    fn groq_model_rejects_excessive_retries() {
        let result = GroqModel::with_model_timeout_and_retries(
            "secret-key",
            "llama-3.1-8b-instant",
            Duration::from_secs(5),
            MAX_GROQ_RETRIES + 1,
        );

        let error = result.unwrap_err().to_string();
        assert!(error.contains("OpenAI-compatible max retries must be at most 20"));
    }
}
