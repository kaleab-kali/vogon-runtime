import json
import unittest

from scripts import check_verify_json


class CheckVerifyJsonTests(unittest.TestCase):
    def test_accepts_expected_match_output(self):
        output = json.dumps(
            {"workflow_name": "support-triage", "is_match": True, "mismatches": []}
        )

        self.assertEqual(
            check_verify_json.check_output(
                output,
                expected_workflow_name="support-triage",
                expected_match=True,
            ),
            [],
        )

    def test_accepts_expected_mismatch_output(self):
        output = json.dumps(
            {
                "workflow_name": "support-triage",
                "is_match": False,
                "mismatches": [{"step_id": "classify"}],
            }
        )

        self.assertEqual(
            check_verify_json.check_output(output, expected_match=False),
            [],
        )

    def test_reports_invalid_json(self):
        self.assertEqual(
            check_verify_json.check_output("{"),
            [
                "verify JSON is invalid: Expecting property name enclosed in double quotes: line 1 column 2 (char 1)"
            ],
        )

    def test_reports_malformed_fields(self):
        output = json.dumps(
            {"workflow_name": "", "is_match": "yes", "mismatches": {}}
        )

        self.assertEqual(
            check_verify_json.check_output(output),
            [
                "verify JSON workflow_name must be a non-empty string",
                "verify JSON is_match must be a boolean",
                "verify JSON mismatches must be an array",
            ],
        )

    def test_reports_expected_match_mismatches(self):
        output = json.dumps(
            {
                "workflow_name": "writing-pipeline",
                "is_match": False,
                "mismatches": [],
            }
        )

        self.assertEqual(
            check_verify_json.check_output(
                output,
                expected_workflow_name="support-triage",
                expected_match=True,
            ),
            [
                'verify JSON workflow_name mismatch: expected support-triage, got "writing-pipeline"',
                "verify JSON is_match mismatch: expected true, got false",
            ],
        )

    def test_reports_match_with_mismatches(self):
        output = json.dumps(
            {
                "workflow_name": "support-triage",
                "is_match": True,
                "mismatches": [{"step_id": "classify"}],
            }
        )

        self.assertEqual(
            check_verify_json.check_output(output),
            ["verify JSON mismatches must be empty when is_match is true"],
        )

    def test_reports_expected_mismatch_without_mismatches(self):
        output = json.dumps(
            {"workflow_name": "support-triage", "is_match": False, "mismatches": []}
        )

        self.assertEqual(
            check_verify_json.check_output(output, expected_match=False),
            ["verify JSON mismatches must be non-empty for expected mismatches"],
        )


if __name__ == "__main__":
    unittest.main()
