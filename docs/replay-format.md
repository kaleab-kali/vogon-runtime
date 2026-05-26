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
