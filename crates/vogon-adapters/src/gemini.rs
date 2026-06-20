use std::{env, fmt, io::Read, time::Duration};

use serde::{Deserialize, Serialize};
use vogon_core::{ModelAdapter, Result, Step, VogonError};

use crate::retry::sleep_before_retry;

/// Default Gemini model used by [`GeminiModel`].
pub const DEFAULT_GEMINI_MODEL: &str = "gemini-3.1-flash-lite";
/// Default Gemini request timeout in seconds.
pub const DEFAULT_GEMINI_TIMEOUT_SECONDS: u64 = 30;
/// Default number of retry attempts for retryable Gemini failures.
pub const DEFAULT_GEMINI_MAX_RETRIES: u32 = 2;
/// Maximum accepted Gemini retry count.
pub const MAX_GEMINI_RETRIES: u32 = 20;

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com";
const MAX_GEMINI_ERROR_BODY_CHARS: usize = 2048;
const MAX_GEMINI_ERROR_BODY_BYTES: usize = MAX_GEMINI_ERROR_BODY_CHARS * 4 + 4;

#[derive(Clone)]
/// Gemini API model adapter.
pub struct GeminiModel {
    api_key: String,
    model: String,
    api_base: String,
    max_retries: u32,
    agent: ureq::Agent,
}

