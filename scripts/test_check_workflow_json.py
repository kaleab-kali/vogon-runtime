import json
import unittest

from scripts import check_workflow_json


class CheckWorkflowJsonTests(unittest.TestCase):
    def test_accepts_expected_workflow_check_output(self):
        output = json.dumps({"workflow_name": "support-triage", "step_count": 2})

        self.assertEqual(
            check_workflow_json.check_output(
                output,
                expected_workflow_name="support-triage",
                expected_step_count=2,
            ),
            [],
        )

    def test_reports_invalid_json(self):
        self.assertEqual(
            check_workflow_json.check_output("{"),
            [
                "workflow check JSON is invalid: Expecting property name enclosed in double quotes: line 1 column 2 (char 1)"
            ],
        )

    def test_reports_non_object_root(self):
        self.assertEqual(
            check_workflow_json.check_output("[]"),
            ["workflow check JSON root must be an object"],
        )

    def test_reports_malformed_fields(self):
        output = json.dumps({"workflow_name": "", "step_count": 0})

        self.assertEqual(
            check_workflow_json.check_output(output),
            [
                "workflow check JSON workflow_name must be a non-empty string",
                "workflow check JSON step_count must be a positive integer",
            ],
        )

    def test_reports_expected_value_mismatches(self):
        output = json.dumps({"workflow_name": "writing-pipeline", "step_count": 3})

        self.assertEqual(
            check_workflow_json.check_output(
                output,
                expected_workflow_name="support-triage",
                expected_step_count=2,
            ),
            [
                'workflow check JSON workflow_name mismatch: expected support-triage, got "writing-pipeline"',
                "workflow check JSON step_count mismatch: expected 2, got 3",
            ],
        )


if __name__ == "__main__":
    unittest.main()
