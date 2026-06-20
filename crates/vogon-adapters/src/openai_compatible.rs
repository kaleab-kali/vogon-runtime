use std::{env, fmt, io::Read, time::Duration};

use serde::{Deserialize, Serialize};
use vogon_core::{ModelAdapter, Result, Step, VogonError};

use crate::retry::sleep_before_retry;

/// Default OpenAI-compatible chat-completions base URL.
pub const DEFAULT_OPENAI_COMPATIBLE_BASE_URL: &str = "https://router.huggingface.co/v1";
/// Default OpenAI-compatible model.
pub const DEFAULT_OPENAI_COMPATIBLE_MODEL: &str = "openai/gpt-oss-120b:fastest";
/// Default OpenAI-compatible request timeout in seconds.
pub const DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS: u64 = 30;
/// Default number of retry attempts for retryable OpenAI-compatible failures.
pub const DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES: u32 = 2;
/// Maximum accepted OpenAI-compatible retry count.
pub const MAX_OPENAI_COMPATIBLE_RETRIES: u32 = 20;

const MAX_OPENAI_COMPATIBLE_ERROR_BODY_CHARS: usize = 2048;
const MAX_OPENAI_COMPATIBLE_ERROR_BODY_BYTES: usize =
    MAX_OPENAI_COMPATIBLE_ERROR_BODY_CHARS * 4 + 4;

#[derive(Clone)]
/// Adapter for OpenAI-compatible chat-completions APIs.
pub struct OpenAiCompatibleModel {
    api_key: String,
    base_url: String,
    model: String,
    max_retries: u32,
    agent: ureq::Agent,
}

impl OpenAiCompatibleModel {
    /// Creates an adapter with the default base URL, model, timeout, and retry count.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_base_url_model_timeout_and_retries(
            api_key,
            DEFAULT_OPENAI_COMPATIBLE_BASE_URL,
            DEFAULT_OPENAI_COMPATIBLE_MODEL,
            Duration::from_secs(DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS),
            DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES,
        )
    }

    /// Creates an adapter using a specific base URL and model.
    pub fn with_base_url_and_model(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self> {
        Self::with_base_url_model_timeout_and_retries(
            api_key,
            base_url,
            model,
            Duration::from_secs(DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS),
            DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES,
        )
    }

    /// Creates an adapter using a specific base URL, model, and timeout.
    pub fn with_base_url_model_and_timeout(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self> {
        Self::with_base_url_model_timeout_and_retries(
            api_key,
            base_url,
            model,
            timeout,
            DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES,
        )
    }

    /// Creates an adapter using a specific base URL, model, timeout, and retry count.
    pub fn with_base_url_model_timeout_and_retries(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        let api_key = api_key.into();
        let base_url = base_url.into();
        let model = model.into();

        if api_key.trim().is_empty() {
            return Err(VogonError::Adapter(
                "OpenAI-compatible API key must not be empty".to_owned(),
            ));
        }

        if base_url.trim().is_empty() {
            return Err(VogonError::Adapter(
                "OpenAI-compatible base URL must not be empty".to_owned(),
            ));
        }

        if model.trim().is_empty() {
            return Err(VogonError::Adapter(
                "OpenAI-compatible model name must not be empty".to_owned(),
            ));
        }

        if timeout.is_zero() {
            return Err(VogonError::Adapter(
                "OpenAI-compatible request timeout must be greater than zero".to_owned(),
            ));
        }

        if max_retries > MAX_OPENAI_COMPATIBLE_RETRIES {
            return Err(VogonError::Adapter(format!(
                "OpenAI-compatible max retries must be at most {MAX_OPENAI_COMPATIBLE_RETRIES}"
            )));
        }

        Ok(Self {
            api_key,
            base_url,
            model,
            max_retries,
            agent: ureq::Agent::config_builder()
                .timeout_global(Some(timeout))
                .http_status_as_error(false)
                .build()
                .into(),
        })
    }

    /// Creates an adapter from `OPENAI_COMPATIBLE_API_KEY`.
    pub fn from_env_with_base_url_model_timeout_and_retries(
        base_url: impl Into<String>,
        model: impl Into<String>,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self> {
        let api_key = env::var("OPENAI_COMPATIBLE_API_KEY").map_err(|_| {
            VogonError::Adapter(
                "OPENAI_COMPATIBLE_API_KEY must be set for the OpenAI-compatible adapter"
                    .to_owned(),
            )
        })?;

        Self::with_base_url_model_timeout_and_retries(
            api_key,
            base_url,
            model,
            timeout,
            max_retries,
        )
    }

    fn chat_completions_url(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }
}

impl fmt::Debug for OpenAiCompatibleModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenAiCompatibleModel")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("max_retries", &self.max_retries)
            .finish_non_exhaustive()
    }
}

