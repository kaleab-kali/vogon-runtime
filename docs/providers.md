# Provider Adapters

Vogon keeps provider integrations outside `vogon-core`. The runtime only
depends on the `ModelAdapter` trait, which lets tests and replay fixtures use a
deterministic adapter while real workflow runs can opt into network-backed
providers.

## Built-In Providers

### Deterministic

The deterministic adapter is the default provider for `vogon run`.

```sh
cargo run -p vogon-cli -- run fixtures/workflows/support-triage.toml
```

Use this provider for tests, fixtures, examples, CI, and replay verification
work. It requires no credentials or network access.

Use `vogon providers` to check which providers are enabled in the current
binary and whether required credential environment variables are configured.
The command reports only boolean credential status and never prints secret
values.

### Gemini

The Gemini adapter uses the Gemini API `generateContent` REST endpoint. Default
CLI builds include the adapter, but it is only used when explicitly selected:

```sh
GEMINI_API_KEY=... cargo run -p vogon-cli -- run --provider gemini fixtures/workflows/support-triage.toml
```

The default model is `gemini-3.1-flash-lite`, chosen as the first real provider
path because Google's Gemini API pricing page lists free-tier usage and the
REST API is simple enough to keep the adapter small.

Use `--gemini-model` to select another Gemini model:

```sh
GEMINI_API_KEY=... cargo run -p vogon-cli -- run --provider gemini --gemini-model gemini-3.1-flash-lite fixtures/workflows/support-triage.toml
```

Gemini requests use a 30 second timeout by default. Use a nonzero
`--gemini-timeout-seconds` value to adjust the bound for slower or stricter
environments:

```sh
GEMINI_API_KEY=... cargo run -p vogon-cli -- run --provider gemini --gemini-timeout-seconds 60 fixtures/workflows/support-triage.toml
```

Transient transport failures and retryable HTTP responses are retried twice by
default. Retryable HTTP responses are status codes `408`, `409`, `425`, `429`,
and `5xx`. Retry counts must be between `0` and `20`; use
`--gemini-max-retries 0` to disable retries. Retries use exponential backoff
with lightweight jitter:

```sh
GEMINI_API_KEY=... cargo run -p vogon-cli -- run --provider gemini --gemini-max-retries 0 fixtures/workflows/support-triage.toml
```

Do not commit API keys or real replay outputs containing private prompts or
customer data. Use redaction rules when writing replays from real provider
runs.

### OpenAI-Compatible

The OpenAI-compatible adapter uses the `/chat/completions` request and response
shape supported by providers such as Hugging Face Inference Providers and
OpenRouter. Default CLI builds include the adapter, but it is only used when
explicitly selected:

```sh
OPENAI_COMPATIBLE_API_KEY=... cargo run -p vogon-cli -- run --provider openai-compatible fixtures/workflows/support-triage.toml
```

The default base URL is `https://router.huggingface.co/v1`, and the default
model is `openai/gpt-oss-120b:fastest`, matching Hugging Face's documented
OpenAI-compatible chat-completions router. Use `--openai-compatible-base-url`
and `--openai-compatible-model` to target OpenRouter or another compatible
service:

```sh
OPENAI_COMPATIBLE_API_KEY=... cargo run -p vogon-cli -- run --provider openai-compatible --openai-compatible-base-url https://openrouter.ai/api/v1 --openai-compatible-model openai/gpt-5.2 fixtures/workflows/support-triage.toml
```

For Hugging Face, set `OPENAI_COMPATIBLE_API_KEY` to a token with permission to
make Inference Providers calls. For OpenRouter, set it to an OpenRouter API key.
Do not commit real provider replays containing private prompts or outputs.

OpenAI-compatible requests use a 30 second timeout and retry retryable
transport/HTTP failures twice by default. Use
`--openai-compatible-timeout-seconds` and
`--openai-compatible-max-retries` to adjust those bounds. Retry counts must be
between `0` and `20`, and retry delays use exponential backoff with lightweight
jitter.

### OpenRouter

The OpenRouter adapter is a preset for OpenRouter's OpenAI-compatible
chat-completions endpoint. Default CLI builds include the adapter, but it is
only used when explicitly selected:

```sh
OPENROUTER_API_KEY=... cargo run -p vogon-cli -- run --provider openrouter fixtures/workflows/support-triage.toml
```

The default base URL is `https://openrouter.ai/api/v1`, and the default model
is `openrouter/free`. Use `--openrouter-model` to select another OpenRouter
model:

```sh
OPENROUTER_API_KEY=... cargo run -p vogon-cli -- run --provider openrouter --openrouter-model openrouter/free fixtures/workflows/support-triage.toml
```

OpenRouter requests use a 30 second timeout and retry retryable
transport/HTTP failures twice by default. Use `--openrouter-timeout-seconds`
and `--openrouter-max-retries` to adjust those bounds. Retry counts must be
between `0` and `20`.

### Groq

The Groq adapter is a preset for Groq's OpenAI-compatible chat-completions
endpoint. Default CLI builds include the adapter, but it is only used when
explicitly selected:

```sh
GROQ_API_KEY=... cargo run -p vogon-cli -- run --provider groq fixtures/workflows/support-triage.toml
```

