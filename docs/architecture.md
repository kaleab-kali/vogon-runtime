# Architecture

Vogon Runtime is split into three crates:

- `vogon-core`: provider-neutral workflow, runtime, replay, hashing, and error types.
- `vogon-adapters`: model adapters that implement the core runtime boundary.
- `vogon-cli`: command-line entrypoint for demos, workflow runs, verification, and traces.

The core crate must not depend on provider SDKs, environment variables, or CLI
parsing. That boundary keeps deterministic replay verification testable without
network access.

Runtime execution can emit `RuntimeEvent` values through observer callbacks.
This keeps observability provider-neutral: callers can log, count, or export
events without coupling `vogon-core` to a tracing backend.

Runtime calls can also use an optional `RunCache` keyed by stable step input
hashes. The cache stores raw adapter outputs, so callers can apply different
redaction rules to cached outputs without changing cache keys.
