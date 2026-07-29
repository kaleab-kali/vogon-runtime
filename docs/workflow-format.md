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

Generate a starter workflow with:

```sh
cargo run -p vogon-cli -- init --output workflow.toml
```

## Fields

### `name`

The workflow name is required and must not be empty. It may contain ASCII
letters, ASCII digits, underscores, and hyphens. Leading or trailing whitespace,
spaces, and other punctuation are rejected. Replay verification compares the
workflow name in the replay file against the workflow being run.

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

## Workflow Inputs

Prompts can declare strict named inputs with `{{input.NAME}}` placeholders:

```toml
name = "git-change-review"

[[steps]]
id = "risk_analysis"
prompt = """
Review this Git diff:

{{input.git_diff}}
"""
```

Input names may contain ASCII letters, digits, underscores, and hyphens.
`vogon check` validates placeholder syntax and reports the sorted required
input names without requiring values.

Supply literal or UTF-8 file values when running or verifying:

```sh
vogon run --input service=payments --input-file policy=release-policy.md workflow.toml
```

All declared inputs are required. Unknown, malformed, duplicate, missing, and
unused inputs fail before provider execution. Input replacement is a single
pass, so placeholder-like text inside a file or Git diff remains literal.
Combined input values are limited to 1 MiB.

Use the reserved `git_diff` input for repository reviews:

```sh
# Staged and unstaged changes to tracked files relative to HEAD.
vogon run --git-diff fixtures/workflows/git-change-review.toml

# Committed pull request changes relative to the merge base with origin/main.
vogon run --git-diff-base origin/main fixtures/workflows/git-change-review.toml
```

Use `--repository DIRECTORY` when the Git working tree is not the current
directory. Git context excludes untracked files, external diff drivers, text
conversion filters, and submodule content. Empty diffs and diffs larger than
1 MiB are rejected.

## Validation

Validate a workflow without executing it:

```sh
cargo run -p vogon-cli -- check fixtures/workflows/support-triage.toml
```

Editor and review tooling can use the published
[workflow schema](../schemas/workflow.schema.json) for the current workflow
shape. The CLI remains the source of truth for validation, including unique
step identifiers.

Run a workflow and write a replay:

```sh
cargo run -p vogon-cli -- run --output target/support-triage.replay.json fixtures/workflows/support-triage.toml
```
