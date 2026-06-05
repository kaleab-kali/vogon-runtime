# CLI Reference

The `vogon` CLI runs deterministic workflows, writes replay files, verifies
saved replays, and prints replay traces.

Run commands from the workspace with:

```sh
cargo run -p vogon-cli -- <command>
```

After installing the binary, use `vogon <command>` directly.

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
trailing whitespace is rejected.
When redaction values overlap, Vogon applies the longest values first.

## `vogon verify`

Runs a workflow and compares the result with a saved replay.

```sh
cargo run -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
```

For redacted replays, pass the same redaction rules used when the replay was
created:

```sh
cargo run -p vogon-cli -- verify --redact api_key=sk-test-123 fixtures/workflows/support-triage.toml target/support-triage.replay.json
```

If a replay contains redaction markers, `vogon verify` rejects it before
execution unless each marker label has a matching `--redact` rule. If
verification still mismatches, actual step output values are masked in the
structured mismatch JSON for redacted replays.

Malformed redaction markers are also rejected before workflow execution.

Successful verification exits with status `0`. Mismatches are printed as
structured JSON and the command exits with a non-zero status.

Emit the verification report as JSON for both matches and mismatches. The JSON
report includes `workflow_name`, `is_match`, and `mismatches` fields:

```sh
cargo run -p vogon-cli -- verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
```

## `vogon trace`

Prints a human-readable replay trace.

```sh
cargo run -p vogon-cli -- trace fixtures/replays/support-triage.replay.json
```

Emit newline-delimited JSON for tools and logs:

```sh
cargo run -p vogon-cli -- trace --jsonl fixtures/replays/support-triage.replay.json
```

Redact a known literal value from trace output:

```sh
cargo run -p vogon-cli -- trace --redact api_key=sk-test-123 fixtures/replays/support-triage.replay.json
```

Malformed redaction markers are rejected before trace output is printed.
