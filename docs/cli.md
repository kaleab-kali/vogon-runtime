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

## `vogon init`

Creates a starter TOML workflow file that can be checked and run immediately
with the deterministic adapter.

```sh
cargo run -p vogon-cli -- init --output workflow.toml
cargo run -p vogon-cli -- check workflow.toml
cargo run -p vogon-cli -- run workflow.toml
```

By default, `vogon init` refuses to overwrite an existing file. Pass `--force`
when intentionally replacing the output path:

```sh
cargo run -p vogon-cli -- init --output workflow.toml --force
```

## `vogon providers`

Shows available model providers, whether provider support is enabled in the
current binary, whether required credential environment variables are
configured, default endpoint/model metadata, provider documentation links, and
public usage or rate-limit links. Secret values are never printed.

```sh
cargo run -p vogon-cli -- providers
```

Emit the provider diagnostics as JSON for scripts:

```sh
cargo run -p vogon-cli -- providers --json
```

The JSON output includes `documentation_url` and `usage_url` fields so operator
tooling can point users to setup, pricing, credit, and rate-limit information
without embedding secret values. `usage_url` is `null` for provider-neutral
entries that do not have a single external pricing or limits page.

## `vogon doctor`

Runs local installation diagnostics without making network calls. The command
executes a deterministic one-step workflow self-check and reports provider
credential status, default endpoint/model metadata, provider documentation
links, and public usage or rate-limit links without printing secret values.

```sh
cargo run -p vogon-cli -- doctor
```

Emit the diagnostics as JSON for scripts:

```sh
cargo run -p vogon-cli -- doctor --json
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

Workflows can reference named `{{input.NAME}}` placeholders. Supply values
with repeatable literal or UTF-8 file options:

```sh
cargo run -p vogon-cli -- run --input service=payments --input-file policy=release-policy.md workflow.toml
```

Inject tracked staged and unstaged changes from a Git working tree as the
reserved `git_diff` input:

```sh
cargo run -p vogon-cli -- run --git-diff fixtures/workflows/git-change-review.toml
```

For pull request CI, inject committed changes relative to a base revision:

```sh
cargo run -p vogon-cli -- run --git-diff-base origin/main fixtures/workflows/git-change-review.toml
```

Use `--repository DIRECTORY` to select another working tree. Git context
excludes untracked files, external diff drivers, text conversion filters, and
submodule content. Empty Git diffs are rejected. Combined input values and Git
diffs are bounded at 1 MiB.

Workflows with a `[decision]` policy can act as a CI gate. Pass
`--enforce-decision` to exit unsuccessfully when the selected final-step value
is denied:

```sh
cargo run -p vogon-cli -- run --provider nvidia --git-diff-base origin/main --enforce-decision --output target/release-gate.replay.json fixtures/workflows/release-gate.toml
```

The replay is written before a valid denied decision causes the command to
fail, preserving the evidence for CI artifacts. Invalid JSON, Markdown fences,
missing or non-string selected fields, and values absent from both policy lists
fail closed. `--enforce-decision` is rejected before provider execution when
the workflow has no decision policy.

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

For a local endpoint that intentionally does not require authentication, pass
`--openai-compatible-no-auth`. For example, Ollama exposes an
OpenAI-compatible API on its local listener:

```sh
cargo run -p vogon-cli -- run --provider openai-compatible --openai-compatible-base-url http://localhost:11434/v1 --openai-compatible-model llama3.2 --openai-compatible-no-auth fixtures/workflows/support-triage.toml
```

The flag is also available on `vogon verify`. Without it, the CLI continues to
require `OPENAI_COMPATIBLE_API_KEY` and sends bearer authentication. Hosted
endpoints must use HTTPS. Plain HTTP is accepted only for loopback hosts so
local development cannot silently enable plaintext remote credential or prompt
transport.

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

Run with Hugging Face Inference Providers by setting `HF_TOKEN` and selecting
the Hugging Face provider:

```sh
HF_TOKEN=... cargo run -p vogon-cli -- run --provider hugging-face fixtures/workflows/support-triage.toml
```

The default model is `openai/gpt-oss-120b:fastest`, routed through
`https://router.huggingface.co/v1`. Use `--hugging-face-model`,
`--hugging-face-timeout-seconds`, and `--hugging-face-max-retries` to adjust the
model and network bounds.

Run with NVIDIA API Catalog by setting `NVIDIA_API_KEY` and selecting the
NVIDIA provider:

```sh
NVIDIA_API_KEY=... cargo run -p vogon-cli -- run --provider nvidia fixtures/workflows/support-triage.toml
```