The default base URL is `https://api.groq.com/openai/v1`, and the default model
is `llama-3.1-8b-instant`. Use `--groq-model` to select another Groq model:

```sh
GROQ_API_KEY=... cargo run -p vogon-cli -- run --provider groq --groq-model llama-3.1-8b-instant fixtures/workflows/support-triage.toml
```

Groq requests use a 30 second timeout and retry retryable transport/HTTP
failures twice by default. Use `--groq-timeout-seconds` and
`--groq-max-retries` to adjust those bounds. Retry counts must be between `0`
and `20`.

### Hugging Face

The Hugging Face adapter is a preset for Hugging Face Inference Providers'
OpenAI-compatible endpoint. Default CLI builds include the adapter, but it is
only used when explicitly selected:

```sh
HF_TOKEN=... cargo run -p vogon-cli -- run --provider hugging-face fixtures/workflows/support-triage.toml
```

The default base URL is `https://router.huggingface.co/v1`, and the default
model is `openai/gpt-oss-120b:fastest`. Use `--hugging-face-model` to select
another Hugging Face routed model:

```sh
HF_TOKEN=... cargo run -p vogon-cli -- run --provider hugging-face --hugging-face-model openai/gpt-oss-120b:fastest fixtures/workflows/support-triage.toml
```

Hugging Face requests use a 30 second timeout and retry retryable
transport/HTTP failures twice by default. Use `--hugging-face-timeout-seconds`
and `--hugging-face-max-retries` to adjust those bounds. Retry counts must be
between `0` and `20`.

### Live Provider Smoke Testing

The `Live Gemini Smoke` GitHub Actions workflow runs a real Gemini-backed
workflow only when `GEMINI_API_KEY` is available as a repository or environment
secret. It is manual by default so pull request CI stays deterministic and does
not depend on provider availability, account quota, or network behavior.

The `Live OpenAI-Compatible Smoke` workflow does the same for the
OpenAI-compatible adapter when `OPENAI_COMPATIBLE_API_KEY` is configured. Its
manual dispatch inputs let maintainers override the base URL and model, so the
same smoke can target Hugging Face, OpenRouter, or another compatible endpoint.

The `Live Groq Smoke` workflow does the same for the Groq preset when
`GROQ_API_KEY` is configured. Its manual dispatch input lets maintainers choose
a different Groq model without changing repository code.

The `Live Hugging Face Smoke` workflow does the same for the Hugging Face
preset when `HF_TOKEN` is configured. Its manual dispatch input lets
maintainers choose a different Hugging Face model without changing repository
code.

The `Live OpenRouter Smoke` workflow does the same for the OpenRouter preset
when `OPENROUTER_API_KEY` is configured. Its manual dispatch input lets
maintainers choose a different OpenRouter model without changing repository
code.

Run the relevant live provider workflow from GitHub Actions after changing
adapter behavior, provider configuration, release packaging, or deployment
settings that affect real provider calls. These workflows write replays to
`target/`, check replay shape, assert provider runtime metadata, confirm API
keys are absent from the serialized replay, and do not upload provider outputs
as artifacts.

Official references:

- Gemini API pricing: <https://ai.google.dev/gemini-api/docs/pricing>
- Gemini text generation API: <https://ai.google.dev/gemini-api/docs/text-generation>
- Hugging Face Inference Providers: <https://huggingface.co/docs/inference-providers>
- OpenRouter docs: <https://openrouter.ai/docs>
- OpenRouter API reference: <https://openrouter.ai/docs/api-reference/chat-completion>
- OpenRouter free models: <https://openrouter.ai/docs/use-cases/free-models>
- Groq OpenAI compatibility: <https://console.groq.com/docs/openai>
- Groq models: <https://console.groq.com/docs/models>
- Groq rate limits: <https://console.groq.com/docs/rate-limits>

## Candidate Providers

These providers are worth evaluating for future adapters. Availability, limits,
model names, and free tiers change over time, so verify current terms before
implementing or recommending one.

| Provider | Why consider it | Notes |
| --- | --- | --- |
| Hugging Face Inference Providers | Broad model catalog and documented free credits for experimentation. | Supported through the Hugging Face preset and the OpenAI-compatible adapter. |
| OpenRouter | OpenAI-compatible routing across many model providers, including some free models. | Supported through the OpenRouter preset and the OpenAI-compatible adapter; model availability and free labels are provider-dependent. |

Official references:

- Hugging Face Inference Providers: <https://huggingface.co/docs/inference-providers>
- Hugging Face pricing: <https://huggingface.co/pricing>
- OpenRouter docs: <https://openrouter.ai/docs>

## Adapter Requirements

New provider adapters should:

- Keep `vogon-core` provider-neutral.
- Keep deterministic execution as the default test and fixture path.
- Avoid printing or logging API keys.
- Override `ModelAdapter::cache_identity` with non-secret provider, endpoint,
  model, and behavior-affecting configuration.
- Return provider failures as `VogonError::Adapter` with actionable context.
- Bound network calls with explicit timeouts.
- Keep retries bounded and configurable.
- Cap provider error output so large HTTP responses stay readable.
- Include unit tests that do not require network access or credentials.
- Add an explicit CLI opt-in instead of changing deterministic defaults.
- Document required environment variables, model selection, and replay redaction
  guidance.
