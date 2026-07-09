#!/usr/bin/env python3
"""Validate live provider smoke workflow wiring."""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path


WORKFLOW_PATTERN = "live-*-smoke.y*ml"


@dataclass(frozen=True)
class ProviderExpectation:
    provider: str
    adapter: str
    base_url: str | None
    secret_env: str
    redaction_label: str


PROVIDERS = {
    "gemini": ProviderExpectation(
        provider="gemini",
        adapter="gemini-generate-content",
        base_url="https://generativelanguage.googleapis.com",
        secret_env="GEMINI_API_KEY",
        redaction_label="gemini_api_key",
    ),
    "groq": ProviderExpectation(
        provider="groq",
        adapter="groq-openai-compatible-chat-completions",
        base_url="https://api.groq.com/openai/v1",
        secret_env="GROQ_API_KEY",
        redaction_label="groq_api_key",
    ),
    "hugging-face": ProviderExpectation(
        provider="hugging-face",
        adapter="hugging-face-openai-compatible-chat-completions",
        base_url="https://router.huggingface.co/v1",
        secret_env="HF_TOKEN",
        redaction_label="hf_token",
    ),
    "openai-compatible": ProviderExpectation(
        provider="openai-compatible",
        adapter="openai-compatible-chat-completions",
        base_url=None,
        secret_env="OPENAI_COMPATIBLE_API_KEY",
        redaction_label="openai_compatible_api_key",
    ),
    "openrouter": ProviderExpectation(
        provider="openrouter",
        adapter="openrouter-openai-compatible-chat-completions",
        base_url="https://openrouter.ai/api/v1",
        secret_env="OPENROUTER_API_KEY",
        redaction_label="openrouter_api_key",
    ),
}


@dataclass(frozen=True)
class LiveWorkflowExpectation:
    provider: str
    file_name: str
    flag_prefix: str
    replay_path: str
    default_model: str
    model_env: str
    requires_base_url: bool = False
    default_base_url: str | None = None
    base_url_env: str | None = None


EXPECTED_WORKFLOWS = {
    "gemini": LiveWorkflowExpectation(
        provider="gemini",
        file_name="live-gemini-smoke.yml",
        flag_prefix="gemini",
        replay_path="target/live-gemini-smoke.replay.json",
        default_model="gemini-3.1-flash-lite",
        model_env="GEMINI_MODEL",
    ),
    "groq": LiveWorkflowExpectation(
        provider="groq",
        file_name="live-groq-smoke.yml",
        flag_prefix="groq",
        replay_path="target/live-groq-smoke.replay.json",
        default_model="llama-3.1-8b-instant",
        model_env="GROQ_MODEL",
    ),
    "hugging-face": LiveWorkflowExpectation(
        provider="hugging-face",
        file_name="live-hugging-face-smoke.yml",
        flag_prefix="hugging-face",
        replay_path="target/live-hugging-face-smoke.replay.json",
        default_model="openai/gpt-oss-120b:fastest",
        model_env="HUGGING_FACE_MODEL",
    ),
    "openai-compatible": LiveWorkflowExpectation(
        provider="openai-compatible",
        file_name="live-openai-compatible-smoke.yml",
        flag_prefix="openai-compatible",
        replay_path="target/live-openai-compatible-smoke.replay.json",
        default_model="openai/gpt-oss-120b:fastest",
        model_env="OPENAI_COMPATIBLE_MODEL",
        requires_base_url=True,
        default_base_url="https://router.huggingface.co/v1",
        base_url_env="OPENAI_COMPATIBLE_BASE_URL",
    ),
    "openrouter": LiveWorkflowExpectation(
        provider="openrouter",
        file_name="live-openrouter-smoke.yml",
        flag_prefix="openrouter",
        replay_path="target/live-openrouter-smoke.replay.json",
        default_model="openrouter/free",
        model_env="OPENROUTER_MODEL",
    ),
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        default=Path.cwd(),
        type=Path,
        help="Repository root to scan. Defaults to the current directory.",
    )
    args = parser.parse_args()

    errors = check_repository(args.root.resolve())
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_repository(root: Path) -> list[str]:
    workflows_dir = root / ".github" / "workflows"
    errors: list[str] = []

    expected_files = {expectation.file_name for expectation in EXPECTED_WORKFLOWS.values()}
    actual_files = {
        path.name
        for path in workflows_dir.glob(WORKFLOW_PATTERN)
        if path.is_file() and path.suffix.lower() in {".yml", ".yaml"}
    }

    for missing in sorted(expected_files - actual_files):
        errors.append(f".github/workflows/{missing}: missing live provider smoke workflow")

    for unexpected in sorted(actual_files - expected_files):
        errors.append(
            f".github/workflows/{unexpected}: unexpected live provider smoke workflow"
        )

    for expectation in EXPECTED_WORKFLOWS.values():
        path = workflows_dir / expectation.file_name
        if path.exists():
            errors.extend(check_workflow_file(root, path, expectation))

    return errors