The default NVIDIA base URL is `https://integrate.api.nvidia.com/v1`, and the
default model is `meta/llama-3.1-8b-instruct`. Use `--nvidia-model`,
`--nvidia-timeout-seconds`, and `--nvidia-max-retries` to select a current
catalog model and adjust the network bounds.

Run with OpenRouter by setting `OPENROUTER_API_KEY` and selecting the
OpenRouter provider:

```sh
OPENROUTER_API_KEY=... cargo run -p vogon-cli -- run --provider openrouter fixtures/workflows/support-triage.toml
```

The default OpenRouter base URL is `https://openrouter.ai/api/v1`, and the
default model is `openrouter/free`. Use `--openrouter-model`,
`--openrouter-timeout-seconds`, and `--openrouter-max-retries` to adjust the
model and network bounds.

Write the replay JSON to a file:

```sh
cargo run -p vogon-cli -- run --output target/support-triage.replay.json fixtures/workflows/support-triage.toml
```

Persist a bounded run cache across repeated runs:

```sh
cargo run -p vogon-cli -- run --cache-file target/vogon.cache.json fixtures/workflows/support-triage.toml
```

Use `--cache-max-entries` to tune the retained output count. The default is
`1024`; `0` disables storage while still allowing the cache file to be
rewritten as an empty cache.

Cache files are performance artifacts, not public replay files. They may
contain raw provider outputs, including values that `--redact` removes from the
replay JSON. Store cache files only in private, trusted locations and do not
commit them.

Redact a known literal value from replay outputs:

```sh
cargo run -p vogon-cli -- run --redact api_key=sk-test-123 fixtures/workflows/support-triage.toml
```

Redaction values use `LABEL=VALUE` syntax and may be repeated.

Use `--redact-env LABEL=ENV_VAR` for credentials or other environment-only
values. Vogon resolves the variable internally, so the value is not expanded
into the process argument list:

```sh
cargo run -p vogon-cli -- run --redact-env api_key=PROVIDER_API_KEY fixtures/workflows/support-triage.toml
```

The environment variable must be set, contain Unicode text, and resolve to a
non-empty value. Labels must remain unique across `--redact` and
`--redact-env`.

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

Use the same private cache file for the original run and exact verification to
avoid a second provider request:

```sh
cargo run -p vogon-cli -- run --provider nvidia --cache-file target/review.cache.json --output target/review.replay.json workflow.toml
cargo run -p vogon-cli -- verify --cache-file target/review.cache.json workflow.toml target/review.replay.json
```

`--cache-max-entries` applies the same bounded retention policy as `run`.
Entries are scoped by adapter identity and step input hash. Cache misses execute
the configured provider normally, so cached verification is reproducible only
when the cache from the original run is retained. The replay and cache paths
must differ.

`--mode exact` is the default and compares all replay hashes, metadata, and
outputs. Use `--mode structure` for a live smoke check of a nondeterministic
provider:

```sh
cargo run -p vogon-cli -- verify --mode structure workflow.toml target/review.replay.json
```

Structural mode executes the workflow and compares its name, provider metadata,
step count, ordered step IDs, and each rendered prompt hash. It deliberately
ignores run hashes, assembled input hashes, output hashes, and output text.
Passing structural verification means the same rendered workflow completed; it
does not mean the model produced a correct or policy-compliant answer. A replay
without prompt hashes is rejected before provider selection, so regenerate
older replays with a current `vogon run` first.

When a workflow declares inputs, verification requires the same `--input`,
`--input-file`, `--git-diff`, or `--git-diff-base` context used to create the
replay. Changed context produces changed step input hashes and a structured
verification mismatch.

For redacted replays, pass the same redaction rules used when the replay was
created:

```sh
cargo run -p vogon-cli -- verify --redact api_key=sk-test-123 fixtures/workflows/support-triage.toml target/support-triage.replay.json
```

If a replay contains redaction markers, `vogon verify` rejects it before
execution unless each marker label has a matching `--redact` or `--redact-env`
rule. If verification still mismatches after redaction rules are provided,
expected and actual step output values are redacted before human-readable or
JSON mismatch reports are printed. Redacted replay markers also cause step
output mismatch values to be replaced with an unreported placeholder.

Malformed redaction markers are also rejected before workflow execution.

Successful verification exits with status `0`. Mismatches are printed as
structured JSON and the command exits with a non-zero status.

Emit the verification report as JSON for both matches and mismatches. The JSON
report includes `workflow_name`, `mode`, `is_match`, and `mismatches` fields:

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
