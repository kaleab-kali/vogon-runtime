# Replay Format

The first replay format is JSON.

```json
{
  "workflow_name": "support-triage",
  "run_hash": "sha256-hex",
  "steps": [
    {
      "step_id": "classify",
      "input_hash": "sha256-hex",
      "output_hash": "sha256-hex",
      "output": "billing"
    }
  ]
}
```

The schema will stay small until the runtime has stable verification semantics.

## Redaction

Replay output can be redacted before it is written. Redaction rules are literal
string replacements, which keeps behavior deterministic and avoids provider- or
regex-specific behavior in `vogon-core`.
When redaction literals overlap, Vogon applies the longest literals first so a
shorter prefix does not partially expose a longer sensitive value.

When redaction is applied, the replay stores the redacted output and hashes the
redacted output. Verification must use the same redaction rules to compare
against a redacted replay.

Redacted outputs use `[REDACTED:<label>]` markers. When a replay contains these
markers, `vogon verify` requires matching `--redact <label>=<literal>` rules
before it executes the workflow. If verification fails after redaction rules are
provided, mismatch JSON masks actual step output values so a bad redaction
literal does not print the original output.

Malformed redaction markers, such as markers without a closing `]` or without a
label, are rejected before verification executes the workflow.
