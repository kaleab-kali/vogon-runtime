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

Do not commit API keys or real replay outputs containing private prompts or
customer data. Use redaction rules when writing replays from real provider
runs.

### Live Provider Smoke Testing

The `Live Gemini Smoke` GitHub Actions workflow runs a real Gemini-backed
workflow only when `GEMINI_API_KEY` is available as a repository or environment
secret. It is manual by default so pull request CI stays deterministic and does
not depend on provider availability, account quota, or network behavior.

Run it from GitHub Actions after changing Gemini adapter behavior, provider
configuration, release packaging, or deployment settings that affect real
provider calls. The workflow writes its replay to `target/`, checks the replay
shape, and does not upload the replay as an artifact.

Official references:

- Gemini API pricing: <https://ai.google.dev/gemini-api/docs/pricing>
- Gemini text generation API: <https://ai.google.dev/gemini-api/docs/text-generation>

## Candidate Providers

These providers are worth evaluating for future adapters. Availability, limits,
model names, and free tiers change over time, so verify current terms before
implementing or recommending one.

| Provider | Why consider it | Notes |
| --- | --- | --- |
| Hugging Face Inference Providers | Broad model catalog and documented free credits for experimentation. | Good candidate for open-model workflows; provider and model routing need explicit configuration. |
| GroqCloud | Fast hosted inference and documented developer rate limits. | Good candidate for low-latency text workflows; free-tier availability should be checked before adding a default. |
| OpenRouter | OpenAI-compatible routing across many model providers, including some free models. | Good candidate for a generic OpenAI-compatible adapter; model availability and free labels are provider-dependent. |

Official references:

- Hugging Face Inference Providers: <https://huggingface.co/docs/inference-providers>
- Hugging Face pricing: <https://huggingface.co/pricing>
- GroqCloud docs: <https://console.groq.com/docs>
- GroqCloud rate limits: <https://console.groq.com/docs/rate-limits>
- OpenRouter docs: <https://openrouter.ai/docs>

## Adapter Requirements

New provider adapters should:

- Keep `vogon-core` provider-neutral.
- Keep deterministic execution as the default test and fixture path.
- Avoid printing or logging API keys.
- Return provider failures as `VogonError::Adapter` with actionable context.
- Bound network calls with explicit timeouts.
- Cap provider error output so large HTTP responses stay readable.
- Include unit tests that do not require network access or credentials.
- Add an explicit CLI opt-in instead of changing deterministic defaults.
- Document required environment variables, model selection, and replay redaction
  guidance.
