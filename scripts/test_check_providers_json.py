import json
import unittest

from scripts import check_providers_json


class CheckProvidersJsonTests(unittest.TestCase):
    def test_accepts_expected_provider_output(self):
        self.assertEqual(check_providers_json.check_output(provider_output()), [])

    def test_reports_invalid_json(self):
        errors = check_providers_json.check_output("{")

        self.assertEqual(
            errors,
            [
                "providers JSON is invalid: Expecting property name enclosed in double quotes: "
                "line 1 column 2 (char 1)"
            ],
        )

    def test_reports_missing_provider(self):
        data = json.loads(provider_output())
        data["providers"] = [
            provider for provider in data["providers"] if provider["name"] != "openrouter"
        ]

        self.assertEqual(
            check_providers_json.check_output(json.dumps(data)),
            ["providers must include openrouter"],
        )

    def test_reports_wrong_default_count(self):
        data = json.loads(provider_output())
        for provider in data["providers"]:
            provider["default"] = False

        self.assertEqual(
            check_providers_json.check_output(json.dumps(data)),
            [
                "provider deterministic default mismatch: expected true, got false",
                "exactly one provider must be default, found 0",
            ],
        )

    def test_reports_provider_metadata_mismatch(self):
        data = json.loads(provider_output())
        gemini = next(provider for provider in data["providers"] if provider["name"] == "gemini")
        gemini["default_model"] = "gemini-old"

        self.assertEqual(
            check_providers_json.check_output(json.dumps(data)),
            [
                'provider gemini default_model mismatch: expected "gemini-3.1-flash-lite", got "gemini-old"'
            ],
        )

    def test_reports_non_boolean_credential_status(self):
        data = json.loads(provider_output())
        groq = next(provider for provider in data["providers"] if provider["name"] == "groq")
        groq["credential_configured"] = "secret-groq-key"

        self.assertEqual(
            check_providers_json.check_output(json.dumps(data)),
            [
                'provider groq credential_configured must be boolean or null, got "secret-groq-key"'
            ],
        )


def provider_output() -> str:
    return json.dumps(
        {
            "providers": [
                {
                    "name": "deterministic",
                    "enabled": True,
                    "default": True,
                    "credential_env": None,
                    "credential_configured": None,
                    "default_base_url": None,
                    "default_model": None,
                    "documentation_url": "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#deterministic",
                    "usage_url": None,
                },
                {
                    "name": "gemini",
                    "enabled": True,
                    "default": False,
                    "credential_env": "GEMINI_API_KEY",
                    "credential_configured": False,
                    "default_base_url": None,
                    "default_model": "gemini-3.1-flash-lite",
                    "documentation_url": "https://ai.google.dev/gemini-api/docs",
                    "usage_url": "https://ai.google.dev/gemini-api/docs/pricing",
                },
                {
                    "name": "groq",
                    "enabled": True,
                    "default": False,
                    "credential_env": "GROQ_API_KEY",
                    "credential_configured": True,
                    "default_base_url": "https://api.groq.com/openai/v1",
                    "default_model": "llama-3.1-8b-instant",
                    "documentation_url": "https://console.groq.com/docs/openai",
                    "usage_url": "https://console.groq.com/docs/rate-limits",
                },
                {
                    "name": "hugging-face",
                    "enabled": True,
                    "default": False,
                    "credential_env": "HF_TOKEN",
                    "credential_configured": True,
                    "default_base_url": "https://router.huggingface.co/v1",
                    "default_model": "openai/gpt-oss-120b:fastest",
                    "documentation_url": "https://huggingface.co/docs/inference-providers",
                    "usage_url": "https://huggingface.co/docs/inference-providers/pricing",
                },
                {
                    "name": "openrouter",
                    "enabled": True,
                    "default": False,
                    "credential_env": "OPENROUTER_API_KEY",
                    "credential_configured": False,
                    "default_base_url": "https://openrouter.ai/api/v1",
                    "default_model": "openrouter/free",
                    "documentation_url": "https://openrouter.ai/docs",
                    "usage_url": "https://openrouter.ai/pricing",
                },
                {
                    "name": "openai-compatible",
                    "enabled": True,
                    "default": False,
                    "credential_env": "OPENAI_COMPATIBLE_API_KEY",
                    "credential_configured": None,
                    "default_base_url": "https://router.huggingface.co/v1",
                    "default_model": "openai/gpt-oss-120b:fastest",
                    "documentation_url": "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#openai-compatible",
                    "usage_url": None,
                },
            ]
        }
    )


if __name__ == "__main__":
    unittest.main()
