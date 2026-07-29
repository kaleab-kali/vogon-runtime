# Replay Format

The replay format is JSON. Current replays include an explicit
`schema_version` and non-secret runtime metadata. Legacy unversioned replay
files can still be read as schema version `0`.

```json
{
  "schema_version": 1,
  "workflow_name": "support-triage",
  "runtime": {
    "provider": "deterministic",
    "adapter": "deterministic-echo",
    "adapter_version": "0.1.4",
    "model": "deterministic-echo",
    "cache_identity": "vogon-adapters@0.1.4:deterministic-echo:v1",
    "parameters": {
      "mode": "offline"
    }
  },
  "run_hash": "sha256-hex",
  "steps": [
    {
      "step_id": "classify",
      "prompt_hash": "sha256-hex",
      "input_hash": "sha256-hex",
      "output_hash": "sha256-hex",
      "output": "billing"
    }
  ]
}
```

The schema will stay small until the runtime has stable verification semantics.
Unknown top-level replay fields, unknown runtime metadata fields, and unknown
step fields are rejected. `schema_version` must be a supported version. Hash
fields must be 64-character lowercase hexadecimal SHA-256 digests.
`workflow_name` uses the same portable identifier rule as workflow files: ASCII
letters, ASCII digits, underscores, and hyphens only.
`steps` must contain at least one step result because Vogon does not produce
empty workflow replays. Step IDs inside a replay must be unique.

New runs include `prompt_hash` for each rendered step prompt. The field is
optional so older schema-version-1 replays remain readable, but structural
verification requires it for every step. Exact verification remains available
for older replays. Prompt hashes include resolved workflow inputs but exclude
previous step output, allowing definition and context drift to be detected
without treating nondeterministic output wording as drift.

Runtime metadata records non-secret provider provenance for auditability. The
metadata includes the provider family, adapter implementation, adapter version,
model when present, cache identity, and provider/runtime parameters such as
base URL, timeout, and retry count. Credentials and private prompt/output data
must not be stored in runtime metadata.

`vogon verify` uses this metadata to select the replay provider by default and
compares current-schema runtime metadata against the actual verification run.
Legacy unversioned replays remain readable and verify with the deterministic
provider unless a provider is selected explicitly.

Exact verification compares the complete recorded run. Structural verification
executes the provider but compares only workflow name, runtime metadata, step
count, ordered step IDs, and prompt hashes. It ignores outputs and therefore
must not be used as evidence that an answer is correct.

Editor and review tooling can use the published
[replay schema](../schemas/replay.schema.json) for current `schema_version: 1`
replay files. Legacy unversioned replays remain supported by the CLI but are not
the schema target.

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
markers, `vogon verify` requires matching `--redact <label>=<literal>` or
`--redact-env <label>=<environment-variable>` rules before it executes the
workflow. If verification fails after redaction rules are provided, mismatch
reports apply those redaction rules to both expected and actual step output
values before printing human-readable or JSON output. If a
replay contains redaction markers, step output mismatch values are replaced with
an unreported placeholder.

Redaction labels may contain ASCII letters, ASCII digits, `_`, and `-`.
Leading or trailing whitespace is rejected instead of normalized so marker
labels remain exact. CLI redaction labels must be unique within one command,
and `vogon-core` redaction sets reject duplicate labels for library callers.
Replay commands reject markers with unsupported labels before verification
executes or trace output is printed. Prefix marker-like text with `\` when it
should be treated as literal replay output instead of redaction metadata.

Complete unescaped redaction markers without a label or with unsupported label
characters are rejected by replay commands before verification executes the
workflow or trace output is printed. An unclosed marker candidate that contains
only valid label characters is also rejected; marker-like prose fragments with
normal text separators are treated as literal output.
