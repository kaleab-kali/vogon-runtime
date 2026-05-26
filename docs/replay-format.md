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

When redaction is applied, the replay stores the redacted output and hashes the
redacted output. Verification must use the same redaction rules to compare
against a redacted replay.
