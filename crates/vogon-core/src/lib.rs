//! Core deterministic workflow runtime primitives.
//!
//! `vogon-core` contains the provider-neutral pieces of Vogon Runtime:
//! workflows, steps, model adapter traits, replay reports, verification, stable
//! hashing, redaction, runtime events, and step output caching.
//!
//! The crate does not depend on provider SDKs or CLI parsing. Callers provide a
//! [`ModelAdapter`] implementation and the runtime records deterministic
//! [`RunReport`] values that can later be verified.
//!
//! ```
//! use vogon_core::{ModelAdapter, Result, Runtime, Step, StepId, Workflow};
//!
//! #[derive(Debug, Clone)]
//! struct EchoModel;
//!
//! impl ModelAdapter for EchoModel {
//!     fn complete(&self, step: &Step, input: &str) -> Result<String> {
//!         Ok(format!("{}:{input}", step.id().as_str()))
//!     }
//! }
//!
//! let workflow = Workflow::new(
//!     "example",
//!     vec![Step::new(StepId::new("draft").unwrap(), "Draft a short note.")],
//! )
//! .unwrap();
//!
//! let report = Runtime::new(EchoModel).run(&workflow).unwrap();
//! assert_eq!(report.workflow_name, "example");
//! assert_eq!(report.steps.len(), 1);
//! ```

#![forbid(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]

mod cache;
mod decision;
mod error;
mod events;
mod hash;
mod redaction;
mod replay;
mod runtime;
mod step;
mod workflow;

pub use cache::{DEFAULT_RUN_CACHE_MAX_ENTRIES, RunCache};
pub use decision::{DecisionOutcome, DecisionPolicy, DecisionResult};
pub use error::{Result, VogonError};
pub use events::RuntimeEvent;
pub use hash::stable_hash;
pub use redaction::{RedactionRule, RedactionSet};
pub use replay::{
    CURRENT_REPLAY_SCHEMA_VERSION, LEGACY_REPLAY_SCHEMA_VERSION, ReplayMismatch, RunReport,
    RuntimeMetadata, StepResult, VerificationMode, VerificationReport,
};
pub use runtime::{ModelAdapter, Runtime};
pub use step::{Step, StepId};
pub use workflow::Workflow;
