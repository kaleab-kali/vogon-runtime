import json
import unittest

from scripts import check_doctor_json


class CheckDoctorJsonTests(unittest.TestCase):
    def test_accepts_expected_doctor_output(self):
        output = json.dumps(
            {
                "status": "ok",
                "version": "0.1.0",
                "checks": [
                    {
                        "name": "deterministic_runtime",
                        "status": "ok",
                        "message": "deterministic runtime executed a one-step workflow",
                    }
                ],
                "providers": [
                    {"name": "deterministic", "usage_url": None},
                    {
                        "name": "gemini",
                        "usage_url": "https://ai.google.dev/gemini-api/docs/pricing",
                    },
                    {
                        "name": "groq",
                        "usage_url": "https://console.groq.com/docs/rate-limits",
                    },
                    {
                        "name": "hugging-face",
                        "usage_url": "https://huggingface.co/docs/inference-providers/pricing",
                    },
                    {
                        "name": "openrouter",
                        "usage_url": "https://openrouter.ai/pricing",
                    },
                    {"name": "openai-compatible", "usage_url": None},
                ],
            }
        )

        self.assertEqual(check_doctor_json.check_output(output), [])

    def test_reports_invalid_json(self):
        errors = check_doctor_json.check_output("{")

        self.assertEqual(
            errors,
            [
                "doctor JSON is invalid: Expecting property name enclosed in double quotes: "
                "line 1 column 2 (char 1)"
            ],
        )

    def test_reports_missing_runtime_check(self):
        output = json.dumps(
            {
                "status": "ok",
                "checks": [],
                "providers": [
                    {"name": "deterministic", "usage_url": None},
                    {
                        "name": "gemini",
                        "usage_url": "https://ai.google.dev/gemini-api/docs/pricing",
                    },
                    {
                        "name": "groq",
                        "usage_url": "https://console.groq.com/docs/rate-limits",
                    },
                    {
                        "name": "hugging-face",
                        "usage_url": "https://huggingface.co/docs/inference-providers/pricing",
                    },
                    {
                        "name": "openrouter",
                        "usage_url": "https://openrouter.ai/pricing",
                    },
                    {"name": "openai-compatible", "usage_url": None},
                ],
            }
        )

        self.assertEqual(
            check_doctor_json.check_output(output),
            ["doctor checks must include ok deterministic_runtime"],
        )

    def test_reports_provider_usage_url_mismatch(self):
        output = json.dumps(
            {
                "status": "ok",
                "checks": [{"name": "deterministic_runtime", "status": "ok"}],
                "providers": [
                    {"name": "deterministic", "usage_url": None},
                    {"name": "gemini", "usage_url": "https://example.com"},
                    {
                        "name": "groq",
                        "usage_url": "https://console.groq.com/docs/rate-limits",
                    },
                    {
                        "name": "hugging-face",
                        "usage_url": "https://huggingface.co/docs/inference-providers/pricing",
                    },
                    {
                        "name": "openrouter",
                        "usage_url": "https://openrouter.ai/pricing",
                    },
                    {
                        "name": "openai-compatible",
                        "usage_url": "https://example.com/usage",
                    },
                ],
            }
        )

        self.assertEqual(
            check_doctor_json.check_output(output),
            [
                "doctor provider gemini usage_url mismatch: expected https://ai.google.dev/gemini-api/docs/pricing, got \"https://example.com\"",
                "doctor provider openai-compatible usage_url must be null, got \"https://example.com/usage\"",
            ],
        )


if __name__ == "__main__":
    unittest.main()
