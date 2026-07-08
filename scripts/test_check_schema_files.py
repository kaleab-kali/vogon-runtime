import tempfile
import unittest
from pathlib import Path

from scripts import check_schema_files


class CheckSchemaFilesTests(unittest.TestCase):
    def test_accepts_expected_schema_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_schema_files(root)

            self.assertEqual(check_schema_files.check_repository(root), [])

    def test_reports_missing_schema_files(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(
                check_schema_files.check_repository(Path(directory)),
                [
                    "schemas/workflow.schema.json: missing schema file",
                    "schemas/replay.schema.json: missing schema file",
                ],
            )

    def test_reports_invalid_schema_json(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_schema_files(root)
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

            self.assertEqual(
                check_schema_files.check_repository(root),
                [
                    "schemas/workflow.schema.json: root additionalProperties must be false"
                ],
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


if __name__ == "__main__":
    unittest.main()
