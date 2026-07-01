import tempfile
import unittest
from pathlib import Path

from scripts import check_env_example


class CheckEnvExampleTests(unittest.TestCase):
    def test_accepts_blank_expected_provider_variables(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".env.example").write_text(
                "\n".join(f"{name}=" for name in sorted(check_env_example.EXPECTED_ENV_VARS)),
                encoding="utf-8",
            )

            self.assertEqual(check_env_example.check_env_example(root), [])

    def test_reports_missing_unexpected_and_populated_values(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".env.example").write_text(
                "\n".join(
                    [
                        "GEMINI_API" + "_KEY=secret",
                        "GROQ_API_KEY=",
                        "HF_TOKEN=",
                        "OPENROUTER_API_KEY=",
                        "EXTRA_KEY=",
                    ]
                ),
                encoding="utf-8",
            )

            errors = check_env_example.check_env_example(root)

            self.assertEqual(len(errors), 3)
            self.assertIn("OPENAI_COMPATIBLE_API_KEY", errors[0])
            self.assertIn("EXTRA_KEY", errors[1])
            self.assertIn("GEMINI_API_KEY", errors[2])

    def test_reports_malformed_assignment_lines(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".env.example").write_text("GEMINI_API_KEY\n", encoding="utf-8")

            errors = check_env_example.check_env_example(root)

            self.assertEqual(len(errors), 1)
            self.assertIn("expected KEY=VALUE assignment", errors[0])


if __name__ == "__main__":
    unittest.main()
