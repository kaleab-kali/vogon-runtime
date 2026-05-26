# Determinism

LLM workflows often fail because prompts, provider behavior, retries, and hidden
state change over time. Vogon Runtime treats each workflow run as an artifact:
the inputs, outputs, and hashes are recorded so a future run can be compared
against the original replay log.

The MVP starts with a deterministic fake model. Real provider adapters can be
added later without weakening the runtime boundary.
