# Architecture

Vogon Runtime is split into three crates:

- `vogon-core`: provider-neutral workflow, runtime, replay, hashing, and error types.
- `vogon-adapters`: model adapters that implement the core runtime boundary.
- `vogon-cli`: command-line entrypoint for demos, workflow runs, verification, and traces.

The core crate must not depend on provider SDKs, environment variables, or CLI
parsing. That boundary keeps deterministic replay verification testable without
network access.
