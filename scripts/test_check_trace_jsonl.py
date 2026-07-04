import json
import unittest

from scripts import check_trace_jsonl


def valid_trace_jsonl() -> str:
    return "\n".join(
        [
            json.dumps(
                {
                    "event": "run",
                    "schema_version": 1,
                    "workflow_name": "support-triage",
                    "runtime": {
                        "provider": "deterministic",
                        "model": "deterministic-echo",
                    },
                    "run_hash": "a" * 64,
                    "step_count": 2,
                }
            ),
            json.dumps(
                {
                    "event": "step",
                    "index": 1,
                    "step_id": "classify",
                    "input_hash": "b" * 64,
                    "output_hash": "c" * 64,
                    "output": "classify:input",
                }
            ),
            json.dumps(
                {
                    "event": "step",
                    "index": 2,
                    "step_id": "draft_response",
                    "input_hash": "d" * 64,
                    "output_hash": "e" * 64,
                    "output": "draft_response:input",
                }
            ),
        ]
    )


class CheckTraceJsonlTests(unittest.TestCase):
    def test_accepts_expected_trace_output(self):
        self.assertEqual(
            check_trace_jsonl.check_output(
                valid_trace_jsonl(),
                expected_provider="deterministic",
                expected_model="deterministic-echo",
                expected_step_count=2,
            ),
            [],
        )

    def test_reports_empty_output(self):
        self.assertEqual(
            check_trace_jsonl.check_output(""),
            ["trace JSONL output must not be empty"],
        )

    def test_reports_invalid_json_line(self):
        self.assertEqual(
            check_trace_jsonl.check_output("{"),
            [
                "trace JSONL line 1 is invalid JSON: Expecting property name enclosed in double quotes: line 1 column 2 (char 1)"
            ],
        )

    def test_reports_runtime_mismatches(self):
        trace = valid_trace_jsonl().replace('"deterministic"', '"gemini"', 1)

        self.assertEqual(
            check_trace_jsonl.check_output(
                trace,
                expected_provider="deterministic",
                expected_model="deterministic-echo",
                expected_step_count=2,
            ),
            [
                'trace JSONL runtime provider mismatch: expected deterministic, got "gemini"'
            ],
        )

    def test_reports_step_count_mismatches(self):
        events = valid_trace_jsonl().splitlines()
        trace = "\n".join(events[:2])

        self.assertEqual(
            check_trace_jsonl.check_output(trace, expected_step_count=2),
            [
                "trace JSONL step event count mismatch: expected 2, got 1",
                "trace JSONL run step_count must match step events: expected 2, got 1",
            ],
        )

    def test_reports_malformed_step_event(self):
        events = [json.loads(line) for line in valid_trace_jsonl().splitlines()]
        events[1]["index"] = 2
        events[1]["output_hash"] = ""
        trace = "\n".join(json.dumps(event) for event in events)

        self.assertEqual(
            check_trace_jsonl.check_output(trace),
            [
                "trace JSONL step index mismatch at event 2: expected 1, got 2",
                "trace JSONL step 1 field output_hash must be a non-empty string",
            ],
        )


if __name__ == "__main__":
    unittest.main()
