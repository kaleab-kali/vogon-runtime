import json
import tempfile
import unittest
from pathlib import Path

from scripts import check_spdx_sbom_json


def valid_sbom() -> dict:
    return {
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": "vogon-runtime v0.1.0",
        "documentNamespace": "https://github.com/kaleab-kali/vogon-runtime/releases/v0.1.0/sbom/1",
        "creationInfo": {
            "creators": ["Tool: vogon-runtime scripts/write_spdx_sbom.py"],
        },
        "packages": [
            {
                "SPDXID": "SPDXRef-Package-vogon-runtime-source",
                "name": "vogon-runtime-source",
                "downloadLocation": "git+https://github.com/kaleab-kali/vogon-runtime.git",
            },
            {
                "SPDXID": "SPDXRef-Package-vogon-core",
                "name": "vogon-core",
                "downloadLocation": "file:///repo/crates/vogon-core/Cargo.toml",
            },
        ],
        "relationships": [
            {
                "spdxElementId": "SPDXRef-DOCUMENT",
                "relationshipType": "DESCRIBES",
                "relatedSpdxElement": "SPDXRef-Package-vogon-core",
            },
            {
                "spdxElementId": "SPDXRef-Package-vogon-cli",
                "relationshipType": "DEPENDS_ON",
                "relatedSpdxElement": "SPDXRef-Package-vogon-core",
            },
        ],
    }


class CheckSpdxSbomJsonTests(unittest.TestCase):
    def test_accepts_expected_sbom(self):
        output = json.dumps(valid_sbom())

        self.assertEqual(
            check_spdx_sbom_json.check_output(
                output,
                expected_name="vogon-runtime v0.1.0",
                expected_packages=["vogon-core"],
            ),
            [],
        )

    def test_accepts_sbom_file_path(self):
        with tempfile.TemporaryDirectory() as directory:
            sbom_file = Path(directory) / "sbom.spdx.json"
            sbom_file.write_text(json.dumps(valid_sbom()), encoding="utf-8")

            self.assertEqual(
                check_spdx_sbom_json.check_file(
                    sbom_file,
                    expected_name="vogon-runtime v0.1.0",
                    expected_packages=["vogon-runtime-source"],
                ),
                [],
            )

    def test_reports_invalid_json(self):
        self.assertEqual(
            check_spdx_sbom_json.check_output("{"),
            [
                "SPDX SBOM JSON is invalid: Expecting property name enclosed in double quotes: line 1 column 2 (char 1)"
            ],
        )

    def test_reports_document_mismatches(self):
        data = valid_sbom()
        data["spdxVersion"] = "SPDX-2.2"
        data["dataLicense"] = "MIT"
        data["name"] = "other"
        data["documentNamespace"] = "not-a-url"

        self.assertEqual(
            check_spdx_sbom_json.check_output(
                json.dumps(data),
                expected_name="vogon-runtime v0.1.0",
            ),
            [
                'SPDX SBOM spdxVersion mismatch: expected SPDX-2.3, got "SPDX-2.2"',
                'SPDX SBOM dataLicense mismatch: expected CC0-1.0, got "MIT"',
                'SPDX SBOM name mismatch: expected vogon-runtime v0.1.0, got "other"',
                "SPDX SBOM documentNamespace must be an HTTPS URL",
            ],
        )

    def test_reports_missing_expected_package(self):
        self.assertEqual(
            check_spdx_sbom_json.check_output(
                json.dumps(valid_sbom()),
                expected_packages=["vogon-cli"],
            ),
            [
                'SPDX SBOM package missing: expected vogon-cli, got ["vogon-core", "vogon-runtime-source"]'
            ],
        )

    def test_reports_missing_relationship_types(self):
        data = valid_sbom()
        data["relationships"] = []

        self.assertEqual(
            check_spdx_sbom_json.check_output(json.dumps(data)),
            ["SPDX SBOM relationships must be a non-empty array"],
        )

        data["relationships"] = [
            {
                "spdxElementId": "SPDXRef-DOCUMENT",
                "relationshipType": "DESCRIBES",
                "relatedSpdxElement": "SPDXRef-Package-vogon-core",
            }
        ]

        self.assertEqual(
            check_spdx_sbom_json.check_output(json.dumps(data)),
            ["SPDX SBOM relationships must include DEPENDS_ON"],
        )


if __name__ == "__main__":
    unittest.main()
