import json
import tempfile
import unittest
from pathlib import Path

from scripts import check_cache_json


class CheckCacheJsonTests(unittest.TestCase):
    def test_accepts_expected_cache_file(self):
        output = json.dumps(
            {
                "outputs": {"abc": "cached output"},
                "insertion_order": ["abc"],
                "max_entries": 1,
            }
        )

        self.assertEqual(
            check_cache_json.check_output(
                output,
                expected_max_entries=1,
                expected_entry_count=1,
            ),
            [],
        )

    def test_accepts_cache_file_path(self):
        with tempfile.TemporaryDirectory() as directory:
            cache_file = Path(directory) / "vogon.cache.json"
            cache_file.write_text(
                json.dumps(
                    {
                        "outputs": {"abc": "cached output"},
                        "insertion_order": ["abc"],
                        "max_entries": 1,
                    }
                ),
                encoding="utf-8",
            )

            self.assertEqual(
                check_cache_json.check_file(
                    cache_file,
                    expected_max_entries=1,
                    expected_entry_count=1,
                ),
                [],
            )

    def test_reports_invalid_json(self):
        self.assertEqual(
            check_cache_json.check_output("{"),
            [
                "cache JSON is invalid: Expecting property name enclosed in double quotes: line 1 column 2 (char 1)"
            ],
        )

    def test_reports_malformed_fields(self):
        output = json.dumps(
            {
                "outputs": [],
                "insertion_order": {},
                "max_entries": -1,
            }
        )

        self.assertEqual(
            check_cache_json.check_output(output),
            [
                "cache JSON outputs must be an object",
                "cache JSON insertion_order must be an array",
                "cache JSON max_entries must be a non-negative integer",
            ],
        )

    def test_reports_expected_value_mismatches(self):
        output = json.dumps(
            {
                "outputs": {"abc": "cached output", "def": "other output"},
                "insertion_order": ["abc", "def"],
                "max_entries": 2,
            }
        )

        self.assertEqual(
            check_cache_json.check_output(
                output,
                expected_max_entries=1,
                expected_entry_count=1,
            ),
            [
                "cache JSON max_entries mismatch: expected 1, got 2",
                "cache JSON output count mismatch: expected 1, got 2",
            ],
        )

    def test_reports_order_mismatches(self):
        output = json.dumps(
            {
                "outputs": {"abc": "cached output"},
                "insertion_order": ["abc", "missing"],
                "max_entries": 2,
            }
        )

        self.assertEqual(
            check_cache_json.check_output(output),
            [
                "cache JSON insertion_order length must match outputs: expected 1, got 2",
                "cache JSON insertion_order entry 2 is missing from outputs",
            ],
        )

    def test_reports_empty_output_values(self):
        output = json.dumps(
            {
                "outputs": {"": "", "abc": ""},
                "insertion_order": [""],
                "max_entries": 2,
            }
        )

        self.assertEqual(
            check_cache_json.check_output(output),
            [
                "cache JSON insertion_order length must match outputs: expected 2, got 1",
                "cache JSON insertion_order entry 1 must be a non-empty string",
                "cache JSON output keys must be non-empty strings",
                "cache JSON output  must be a non-empty string",
                "cache JSON output abc must be a non-empty string",
            ],
        )


if __name__ == "__main__":
    unittest.main()
