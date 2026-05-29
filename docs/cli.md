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

Successful verification exits with status `0`. Mismatches are printed as
structured JSON and the command exits with a non-zero status.

## `vogon trace`

Prints a human-readable replay trace.

```sh
cargo run -p vogon-cli -- trace fixtures/replays/support-triage.replay.json
```

Emit newline-delimited JSON for tools and logs:

```sh
cargo run -p vogon-cli -- trace --jsonl fixtures/replays/support-triage.replay.json
```
