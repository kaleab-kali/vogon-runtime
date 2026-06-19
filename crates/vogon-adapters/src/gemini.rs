use std::{env, fmt, time::Duration};

use serde::{Deserialize, Serialize};
use vogon_core::{ModelAdapter, Result, Step, VogonError};

pub const DEFAULT_GEMINI_MODEL: &str = "gemini-3.1-flash-lite";
pub const DEFAULT_GEMINI_TIMEOUT_SECONDS: u64 = 30;

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com";
const MAX_GEMINI_ERROR_BODY_CHARS: usize = 2048;

#[derive(Clone)]
pub struct GeminiModel {
    api_key: String,
    model: String,
    api_base: String,
    agent: ureq::Agent,
}

impl GeminiModel {
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_model(api_key, DEFAULT_GEMINI_MODEL)
    }

    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        Self::with_model_and_timeout(
            api_key,
            model,
            Duration::from_secs(DEFAULT_GEMINI_TIMEOUT_SECONDS),
        )
    }

    pub fn with_model_and_timeout(
        api_key: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self> {
        Self::with_base_url(api_key, model, GEMINI_API_BASE, timeout)
    }

    pub fn from_env(model: impl Into<String>) -> Result<Self> {
        Self::from_env_with_timeout(model, Duration::from_secs(DEFAULT_GEMINI_TIMEOUT_SECONDS))
    }

    pub fn from_env_with_timeout(model: impl Into<String>, timeout: Duration) -> Result<Self> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
            VogonError::Adapter("GEMINI_API_KEY must be set for the Gemini adapter".to_owned())
        })?;

        Self::with_model_and_timeout(api_key, model, timeout)
    }

    fn with_base_url(
        api_key: impl Into<String>,
        model: impl Into<String>,
        api_base: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self> {
        let api_key = api_key.into();
        let model = model.into();
        let api_base = api_base.into();

        if api_key.trim().is_empty() {
            return Err(VogonError::Adapter(
                "Gemini API key must not be empty".to_owned(),
            ));
        }

        if model.trim().is_empty() {
            return Err(VogonError::Adapter(
                "Gemini model name must not be empty".to_owned(),
            ));
        }

        if timeout.is_zero() {
            return Err(VogonError::Adapter(
                "Gemini request timeout must be greater than zero".to_owned(),
            ));
        }

        Ok(Self {
            api_key,
            model,
            api_base,
            agent: ureq::AgentBuilder::new().timeout(timeout).build(),
        })
    }

    fn generate_content_url(&self) -> String {
        format!(
            "{}/v1beta/models/{}:generateContent",
            self.api_base.trim_end_matches('/'),
            self.model
        )
    }
}

impl fmt::Debug for GeminiModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GeminiModel")
            .field("api_key", &"<redacted>")
            .field("model", &self.model)
            .field("api_base", &self.api_base)
            .finish_non_exhaustive()
    }
}

impl ModelAdapter for GeminiModel {
    fn complete(&self, _step: &Step, input: &str) -> Result<String> {
        let request = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: input.to_owned(),
                }],
            }],
        };
        let response = self
            .agent
            .post(&self.generate_content_url())
            .set("x-goog-api-key", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(serde_json::to_value(request).map_err(adapter_error)?)
            .map_err(http_error)?
            .into_json::<GenerateContentResponse>()
            .map_err(adapter_error)?;

        extract_text(&response).ok_or_else(|| {
            VogonError::Adapter("Gemini API response did not include text output".to_owned())
        })
    }
}

#[derive(Debug, Serialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GenerateContentResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

fn extract_text(response: &GenerateContentResponse) -> Option<String> {
    let text = response
        .candidates
        .first()?
        .content
        .parts
        .iter()
        .filter_map(|part| part.text.as_deref())
        .collect::<Vec<_>>()
        .join("");

    if text.is_empty() { None } else { Some(text) }
}

fn http_error(error: ureq::Error) -> VogonError {
    match error {
        ureq::Error::Status(status, response) => {
            let body = truncate_error_body(
                response
                    .into_string()
                    .unwrap_or_else(|_| "<unreadable response body>".to_owned()),
            );
            VogonError::Adapter(format!(
                "Gemini API request failed with HTTP {status}: {body}"
            ))
        }
        ureq::Error::Transport(error) => {
            VogonError::Adapter(format!("Gemini API request failed: {error}"))
        }
    }
}

fn truncate_error_body(body: String) -> String {
    if body.chars().count() <= MAX_GEMINI_ERROR_BODY_CHARS {
        return body;
    }

    let mut truncated = body
        .chars()
        .take(MAX_GEMINI_ERROR_BODY_CHARS)
        .collect::<String>();
    truncated.push_str("...[truncated]");
    truncated
}

fn adapter_error(error: impl std::error::Error) -> VogonError {
    VogonError::Adapter(format!("Gemini adapter failed: {error}"))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        GeminiModel, GenerateContentResponse, MAX_GEMINI_ERROR_BODY_CHARS, extract_text,
        truncate_error_body,
    };

    #[test]
    fn debug_output_redacts_api_key() {
        let model = GeminiModel::new("secret-key").unwrap();

        let debug = format!("{model:?}");

        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret-key"));
    }

    #[test]
    fn gemini_model_rejects_empty_api_keys() {
        let result = GeminiModel::new(" ");

        assert!(result.is_err());
    }

    #[test]
    fn gemini_model_rejects_zero_timeout() {
        let result = GeminiModel::with_model_and_timeout(
            "secret-key",
            "gemini-3.1-flash-lite",
            Duration::ZERO,
        );

        assert!(result.is_err());
    }

    #[test]
    fn response_text_combines_text_parts() {
        let response: GenerateContentResponse = serde_json::from_str(
            r#"{
                "candidates": [
                    {
                        "content": {
                            "parts": [
                                { "text": "hello" },
                                { "text": " world" }
                            ]
                        }
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(extract_text(&response).as_deref(), Some("hello world"));
    }

    #[test]
    fn short_error_bodies_are_not_truncated() {
        let body = "provider error".to_owned();

        assert_eq!(truncate_error_body(body), "provider error");
    }

    #[test]
    fn long_error_bodies_are_truncated_on_char_boundaries() {
        let body = format!("{}tail", "é".repeat(MAX_GEMINI_ERROR_BODY_CHARS + 1));

        let truncated = truncate_error_body(body);

        assert!(truncated.ends_with("...[truncated]"));
        assert!(!truncated.contains("tail"));
        assert_eq!(
            truncated.trim_end_matches("...[truncated]").chars().count(),
            MAX_GEMINI_ERROR_BODY_CHARS
        );
    }
}
