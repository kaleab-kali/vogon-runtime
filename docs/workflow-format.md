# Workflow Format

Vogon CLI workflow files are TOML documents with a workflow name and an ordered
list of steps.

```toml
name = "support-triage"

[[steps]]
id = "classify"
prompt = "Classify this support request as billing, bug, or general."

[[steps]]
id = "draft_response"
prompt = "Draft a concise customer response based on the classification."
```

## Fields

### `name`

The workflow name is required and must not be empty. Leading or trailing
whitespace is rejected. Replay verification compares the workflow name in the
replay file against the workflow being run.

### `steps`

Each `[[steps]]` entry is executed in order. A workflow must contain at least
one step.

Each step has:

- `id`: a unique step identifier.
- `prompt`: the non-empty prompt text for that step.

Step identifiers may contain ASCII letters, ASCII digits, underscores, and
hyphens. Spaces and other punctuation are rejected.

Unknown top-level workflow fields and unknown step fields are rejected. This
keeps workflow files explicit and catches misspelled keys before execution.

## Step Inputs

The first step receives its prompt as input. Each later step receives its prompt
plus the previous step output:

```text
{prompt}

Previous output:
{previous output}
```

This makes ordered execution explicit and keeps replay hashes stable.

## Validation

Validate a workflow without executing it:

```sh
cargo run -p vogon-cli -- check fixtures/workflows/support-triage.toml
```

Run a workflow and write a replay:

```sh
cargo run -p vogon-cli -- run --output target/support-triage.replay.json fixtures/workflows/support-triage.toml
```
