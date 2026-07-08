# Schemas

Vogon Runtime publishes JSON Schema files for editor integration and fixture
review:

- [Workflow schema](../schemas/workflow.schema.json)
- [Replay schema](../schemas/replay.schema.json)

The workflow schema describes the TOML-compatible object shape accepted by
`vogon check`. The replay schema describes the current replay JSON format with
`schema_version` set to `1`.

These schemas are aids for contributors and tooling. The CLI validators remain
the source of truth because they also enforce cross-field rules such as unique
step identifiers and replay step ordering.
