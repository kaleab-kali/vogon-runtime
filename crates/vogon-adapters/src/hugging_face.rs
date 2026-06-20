use std::{env, fmt, time::Duration};

use vogon_core::{ModelAdapter, Result, RuntimeMetadata, Step, VogonError};

use crate::openai_compatible::{
    DEFAULT_OPENAI_COMPATIBLE_BASE_URL, DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES,
    DEFAULT_OPENAI_COMPATIBLE_MODEL, DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS,
    MAX_OPENAI_COMPATIBLE_RETRIES, OpenAiCompatibleModel,
};

/// Default Hugging Face OpenAI-compatible API base URL.
pub const DEFAULT_HUGGING_FACE_BASE_URL: &str = DEFAULT_OPENAI_COMPATIBLE_BASE_URL;
/// Default Hugging Face model used by [`HuggingFaceModel`].
pub const DEFAULT_HUGGING_FACE_MODEL: &str = DEFAULT_OPENAI_COMPATIBLE_MODEL;
/// Default Hugging Face request timeout in seconds.
pub const DEFAULT_HUGGING_FACE_TIMEOUT_SECONDS: u64 = DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS;
/// Default number of retry attempts for retryable Hugging Face failures.
pub const DEFAULT_HUGGING_FACE_MAX_RETRIES: u32 = DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES;
/// Maximum accepted Hugging Face retry count.
pub const MAX_HUGGING_FACE_RETRIES: u32 = MAX_OPENAI_COMPATIBLE_RETRIES;

/// Hugging Face model adapter using the Inference Providers OpenAI-compatible endpoint.
#[derive(Clone)]
pub struct HuggingFaceModel {
    inner: OpenAiCompatibleModel,
    base_url: String,
    model: String,
    timeout: Duration,
    max_retries: u32,
}

impl HuggingFaceModel {
    /// Creates a Hugging Face adapter using the default model, timeout, and retry count.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_model(api_key, DEFAULT_HUGGING_FACE_MODEL)
    }

    /// Creates a Hugging Face adapter using a specific model and default timeout.
    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        Self::with_model_timeout_and_retries(
            api_key,
            model,
            Duration::from_secs(DEFAULT_HUGGING_FACE_TIMEOUT_SECONDS),
            DEFAULT_HUGGING_FACE_MAX_RETRIES,
        )
    }

    /// Creates a Hugging Face adapter using a specific model, timeout, and retry count.
    pub fn with_model_timeout_and_retries(
        api_key: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        Self::with_base_url(
            api_key,
            DEFAULT_HUGGING_FACE_BASE_URL,
            model,
            timeout,
            max_retries,
        )
    }

    /// Creates a Hugging Face adapter from the `HF_TOKEN` environment variable.
    pub fn from_env_with_timeout_and_retries(
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        let api_key = env::var("HF_TOKEN").map_err(|_| {
            VogonError::Adapter("HF_TOKEN must be set for the Hugging Face adapter".to_owned())
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

impl fmt::Debug for HuggingFaceModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HuggingFaceModel")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish_non_exhaustive()
    }
}

impl ModelAdapter for HuggingFaceModel {
    fn complete(&self, step: &Step, input: &str) -> Result<String> {
        self.inner.complete(step, input)
    }

    fn cache_identity(&self) -> String {
        format!(
            "vogon-adapters@{}:hugging-face:v1:base={}:model={}:timeout_nanos={}:max_retries={}",
            env!("CARGO_PKG_VERSION"),
            self.base_url.trim_end_matches('/'),
            self.model,
            self.timeout.as_nanos(),
            self.max_retries
        )
    }

    fn runtime_metadata(&self) -> RuntimeMetadata {
        RuntimeMetadata::new(
            "hugging-face",
            "hugging-face-openai-compatible-chat-completions",
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

    use super::{HuggingFaceModel, MAX_HUGGING_FACE_RETRIES};
    use vogon_core::ModelAdapter;

    #[test]
    fn debug_output_redacts_api_key() {
        let model = HuggingFaceModel::new("secret-key").unwrap();

        let debug = format!("{model:?}");

        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret-key"));
    }

    #[test]
    fn cache_identity_includes_model_and_omits_api_key() {
        let model = HuggingFaceModel::with_model("secret-key", "hf-test").unwrap();

        let identity = model.cache_identity();

        assert!(identity.contains("hugging-face"));
        assert!(identity.contains("hf-test"));
        assert!(!identity.contains("secret-key"));
    }

    #[test]
    fn runtime_metadata_describes_hugging_face_provider_and_omits_api_key() {
        let model = HuggingFaceModel::with_model_timeout_and_retries(
            "secret-key",
            "hf-test",
            Duration::from_secs(5),
            1,
        )
        .unwrap();

        let metadata = model.runtime_metadata();

        assert_eq!(metadata.provider, "hugging-face");
        assert_eq!(
            metadata.adapter,
            "hugging-face-openai-compatible-chat-completions"
        );
        assert_eq!(metadata.model.as_deref(), Some("hf-test"));
        assert_eq!(
            metadata.parameters.get("max_retries").map(String::as_str),
            Some("1")
        );
        assert!(!metadata.cache_identity.contains("secret-key"));
    }

    #[test]
    fn hugging_face_model_rejects_excessive_retries() {
        let result = HuggingFaceModel::with_model_timeout_and_retries(
            "secret-key",
            "openai/gpt-oss-120b:fastest",
            Duration::from_secs(5),
            MAX_HUGGING_FACE_RETRIES + 1,
        );

        let error = result.unwrap_err().to_string();
        assert!(error.contains("OpenAI-compatible max retries must be at most 20"));
    }
}
