# Architecture

Vogon Runtime is split into three crates:

- `vogon-core`: provider-neutral workflow, runtime, replay, hashing, and error types.
- `vogon-adapters`: model adapters that implement the core runtime boundary.
- `vogon-cli`: command-line entrypoint for demos, workflow runs, verification, and traces.

The core crate must not depend on provider SDKs, environment variables, or CLI
parsing. That boundary keeps deterministic replay verification testable without
network access.

Provider-backed adapters, such as the Gemini adapter, live in `vogon-adapters`
behind explicit CLI selection so `vogon-core` stays provider-neutral and
deterministic fixtures do not require network access.

The OpenAI-compatible adapter is also isolated in `vogon-adapters`. It targets
providers that expose `/chat/completions`, including Hugging Face Inference
Providers and OpenRouter-style routers, while preserving deterministic local
execution as the default CLI path.

Runtime execution can emit `RuntimeEvent` values through observer callbacks.
This keeps observability provider-neutral: callers can log, count, or export
events without coupling `vogon-core` to a tracing backend.

Runtime calls can also use an optional `RunCache` keyed by stable step input
hashes. The cache stores raw adapter outputs, so callers can apply different
redaction rules to cached outputs without changing cache keys. `RunCache` is
bounded by entry count, supports explicit removal and clearing, and uses a
default limit of 1024 outputs for long-lived callers.