def check_workflow_file(
    root: Path,
    path: Path,
    expectation: LiveWorkflowExpectation,
) -> list[str]:
    relative_path = path.relative_to(root).as_posix()
    text = path.read_text(encoding="utf-8")
    provider = PROVIDERS[expectation.provider]
    secret_ref = "${{ secrets." + provider.secret_env + " }}"

    required_snippets = {
        "workflow_dispatch trigger": "  workflow_dispatch:",
        "workflow_call trigger": "  workflow_call:",
        "read-only top-level contents permission": "permissions:\n  contents: read",
        "concurrency group": "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
        "concurrency preserves live runs": "  cancel-in-progress: false",
        "cargo network retry env": "env:\n  CARGO_NET_RETRY: 10",
        "ubuntu runner": "    runs-on: ubuntu-24.04",
        "job timeout": "    timeout-minutes:",
        "workflow_call secret declaration": (
            f"      {provider.secret_env}:\n        required: true"
        ),
        "checkout step": "        uses: actions/checkout@v7",
        "Rust toolchain step": "        run: rustup show",
        "release CLI build": "        run: cargo build --release -p vogon-cli --locked",
        "secret env wiring": f"      {provider.secret_env}: {secret_ref}",
        "secret presence guard": f'if [ -z "${{{provider.secret_env}:-}}" ]; then',
        "provider run flag": f"            --provider {expectation.provider}",
        "timeout run flag": (
            f"            --{expectation.flag_prefix}-timeout-seconds 60"
        ),
        "retry run flag": f"            --{expectation.flag_prefix}-max-retries 2",
        "redaction run flag": (
            f'            --redact {provider.redaction_label}="$'
            f'{provider.secret_env}"'
        ),
        "replay output path": f"            --output {expectation.replay_path}",
        "live replay validator": "          cargo run -p vogon-xtask -- check-live-replay",
        "validator replay path": f"            --replay {expectation.replay_path}",
        "validator provider": f"            --provider {expectation.provider}",
        "validator model": validator_model_snippet(expectation),
        "validator secret env": f"            --secret-env {provider.secret_env}",
    }

    if expectation.provider != "gemini":
        required_snippets["workflow_dispatch model input"] = (
            "      model:\n"
            f"        description: {model_description(expectation)}\n"
            "        required: false\n"
            f"        default: {expectation.default_model}"
        )
        required_snippets["workflow_call model input"] = (
            "      model:\n"
            "        type: string\n"
            "        required: false\n"
            f"        default: {expectation.default_model}"
        )
        required_snippets["model env wiring"] = (
            f"      {expectation.model_env}: ${{{{ inputs.model }}}}"
        )
        required_snippets["model fallback"] = (
            f'model="${{{expectation.model_env}:-{expectation.default_model}}}"'
        )
        required_snippets["model export"] = f'export {expectation.model_env}="$model"'
        required_snippets["model run flag"] = (
            f"            --{expectation.flag_prefix}-model \"$model\""
        )

    if expectation.requires_base_url:
        assert expectation.default_base_url is not None
        assert expectation.base_url_env is not None
        required_snippets["workflow_dispatch base URL input"] = (
            "      base_url:\n"
            "        description: OpenAI-compatible API base URL.\n"
            "        required: false\n"
            f"        default: {expectation.default_base_url}"
        )
        required_snippets["workflow_call base URL input"] = (
            "      base_url:\n"
            "        type: string\n"
            "        required: false\n"
            f"        default: {expectation.default_base_url}"
        )
        required_snippets["base URL env wiring"] = (
            f"      {expectation.base_url_env}: ${{{{ inputs.base_url }}}}"
        )
        required_snippets["base URL fallback"] = (
            f'base_url="${{{expectation.base_url_env}:-{expectation.default_base_url}}}"'
        )
        required_snippets["base URL export"] = (
            f'export {expectation.base_url_env}="$base_url"'
        )
        required_snippets["base URL run flag"] = (
            f"            --{expectation.flag_prefix}-base-url"
        )
        required_snippets["validator base URL"] = "            --base-url"

    errors = []
    for description, snippet in required_snippets.items():
        if snippet not in text:
            errors.append(f"{relative_path}: missing {description}")

    return errors


def model_description(expectation: LiveWorkflowExpectation) -> str:
    if expectation.provider == "openai-compatible":
        return "OpenAI-compatible model name."
    if expectation.provider == "hugging-face":
        return "Hugging Face model name."
    return f"{provider_display_name(expectation.provider)} model name."


def validator_model_snippet(expectation: LiveWorkflowExpectation) -> str:
    if expectation.provider == "gemini":
        return f"            --model {expectation.default_model}"
    return '            --model "$model"'


def provider_display_name(provider: str) -> str:
    names = {
        "groq": "Groq",
        "openrouter": "OpenRouter",
    }
    return names.get(provider, provider)


if __name__ == "__main__":
    raise SystemExit(main())
