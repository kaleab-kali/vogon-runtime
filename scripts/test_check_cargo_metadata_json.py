import json
import tempfile
import unittest
from pathlib import Path

from scripts import check_cargo_metadata_json


def valid_metadata() -> dict:
    return {
        "packages": [
            {
                "id": "path+file:///repo#vogon-core@0.1.0",
                "name": "vogon-core",
                "version": "0.1.0",
                "manifest_path": "/repo/crates/vogon-core/Cargo.toml",
            },
            {
                "id": "path+file:///repo#vogon-cli@0.1.0",
                "name": "vogon-cli",
                "version": "0.1.0",
                "manifest_path": "/repo/crates/vogon-cli/Cargo.toml",
            },
        ],
        "workspace_members": [
            "path+file:///repo#vogon-core@0.1.0",
            "path+file:///repo#vogon-cli@0.1.0",
        ],
        "resolve": {
            "nodes": [
                {
                    "id": "path+file:///repo#vogon-core@0.1.0",
                    "deps": [],
                },
                {
                    "id": "path+file:///repo#vogon-cli@0.1.0",
                    "deps": [],
                },
            ]
        },
    }


class CheckCargoMetadataJsonTests(unittest.TestCase):
    def test_accepts_expected_metadata(self):
        output = json.dumps(valid_metadata())

        self.assertEqual(
            check_cargo_metadata_json.check_output(
                output,
                expected_workspace_packages=["vogon-core", "vogon-cli"],
            ),
            [],
        )

    def test_accepts_metadata_file_path(self):
        with tempfile.TemporaryDirectory() as directory:
            metadata_file = Path(directory) / "metadata.json"
            metadata_file.write_text(json.dumps(valid_metadata()), encoding="utf-8")

            self.assertEqual(
                check_cargo_metadata_json.check_file(
                    metadata_file,
                    expected_workspace_packages=["vogon-core"],
                ),
                [],
            )

    def test_reports_invalid_json(self):
        self.assertEqual(
            check_cargo_metadata_json.check_output("{"),
            [
                "Cargo metadata JSON is invalid: Expecting property name enclosed in double quotes: line 1 column 2 (char 1)"
            ],
        )

    def test_reports_missing_package_fields(self):
        data = valid_metadata()
        data["packages"] = [{"id": "", "name": "vogon-core"}]

        self.assertEqual(
            check_cargo_metadata_json.check_output(json.dumps(data)),
            [
                "Cargo metadata package 1 id must be a non-empty string",
                "Cargo metadata package 1 version must be a non-empty string",
                "Cargo metadata package 1 manifest_path must be a non-empty string",
            ],
        )

    def test_reports_missing_expected_workspace_package(self):
        self.assertEqual(
            check_cargo_metadata_json.check_output(
                json.dumps(valid_metadata()),
                expected_workspace_packages=["vogon-adapters"],
            ),
            [
                'Cargo metadata workspace package missing: expected vogon-adapters, got ["vogon-cli", "vogon-core"]'
            ],
        )

    def test_reports_missing_resolve_nodes(self):
        data = valid_metadata()
        data["resolve"] = {"nodes": []}

        self.assertEqual(
            check_cargo_metadata_json.check_output(json.dumps(data)),
            ["Cargo metadata JSON resolve.nodes must be a non-empty array"],
        )


if __name__ == "__main__":
    unittest.main()
