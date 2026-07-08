import tempfile
import unittest
from pathlib import Path

from scripts import check_schema_files


class CheckSchemaFilesTests(unittest.TestCase):
    def test_accepts_expected_schema_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_schema_files(root)
            write_fixture_files(root)

            self.assertEqual(check_schema_files.check_repository(root), [])

    def test_reports_missing_schema_files(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(
                check_schema_files.check_repository(Path(directory)),
                [
                    "schemas/workflow.schema.json: missing schema file",
                    "schemas/replay.schema.json: missing schema file",
                    "fixtures/workflows: missing workflow fixtures",
                    "fixtures/replays: missing replay fixtures",
                ],
            )

    def test_reports_invalid_schema_json(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_schema_files(root)
            write_fixture_files(root)
            (root / "schemas" / "workflow.schema.json").write_text(
                "{",
                encoding="utf-8",
            )

            self.assertEqual(
                check_schema_files.check_repository(root),
                ["schemas/workflow.schema.json: invalid JSON: Expecting property name enclosed in double quotes"],
            )

    def test_reports_weakened_root_strictness(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_schema_files(
                root,
                workflow_schema=workflow_schema_text().replace(
                    '"additionalProperties": false',
                    '"additionalProperties": true',
                    1,
                ),
            )
            write_fixture_files(root)

            self.assertEqual(
                check_schema_files.check_repository(root),
                [
                    "schemas/workflow.schema.json: root additionalProperties must be false"
                ],
            )

    def test_reports_workflow_fixture_outside_schema_shape(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_schema_files(root)
            write_fixture_files(root, workflow_text='name = "support triage"\n')

            self.assertEqual(
                check_schema_files.check_repository(root),
                [
                    "fixtures/workflows/support-triage.toml: workflow name must use ASCII letters, digits, underscores, and hyphens",
                    "fixtures/workflows/support-triage.toml: workflow steps must be a non-empty list",
                ],
            )

    def test_reports_replay_fixture_outside_schema_shape(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_schema_files(root)
            write_fixture_files(
                root,
                replay_text=replay_fixture_text().replace('"schema_version": 1', '"schema_version": 0'),
            )

            self.assertEqual(
                check_schema_files.check_repository(root),
                ["fixtures/replays/support-triage.replay.json: replay schema_version must be 1"],
            )


def write_schema_files(
    root: Path,
    workflow_schema: str | None = None,
    replay_schema: str | None = None,
) -> None:
    schemas = root / "schemas"
    schemas.mkdir()
    (schemas / "workflow.schema.json").write_text(
        workflow_schema or workflow_schema_text(),
        encoding="utf-8",
    )
    (schemas / "replay.schema.json").write_text(
        replay_schema or replay_schema_text(),
        encoding="utf-8",
    )


def write_fixture_files(
    root: Path,
    workflow_text: str | None = None,
    replay_text: str | None = None,
) -> None:
    workflows = root / "fixtures" / "workflows"
    replays = root / "fixtures" / "replays"
    workflows.mkdir(parents=True)
    replays.mkdir(parents=True)
    (workflows / "support-triage.toml").write_text(
        workflow_text or workflow_fixture_text(),
        encoding="utf-8",
    )
    (replays / "support-triage.replay.json").write_text(
        replay_text or replay_fixture_text(),
        encoding="utf-8",
    )


def workflow_schema_text() -> str:
    return """{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Vogon Workflow",
  "type": "object",
  "additionalProperties": false,
  "required": ["name", "steps"],
  "properties": {
    "name": {},
    "steps": {}
  }
}
"""


def workflow_fixture_text() -> str:
    return """name = "support-triage"

[[steps]]
id = "classify"
prompt = "Classify this support request."
"""


def replay_schema_text() -> str:
    return """{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Vogon Replay",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema_version", "workflow_name", "runtime", "run_hash", "steps"],
  "properties": {
    "schema_version": {},
    "workflow_name": {},
    "runtime": {},
    "run_hash": {},
    "steps": {}
  }
}
"""


def replay_fixture_text() -> str:
    return """{
  "schema_version": 1,
  "workflow_name": "support-triage",
  "runtime": {
    "provider": "deterministic",
    "adapter": "deterministic-echo",
    "adapter_version": "0.1.0",
    "model": "deterministic-echo",
    "cache_identity": "vogon-adapters@0.1.0:deterministic-echo:v1",
    "parameters": {
      "mode": "offline"
    }
  },
  "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "steps": [
    {
      "step_id": "classify",
      "input_hash": "1111111111111111111111111111111111111111111111111111111111111111",
      "output_hash": "2222222222222222222222222222222222222222222222222222222222222222",
      "output": "done"
    }
  ]
}
"""


if __name__ == "__main__":
    unittest.main()
