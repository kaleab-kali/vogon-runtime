# CLI Reference

The `vogon` CLI runs deterministic workflows, writes replay files, verifies
saved replays, and prints replay traces.

Run commands from the workspace with:

```sh
cargo run -p vogon-cli -- <command>
```

After installing the binary, use `vogon <command>` directly.

Workflow and replay inputs are rejected when they exceed 1 MiB, so accidental
or adversarially large files fail before the CLI buffers them into memory.

## Global Options

```sh
vogon --help
vogon --version
```

`--help` prints the available commands. `--version` prints the CLI package
version.

## `vogon demo`

Runs the built-in deterministic demo workflow.

```sh
cargo run -p vogon-cli -- demo
```

## `vogon providers`

Shows available model providers, whether provider support is enabled in the
current binary, and whether required credential environment variables are
configured. Secret values are never printed.

```sh
cargo run -p vogon-cli -- providers
```

Emit the provider diagnostics as JSON for scripts:

```sh
cargo run -p vogon-cli -- providers --json
```

## `vogon check`

Validates a TOML workflow without executing it.

```sh
cargo run -p vogon-cli -- check fixtures/workflows/support-triage.toml
```

Use this before committing new workflow fixtures or before writing a replay.

Emit a machine-readable validation summary:

```sh
cargo run -p vogon-cli -- check --json fixtures/workflows/support-triage.toml
```

## `vogon run`

Runs a TOML workflow and prints a replay JSON document to stdout.

```sh
cargo run -p vogon-cli -- run fixtures/workflows/support-triage.toml
```

By default, `vogon run` uses the deterministic adapter so tests and fixture
replays do not require network access. To run against the Gemini API, set
`GEMINI_API_KEY` and select the Gemini provider:

```sh
GEMINI_API_KEY=... cargo run -p vogon-cli -- run --provider gemini fixtures/workflows/support-triage.toml
```

Use `--gemini-model` to override the default Gemini model:

```sh
GEMINI_API_KEY=... cargo run -p vogon-cli -- run --provider gemini --gemini-model gemini-3.1-flash-lite fixtures/workflows/support-triage.toml
```

Gemini requests use a 30 second timeout by default. Use
`--gemini-timeout-seconds` to choose a larger or smaller nonzero timeout:

```sh
GEMINI_API_KEY=... cargo run -p vogon-cli -- run --provider gemini --gemini-timeout-seconds 60 fixtures/workflows/support-triage.toml
```

Transient Gemini transport failures and retryable HTTP responses are retried
twice by default. Use `--gemini-max-retries` to change the retry count. Valid
values are `0` through `20`:

```sh
GEMINI_API_KEY=... cargo run -p vogon-cli -- run --provider gemini --gemini-max-retries 0 fixtures/workflows/support-triage.toml
```

To run against an OpenAI-compatible chat-completions endpoint, set
`OPENAI_COMPATIBLE_API_KEY` and select the OpenAI-compatible provider:

```sh
OPENAI_COMPATIBLE_API_KEY=... cargo run -p vogon-cli -- run --provider openai-compatible fixtures/workflows/support-triage.toml
```

The default base URL is `https://router.huggingface.co/v1`, and the default
model is `openai/gpt-oss-120b:fastest`. Override them for OpenRouter or another
compatible service:

```sh
OPENAI_COMPATIBLE_API_KEY=... cargo run -p vogon-cli -- run --provider openai-compatible --openai-compatible-base-url https://openrouter.ai/api/v1 --openai-compatible-model openai/gpt-5.2 fixtures/workflows/support-triage.toml
```

OpenAI-compatible requests use a 30 second timeout and two retries by default.
Use `--openai-compatible-timeout-seconds` and
`--openai-compatible-max-retries` to adjust those bounds. Retry counts must be
between `0` and `20`.

Run with Groq by setting `GROQ_API_KEY` and selecting the Groq provider:

```sh
GROQ_API_KEY=... cargo run -p vogon-cli -- run --provider groq fixtures/workflows/support-triage.toml
```

The default Groq base URL is `https://api.groq.com/openai/v1`, and the default
model is `llama-3.1-8b-instant`. Use `--groq-model`,
`--groq-timeout-seconds`, and `--groq-max-retries` to adjust the model and
network bounds.

Write the replay JSON to a file:

```sh
cargo run -p vogon-cli -- run --output target/support-triage.replay.json fixtures/workflows/support-triage.toml
```

Redact a known literal value from replay outputs:

```sh
cargo run -p vogon-cli -- run --redact api_key=sk-test-123 fixtures/workflows/support-triage.toml
```

Redaction values use `LABEL=VALUE` syntax and may be repeated.
Labels may contain ASCII letters, ASCII digits, `_`, and `-`; leading or
trailing whitespace is rejected. Labels must be unique within one command.
When redaction values overlap, Vogon applies the longest values first.

## `vogon verify`

Runs a workflow and compares the result with a saved replay.

```sh
cargo run -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
```

By default, `vogon verify` uses the provider metadata recorded in the replay.
For legacy unversioned replays, it falls back to the deterministic provider.
Use `--provider` and provider-specific model, timeout, base URL, and retry flags
to override the replay metadata when intentionally checking another adapter.

For redacted replays, pass the same redaction rules used when the replay was
created:

```sh
cargo run -p vogon-cli -- verify --redact api_key=sk-test-123 fixtures/workflows/support-triage.toml target/support-triage.replay.json
```

If a replay contains redaction markers, `vogon verify` rejects it before
execution unless each marker label has a matching `--redact` rule. If
verification still mismatches after redaction rules are provided, expected and
actual step output values are redacted before human-readable or JSON mismatch
reports are printed. Redacted replay markers also cause step output mismatch
values to be replaced with an unreported placeholder.

Malformed redaction markers are also rejected before workflow execution.

Successful verification exits with status `0`. Mismatches are printed as
structured JSON and the command exits with a non-zero status.

Emit the verification report as JSON for both matches and mismatches. The JSON
report includes `workflow_name`, `is_match`, and `mismatches` fields:

```sh
cargo run -p vogon-cli -- verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
```

## `vogon trace`

Prints a human-readable replay trace, including replay schema and runtime
metadata.

```sh
cargo run -p vogon-cli -- trace fixtures/replays/support-triage.replay.json
```

Emit newline-delimited JSON for tools and logs. The first run event includes
replay schema and runtime metadata:

```sh
cargo run -p vogon-cli -- trace --jsonl fixtures/replays/support-triage.replay.json
```

Redact a known literal value from trace output:

```sh
cargo run -p vogon-cli -- trace --redact api_key=sk-test-123 fixtures/replays/support-triage.replay.json
```

Malformed redaction markers are rejected before trace output is printed.
