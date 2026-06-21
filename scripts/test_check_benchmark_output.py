import unittest

from scripts import check_benchmark_output


class CheckBenchmarkOutputTests(unittest.TestCase):
    def test_accepts_expected_metrics(self):
        output = "\n".join(
            [
                "Compiling benchmark harness",
                "iterations: 100",
                "elapsed_ms: 2.5",
                "iterations_per_second: 40",
            ]
        )

        self.assertEqual(
            check_benchmark_output.check_output(output, expected_iterations=100),
            [],
        )

    def test_reports_missing_metrics(self):
        errors = check_benchmark_output.check_output(
            "iterations: 100\n", expected_iterations=100
        )

        self.assertEqual(
            errors,
            [
                "missing benchmark metric: elapsed_ms",
                "missing benchmark metric: iterations_per_second",
            ],
        )

    def test_reports_iteration_mismatch(self):
        output = "\n".join(
            [
                "iterations: 10",
                "elapsed_ms: 1",
                "iterations_per_second: 10",
            ]
        )

        errors = check_benchmark_output.check_output(output, expected_iterations=100)

        self.assertEqual(
            errors,
            ["benchmark iterations mismatch: expected 100, got 10"],
        )

    def test_rejects_invalid_and_non_positive_metrics(self):
        output = "\n".join(
            [
                "iterations: no",
                "elapsed_ms: 0",
                "iterations_per_second: nan",
            ]
        )

        errors = check_benchmark_output.check_output(output, expected_iterations=100)

        self.assertEqual(
            errors,
            [
                "benchmark iterations must be an integer",
                "benchmark elapsed_ms must be greater than zero",
                "benchmark iterations_per_second must be finite",
            ],
        )


if __name__ == "__main__":
    unittest.main()