impl ModelAdapter for OpenAiCompatibleModel {
    fn complete(&self, _step: &Step, input: &str) -> Result<String> {
        let request = ChatCompletionsRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user",
                content: input.to_owned(),
            }],
            stream: false,
        };
        let request_json = serde_json::to_string(&request).map_err(adapter_error)?;
        let mut retries_remaining = self.max_retries;
        let mut retry_attempt = 0;

        let mut response = loop {
            match self
                .agent
                .post(&self.chat_completions_url())
                .header("Authorization", &format!("Bearer {}", self.api_key))
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
        let response = serde_json::from_str::<ChatCompletionsResponse>(&response_body)
            .map_err(adapter_error)?;

        extract_text(&response).ok_or_else(|| {
            VogonError::Adapter(
                "OpenAI-compatible API response did not include text output".to_owned(),
            )
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
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionsResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

fn extract_text(response: &ChatCompletionsResponse) -> Option<String> {
    response
        .choices
        .first()?
        .message
        .content
        .as_deref()
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

fn http_status_error(mut response: ureq::http::Response<ureq::Body>) -> VogonError {
    let status = response.status();
    let body = truncate_error_body(read_error_body(response.body_mut()));
    VogonError::Adapter(format!(
        "OpenAI-compatible API request failed with HTTP {status}: {body}"
    ))
}

fn http_error(error: ureq::Error) -> VogonError {
    VogonError::Adapter(format!("OpenAI-compatible API request failed: {error}"))
}

fn truncate_error_body(body: String) -> String {
    if body.chars().count() <= MAX_OPENAI_COMPATIBLE_ERROR_BODY_CHARS {
        return body;
    }

    let mut truncated = body
        .chars()
        .take(MAX_OPENAI_COMPATIBLE_ERROR_BODY_CHARS)
        .collect::<String>();
    truncated.push_str("...[truncated]");
    truncated
}

fn read_error_body(body: &mut ureq::Body) -> String {
    let mut bytes = Vec::new();
    let read_result = body
        .as_reader()
        .take(MAX_OPENAI_COMPATIBLE_ERROR_BODY_BYTES as u64)
        .read_to_end(&mut bytes);

    if read_result.is_err() {
        return "<unreadable response body>".to_owned();
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

fn adapter_error(error: impl std::error::Error) -> VogonError {
    VogonError::Adapter(format!("OpenAI-compatible adapter failed: {error}"))
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
        ChatCompletionsResponse, MAX_OPENAI_COMPATIBLE_ERROR_BODY_BYTES,
        MAX_OPENAI_COMPATIBLE_ERROR_BODY_CHARS, MAX_OPENAI_COMPATIBLE_RETRIES,
        OpenAiCompatibleModel, extract_text, truncate_error_body,
    };
    use vogon_core::{ModelAdapter, Step, StepId};

    #[test]
    fn debug_output_redacts_api_key() {
        let model = OpenAiCompatibleModel::new("secret-key").unwrap();

        let debug = format!("{model:?}");

        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret-key"));
    }

    #[test]
    fn model_rejects_empty_api_keys() {
        let result = OpenAiCompatibleModel::new(" ");

        assert!(result.is_err());
    }

    #[test]
    fn model_rejects_zero_timeout() {
        let result = OpenAiCompatibleModel::with_base_url_model_and_timeout(
            "secret-key",
            "https://example.test/v1",
            "example/model",
            Duration::ZERO,
        );

        assert!(result.is_err());
    }

    #[test]
    fn model_rejects_excessive_retries() {
        let result = OpenAiCompatibleModel::with_base_url_model_timeout_and_retries(
            "secret-key",
            "https://example.test/v1",
            "example/model",
            Duration::from_secs(5),
            MAX_OPENAI_COMPATIBLE_RETRIES + 1,
        );

        let error = result.unwrap_err().to_string();
        assert!(error.contains("OpenAI-compatible max retries must be at most 20"));
    }

    #[test]
    fn response_text_uses_first_choice_message_content() {
        let response: ChatCompletionsResponse = serde_json::from_str(
            r#"{
                "choices": [
                    { "message": { "content": "hello world" } }
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
        let body = format!(
            "{}tail",
            "\u{00E9}".repeat(MAX_OPENAI_COMPATIBLE_ERROR_BODY_CHARS + 1)
        );

        let truncated = truncate_error_body(body);

        assert!(truncated.ends_with("...[truncated]"));
        assert!(!truncated.contains("tail"));
        assert_eq!(
            truncated.trim_end_matches("...[truncated]").chars().count(),
            MAX_OPENAI_COMPATIBLE_ERROR_BODY_CHARS
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
                        r#"{"choices":[{"message":{"content":"retried"}}]}"#,
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

        let model = OpenAiCompatibleModel::with_base_url_model_timeout_and_retries(
            "secret-key",
            format!("http://{address}"),
            "example/model",
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

        let model = OpenAiCompatibleModel::with_base_url_model_timeout_and_retries(
            "secret-key",
            format!("http://{address}"),
            "example/model",
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

            let body = format!(
                "{}tail",
                "x".repeat(MAX_OPENAI_COMPATIBLE_ERROR_BODY_BYTES + 1024)
            );
            write!(
                stream,
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        });

        let model = OpenAiCompatibleModel::with_base_url_model_timeout_and_retries(
            "secret-key",
            format!("http://{address}"),
            "example/model",
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
