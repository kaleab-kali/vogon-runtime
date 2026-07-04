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

Adapters also provide non-secret runtime metadata for replay reports. This lets
saved replays record provider family, adapter implementation, adapter version,
model, cache identity, and runtime parameters without coupling `vogon-core` to
provider SDKs or credentials.

Runtime execution can emit `RuntimeEvent` values through observer callbacks.
This keeps observability provider-neutral: callers can log, count, or export
events without coupling `vogon-core` to a tracing backend. Runtime events cover
step start/finish, replay mismatches, and cache hit/miss status for calls that
use a `RunCache`.

Runtime calls can also use an optional `RunCache`. Runtime cache keys combine
the adapter cache identity with each stable step input hash, then hash that
material before lookup. Provider-backed adapters include non-secret provider
configuration such as adapter kind, endpoint, and model in their cache identity
so outputs are not reused across incompatible providers or models. The cache
stores raw adapter outputs, so callers can apply different redaction rules to
cached outputs without changing cache keys. `RunCache` is bounded by entry
count, supports explicit removal and clearing, and uses a default limit of 1024
outputs for long-lived callers.
