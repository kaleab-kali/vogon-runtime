import unittest

from scripts import check_live_replay


def valid_replay(**overrides):
    replay = {
        "schema_version": 1,
        "workflow_name": "support-triage",
        "runtime": {
            "provider": "openrouter",
            "adapter": "openrouter-openai-compatible-chat-completions",
            "model": "openrouter/free",
            "parameters": {
                "base_url": "https://openrouter.ai/api/v1",
                "timeout_nanos": "60000000000",
                "max_retries": "2",
            },
        },
        "steps": [
            {"step_id": "classify", "output": "billing"},
            {"step_id": "draft_response", "output": "Hello"},
        ],
    }
    replay.update(overrides)
    return replay


class CheckLiveReplayTests(unittest.TestCase):
    def test_accepts_valid_provider_replay(self):
        self.assertEqual(
            check_live_replay.check_replay(
                valid_replay(),
                provider="openrouter",
                model="openrouter/free",
                secret_value="secret-value",
            ),
            [],
        )

    def test_accepts_configured_openai_compatible_base_url(self):
        replay = valid_replay(
            runtime={
                "provider": "openai-compatible",
                "adapter": "openai-compatible-chat-completions",
                "model": "model-name",
                "parameters": {
                    "base_url": "https://example.com/v1",
                    "timeout_nanos": "60000000000",
                    "max_retries": "2",
                },
            }
        )

        self.assertEqual(
            check_live_replay.check_replay(
                replay,
                provider="openai-compatible",
                model="model-name",
                base_url="https://example.com/v1/",
            ),
            [],
        )

    def test_reports_runtime_and_step_mismatches(self):
        replay = valid_replay(
            workflow_name="other",
            runtime={
                "provider": "openrouter",
                "adapter": "wrong",
                "model": "wrong-model",
                "parameters": {
                    "base_url": "https://openrouter.ai/api/v1",
                    "timeout_nanos": "1",
                    "max_retries": "99",
                },
            },
            steps=[
                {"output": ""},
                {"output": "[REDACTED:openrouter_api_key]"},
                {"output": "extra"},
            ],
        )

        errors = check_live_replay.check_replay(
            replay,
            provider="openrouter",
            model="openrouter/free",
            secret_value="secret-value",
        )

        self.assertIn("workflow_name mismatch: expected 'support-triage', got 'other'", errors)
        self.assertIn("runtime.adapter mismatch: expected 'openrouter-openai-compatible-chat-completions', got 'wrong'", errors)
        self.assertIn("runtime.model mismatch: expected 'openrouter/free', got 'wrong-model'", errors)
        self.assertIn("runtime.parameters.timeout_nanos mismatch: expected '60000000000', got '1'", errors)
        self.assertIn("runtime.parameters.max_retries mismatch: expected '2', got '99'", errors)
        self.assertIn("steps length mismatch: expected 2, got 3", errors)
        self.assertIn("steps[0].output must be a non-empty string", errors)
        self.assertIn(
            "steps[1].output contains redaction marker [REDACTED:openrouter_api_key]",
            errors,
        )

    def test_reports_secret_leak(self):
        replay = valid_replay(steps=[{"output": "secret-value"}, {"output": "ok"}])

        self.assertIn(
            "replay contains secret value from OPENROUTER_API_KEY",
            check_live_replay.check_replay(
                replay,
                provider="openrouter",
                model="openrouter/free",
                secret_value="secret-value",
            ),
        )


if __name__ == "__main__":
    unittest.main()