impl GeminiModel {
    /// Creates a Gemini adapter using the default model, timeout, and retry count.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_model(api_key, DEFAULT_GEMINI_MODEL)
    }

    /// Creates a Gemini adapter using a specific model and default timeout.
    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        Self::with_model_and_timeout(
            api_key,
            model,
            Duration::from_secs(DEFAULT_GEMINI_TIMEOUT_SECONDS),
        )
    }

    /// Creates a Gemini adapter using a specific model and request timeout.
    pub fn with_model_and_timeout(
        api_key: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self> {
        Self::with_model_timeout_and_retries(api_key, model, timeout, DEFAULT_GEMINI_MAX_RETRIES)
    }

    /// Creates a Gemini adapter using a specific model, timeout, and retry count.
    pub fn with_model_timeout_and_retries(
        api_key: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        Self::with_base_url(api_key, model, GEMINI_API_BASE, timeout, max_retries)
    }

    /// Creates a Gemini adapter from the `GEMINI_API_KEY` environment variable.
    pub fn from_env(model: impl Into<String>) -> Result<Self> {
        Self::from_env_with_timeout(model, Duration::from_secs(DEFAULT_GEMINI_TIMEOUT_SECONDS))
    }

    /// Creates a Gemini adapter from the environment with a custom timeout.
    pub fn from_env_with_timeout(model: impl Into<String>, timeout: Duration) -> Result<Self> {
        Self::from_env_with_timeout_and_retries(model, timeout, DEFAULT_GEMINI_MAX_RETRIES)
    }

    /// Creates a Gemini adapter from the environment with a custom timeout and retry count.
    pub fn from_env_with_timeout_and_retries(
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
            VogonError::Adapter("GEMINI_API_KEY must be set for the Gemini adapter".to_owned())
        })?;

        Self::with_model_timeout_and_retries(api_key, model, timeout, max_retries)
    }

    fn with_base_url(
        api_key: impl Into<String>,
        model: impl Into<String>,
        api_base: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
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

        if max_retries > MAX_GEMINI_RETRIES {
            return Err(VogonError::Adapter(format!(
                "Gemini max retries must be at most {MAX_GEMINI_RETRIES}"
            )));
        }

        Ok(Self {
            api_key,
            model,
            api_base,
            max_retries,
            agent: ureq::Agent::config_builder()
                .timeout_global(Some(timeout))
                .http_status_as_error(false)
                .build()
                .into(),
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
            .field("max_retries", &self.max_retries)
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
        let request_json = serde_json::to_string(&request).map_err(adapter_error)?;
        let mut retries_remaining = self.max_retries;
        let mut retry_attempt = 0;

        let mut response = loop {
            match self
                .agent
                .post(&self.generate_content_url())
                .header("x-goog-api-key", &self.api_key)
                .header("Content-Type", "application/json")
                .send(request_json.as_str())
            {
                Ok(response) if response.status().is_success() => break response,
                Ok(response) if retries_remaining > 0 && is_retryable_status(response.status()) => {
                    sleep_before_retry(retry_attempt);
                    retry_attempt += 1;
                    retries_remaining -= 1;
                }
                Ok(response) => return Err(http_status_error(response)),
                Err(error) if retries_remaining > 0 && is_retryable_error(&error) => {
                    sleep_before_retry(retry_attempt);
                    retry_attempt += 1;
                    retries_remaining -= 1;
                }
                Err(error) => return Err(http_error(error)),
            }
        };
        let response_body = response
            .body_mut()
            .read_to_string()
            .map_err(adapter_error)?;
        let response = serde_json::from_str::<GenerateContentResponse>(&response_body)
            .map_err(adapter_error)?;

        extract_text(&response).ok_or_else(|| {
            VogonError::Adapter("Gemini API response did not include text output".to_owned())
        })
    }
}

fn is_retryable_error(error: &ureq::Error) -> bool {
    !matches!(error, ureq::Error::BodyExceedsLimit(_))
}

fn is_retryable_status(status: ureq::http::StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 409 | 425 | 429 | 500..=599)
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

fn http_status_error(mut response: ureq::http::Response<ureq::Body>) -> VogonError {
    let status = response.status();
    let body = truncate_error_body(read_error_body(response.body_mut()));
    VogonError::Adapter(format!(
        "Gemini API request failed with HTTP {status}: {body}"
    ))
}

fn http_error(error: ureq::Error) -> VogonError {
    VogonError::Adapter(format!("Gemini API request failed: {error}"))
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

fn read_error_body(body: &mut ureq::Body) -> String {
    let mut bytes = Vec::new();
    let read_result = body
        .as_reader()
        .take(MAX_GEMINI_ERROR_BODY_BYTES as u64)
        .read_to_end(&mut bytes);

    if read_result.is_err() {
        return "<unreadable response body>".to_owned();
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

fn adapter_error(error: impl std::error::Error) -> VogonError {
    VogonError::Adapter(format!("Gemini adapter failed: {error}"))
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        thread,
        time::Duration,
    };

    use super::{
        GeminiModel, GenerateContentResponse, MAX_GEMINI_ERROR_BODY_BYTES,
        MAX_GEMINI_ERROR_BODY_CHARS, MAX_GEMINI_RETRIES, extract_text, truncate_error_body,
    };
    use vogon_core::{ModelAdapter, Step, StepId};

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
    fn gemini_model_rejects_excessive_retries() {
        let result = GeminiModel::with_model_timeout_and_retries(
            "secret-key",
            "gemini-3.1-flash-lite",
            Duration::from_secs(5),
            MAX_GEMINI_RETRIES + 1,
        );

        let error = result.unwrap_err().to_string();
        assert!(error.contains("Gemini max retries must be at most 20"));
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
        let body = format!("{}tail", "\u{00E9}".repeat(MAX_GEMINI_ERROR_BODY_CHARS + 1));

        let truncated = truncate_error_body(body);

        assert!(truncated.ends_with("...[truncated]"));
        assert!(!truncated.contains("tail"));
        assert_eq!(
            truncated.trim_end_matches("...[truncated]").chars().count(),
            MAX_GEMINI_ERROR_BODY_CHARS
        );
    }

    #[test]
    fn retryable_status_errors_are_retried() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(AtomicUsize::new(0));
        let server_requests = Arc::clone(&requests);
        let server = thread::spawn(move || {
            for response_index in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                read_http_request(&mut stream);
                server_requests.fetch_add(1, Ordering::SeqCst);

                let (status, body) = if response_index == 0 {
                    ("500 Internal Server Error", r#"{"error":"retry"}"#)
                } else {
                    (
                        "200 OK",
                        r#"{"candidates":[{"content":{"parts":[{"text":"retried"}]}}]}"#,
                    )
                };

                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        });

        let model = GeminiModel::with_base_url(
            "secret-key",
            "gemini-3.1-flash-lite",
            format!("http://{address}"),
            Duration::from_secs(5),
            1,
        )
        .unwrap();
        let step = Step::new(StepId::new("classify").unwrap(), "Classify");

        let output = model.complete(&step, "input").unwrap();

        server.join().unwrap();
        assert_eq!(output, "retried");
        assert_eq!(requests.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn non_retryable_status_errors_include_response_body() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            read_http_request(&mut stream);

            let body = r#"{"error":"bad request"}"#;
            write!(
                stream,
                "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        });

        let model = GeminiModel::with_base_url(
            "secret-key",
            "gemini-3.1-flash-lite",
            format!("http://{address}"),
            Duration::from_secs(5),
            1,
        )
        .unwrap();
        let step = Step::new(StepId::new("classify").unwrap(), "Classify");

        let error = model.complete(&step, "input").unwrap_err();

        server.join().unwrap();
        let error = error.to_string();
        assert!(error.contains("HTTP 400 Bad Request"));
        assert!(error.contains(r#"{"error":"bad request"}"#));
    }

    #[test]
    fn non_retryable_status_error_bodies_are_bounded_before_truncation() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            read_http_request(&mut stream);

            let body = format!("{}tail", "x".repeat(MAX_GEMINI_ERROR_BODY_BYTES + 1024));
            write!(
                stream,
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        });

        let model = GeminiModel::with_base_url(
            "secret-key",
            "gemini-3.1-flash-lite",
            format!("http://{address}"),
            Duration::from_secs(5),
            0,
        )
        .unwrap();
        let step = Step::new(StepId::new("classify").unwrap(), "Classify");

        let error = model.complete(&step, "input").unwrap_err();

        server.join().unwrap();
        let error = error.to_string();
        assert!(error.contains("...[truncated]"));
        assert!(!error.contains("tail"));
    }

    fn read_http_request(stream: &mut TcpStream) {
        let mut buffer = Vec::new();
        let mut chunk = [0; 1024];

        loop {
            let bytes_read = stream.read(&mut chunk).unwrap();
            if bytes_read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..bytes_read]);

            let Some(header_end) = find_header_end(&buffer) else {
                continue;
            };
            let content_length = content_length(&buffer[..header_end]);
            if buffer.len() >= header_end + 4 + content_length {
                break;
            }
        }
    }

    fn find_header_end(buffer: &[u8]) -> Option<usize> {
        buffer.windows(4).position(|window| window == b"\r\n\r\n")
    }

    fn content_length(headers: &[u8]) -> usize {
        String::from_utf8_lossy(headers)
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().ok())
                    .flatten()
            })
            .unwrap_or(0)
    }
}
