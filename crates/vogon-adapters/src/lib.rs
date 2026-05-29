//! Model adapters for Vogon Runtime.
//!
//! `vogon-adapters` contains [`ModelAdapter`](vogon_core::ModelAdapter)
//! implementations that can be plugged into `vogon-core`.
//!
//! The current crate exposes [`DeterministicEchoModel`], a deterministic adapter
//! intended for local development, tests, examples, and replay fixtures.
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

#![deny(rustdoc::broken_intra_doc_links)]

mod fake;

pub use fake::DeterministicEchoModel;
