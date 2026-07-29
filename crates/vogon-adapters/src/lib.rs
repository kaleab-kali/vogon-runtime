//! Model adapters for Vogon Runtime.
//!
//! `vogon-adapters` contains [`ModelAdapter`](vogon_core::ModelAdapter)
//! implementations that can be plugged into `vogon-core`.
//!
//! The current crate exposes [`DeterministicEchoModel`], a deterministic adapter
//! intended for local development, tests, examples, and replay fixtures. With
//! default features enabled, it also exposes [`GeminiModel`],
//! [`OpenAiCompatibleModel`], [`HuggingFaceModel`], [`GroqModel`], and
//! [`OpenRouterModel`], and [`NvidiaModel`] for real provider-backed API calls.
//!
//! ```
//! use vogon_adapters::DeterministicEchoModel;
//! use vogon_core::{Runtime, Step, StepId, Workflow};
//!
//! let workflow = Workflow::new(
//!     "example",
//!     vec![Step::new(StepId::new("outline").unwrap(), "Write an outline.")],
//! )
//! .unwrap();
//!
//! let report = Runtime::new(DeterministicEchoModel).run(&workflow).unwrap();
//! assert_eq!(report.workflow_name, "example");
//! assert_eq!(report.steps.len(), 1);
//! ```

#![forbid(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]

#[cfg(any(feature = "gemini", feature = "openai-compatible"))]
const MAX_PROVIDER_RESPONSE_BODY_BYTES: u64 = 1024 * 1024;

mod fake;
#[cfg(feature = "gemini")]
mod gemini;
#[cfg(feature = "openai-compatible")]
mod groq;
#[cfg(feature = "openai-compatible")]
mod hugging_face;
#[cfg(feature = "openai-compatible")]
mod nvidia;
#[cfg(feature = "openai-compatible")]
mod openai_compatible;
#[cfg(feature = "openai-compatible")]
mod openrouter;
#[cfg(any(feature = "gemini", feature = "openai-compatible"))]
mod retry;

pub use fake::DeterministicEchoModel;
#[cfg(feature = "gemini")]
pub use gemini::{
    DEFAULT_GEMINI_MAX_RETRIES, DEFAULT_GEMINI_MODEL, DEFAULT_GEMINI_TIMEOUT_SECONDS, GeminiModel,
    MAX_GEMINI_RETRIES,
};
#[cfg(feature = "openai-compatible")]
pub use groq::{
    DEFAULT_GROQ_BASE_URL, DEFAULT_GROQ_MAX_RETRIES, DEFAULT_GROQ_MODEL,
    DEFAULT_GROQ_TIMEOUT_SECONDS, GroqModel, MAX_GROQ_RETRIES,
};
#[cfg(feature = "openai-compatible")]
pub use hugging_face::{
    DEFAULT_HUGGING_FACE_BASE_URL, DEFAULT_HUGGING_FACE_MAX_RETRIES, DEFAULT_HUGGING_FACE_MODEL,
    DEFAULT_HUGGING_FACE_TIMEOUT_SECONDS, HuggingFaceModel, MAX_HUGGING_FACE_RETRIES,
};
#[cfg(feature = "openai-compatible")]
pub use nvidia::{
    DEFAULT_NVIDIA_BASE_URL, DEFAULT_NVIDIA_MAX_RETRIES, DEFAULT_NVIDIA_MODEL,
    DEFAULT_NVIDIA_TIMEOUT_SECONDS, MAX_NVIDIA_RETRIES, NvidiaModel,
};
#[cfg(feature = "openai-compatible")]
pub use openai_compatible::{
    DEFAULT_OPENAI_COMPATIBLE_BASE_URL, DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES,
    DEFAULT_OPENAI_COMPATIBLE_MODEL, DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS,
    MAX_OPENAI_COMPATIBLE_RETRIES, OpenAiCompatibleModel,
};
#[cfg(feature = "openai-compatible")]
pub use openrouter::{
    DEFAULT_OPENROUTER_BASE_URL, DEFAULT_OPENROUTER_MAX_RETRIES, DEFAULT_OPENROUTER_MODEL,
    DEFAULT_OPENROUTER_TIMEOUT_SECONDS, MAX_OPENROUTER_RETRIES, OpenRouterModel,
};
