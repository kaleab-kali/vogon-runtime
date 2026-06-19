//! Model adapters for Vogon Runtime.
//!
//! `vogon-adapters` contains [`ModelAdapter`](vogon_core::ModelAdapter)
//! implementations that can be plugged into `vogon-core`.
//!
//! The current crate exposes [`DeterministicEchoModel`], a deterministic adapter
//! intended for local development, tests, examples, and replay fixtures. With
//! the default `gemini` feature, it also exposes [`GeminiModel`] for real
//! Gemini API calls.
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

mod fake;
#[cfg(feature = "gemini")]
mod gemini;

pub use fake::DeterministicEchoModel;
#[cfg(feature = "gemini")]
pub use gemini::{
    DEFAULT_GEMINI_MAX_RETRIES, DEFAULT_GEMINI_MODEL, DEFAULT_GEMINI_TIMEOUT_SECONDS, GeminiModel,
};
