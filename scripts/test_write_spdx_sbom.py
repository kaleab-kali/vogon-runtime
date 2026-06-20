import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import write_spdx_sbom


class WriteSpdxSbomTests(unittest.TestCase):
    def test_build_document_includes_workspace_packages_and_dependencies(self):
        metadata = {
            "packages": [
                {
                    "id": "path+file:///repo/crates/vogon-core#0.1.0",
                    "name": "vogon-core",
                    "version": "0.1.0",
                    "license": "MIT",
                    "manifest_path": "/repo/crates/vogon-core/Cargo.toml",
                    "source": None,
                },
                {
                    "id": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0",
                    "name": "serde",
                    "version": "1.0.0",
                    "license": "MIT OR Apache-2.0",
                    "manifest_path": "/cargo/registry/serde/Cargo.toml",
                    "source": "registry+https://github.com/rust-lang/crates.io-index",
                },
            ],
            "workspace_members": ["path+file:///repo/crates/vogon-core#0.1.0"],
            "resolve": {
                "root": None,
                "nodes": [
                    {
                        "id": "path+file:///repo/crates/vogon-core#0.1.0",
                        "deps": [
                            {
                                "pkg": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0"
                            }
                        ],
                    },
                    {
                        "id": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0",
                        "deps": [],
                    },
                ],
            },
        }

        document = write_spdx_sbom.build_document(
            metadata,
            document_name="vogon-runtime test",
            namespace="https://github.com/kaleab-kali/vogon-runtime/releases/test",
            created="2026-06-21T00:00:00Z",
        )

        self.assertEqual(document["spdxVersion"], "SPDX-2.3")
        self.assertEqual(document["dataLicense"], "CC0-1.0")
        package_ids = {package["SPDXID"] for package in document["packages"]}
        core_id = write_spdx_sbom.package_spdx_id(metadata["packages"][0])
        serde_id = write_spdx_sbom.package_spdx_id(metadata["packages"][1])
        self.assertIn(core_id, package_ids)
        self.assertIn(serde_id, package_ids)
        serde_package = next(
            package for package in document["packages"] if package["SPDXID"] == serde_id
        )
        self.assertEqual(
            serde_package["downloadLocation"],
            "https://github.com/rust-lang/crates.io-index",
        )
        self.assertIn(
            {
                "spdxElementId": core_id,
                "relationshipType": "DEPENDS_ON",
                "relatedSpdxElement": serde_id,
            },
            document["relationships"],
        )

    def test_cli_writes_sorted_json_document(self):
        metadata = {
            "packages": [
                {
                    "id": "path+file:///repo/crates/vogon-core#0.1.0",
                    "name": "vogon-core",
                    "version": "0.1.0",
                    "license": "MIT",
                    "manifest_path": "/repo/crates/vogon-core/Cargo.toml",
                    "source": None,
                }
            ],
            "workspace_members": ["path+file:///repo/crates/vogon-core#0.1.0"],
            "resolve": {
                "root": None,
                "nodes": [
                    {
                        "id": "path+file:///repo/crates/vogon-core#0.1.0",
                        "deps": [],
                    }
                ],
            },
        }

        with tempfile.TemporaryDirectory() as directory:
            metadata_path = Path(directory) / "metadata.json"
            output_path = Path(directory) / "sbom.spdx.json"
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")

            exit_code = write_spdx_sbom.main_with_args(
                [
                    "--metadata",
                    str(metadata_path),
                    "--output",
                    str(output_path),
                    "--document-name",
                    "vogon-runtime test",
                    "--namespace",
                    "https://github.com/kaleab-kali/vogon-runtime/releases/test",
                    "--created",
                    "2026-06-21T00:00:00Z",
                ]
            )

            self.assertEqual(exit_code, 0)
            written = json.loads(output_path.read_text(encoding="utf-8"))
            self.assertEqual(written["name"], "vogon-runtime test")


if __name__ == "__main__":
    unittest.main()
