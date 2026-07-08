# Contributing

Thanks for helping improve Vogon Runtime.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo check -p vogon-cli --no-default-features --locked
python -m unittest scripts.test_write_spdx_sbom
python -m unittest scripts.test_check_cargo_metadata_json
python -m unittest scripts.test_check_spdx_sbom_json
python -m unittest scripts.test_check_sha256_file
python -m unittest scripts.test_check_archive_contents
python -m unittest scripts.test_check_benchmark_output
python -m unittest scripts.test_check_cargo_manifests
python -m unittest scripts.test_check_changelog
python -m unittest scripts.test_check_contributing_checklist
python -m unittest scripts.test_check_container_policy
python -m unittest scripts.test_check_container_image
python -m unittest scripts.test_check_dependabot_config
python -m unittest scripts.test_check_deployment_checklist
python -m unittest scripts.test_check_doctor_json
python -m unittest scripts.test_check_providers_json
python -m unittest scripts.test_check_cache_json
python -m unittest scripts.test_check_workflow_json
python -m unittest scripts.test_check_verify_json
python -m unittest scripts.test_check_trace_jsonl
python -m unittest scripts.test_check_docs_links
python -m unittest scripts.test_check_public_status_docs
python -m unittest scripts.test_check_env_example
python -m unittest scripts.test_check_issue_templates
python -m unittest scripts.test_check_live_replay
python -m unittest scripts.test_check_live_workflows
python -m unittest scripts.test_check_package_verification_docs
python -m unittest scripts.test_check_pr_template
python -m unittest scripts.test_check_release_checklist
python -m unittest scripts.test_check_release_workflow
python -m unittest scripts.test_check_schema_files
python -m unittest scripts.test_check_secrets
python -m unittest scripts.test_check_ci_workflow
python -m unittest scripts.test_check_security_workflows
python -m unittest scripts.test_check_workflow_policies
python scripts/check_cargo_manifests.py --root .
python scripts/check_changelog.py --root .
python scripts/check_contributing_checklist.py --root .
python scripts/check_deployment_checklist.py --root .
python scripts/check_docs_links.py --root .
python scripts/check_public_status_docs.py --root .
python scripts/check_env_example.py --root .
python scripts/check_issue_templates.py --root .
python scripts/check_container_policy.py --root .
python scripts/check_dependabot_config.py --root .
python scripts/check_live_workflows.py --root .
python scripts/check_package_verification_docs.py --root .
python scripts/check_pr_template.py --root .
python scripts/check_release_checklist.py --root .
python scripts/check_release_workflow.py --root .
python scripts/check_schema_files.py --root .
python scripts/check_secrets.py --root .
python scripts/check_ci_workflow.py --root .
python scripts/check_security_workflows.py --root .
python scripts/check_workflow_policies.py --root .
cargo +1.85.0 test --workspace --all-features --locked
cargo bench -p vogon-core --bench runtime --locked -- --iterations 100 | python scripts/check_benchmark_output.py --expected-iterations 100
cargo build --release --workspace --all-features --locked
cargo run --release -p vogon-cli -- doctor --json | python scripts/check_doctor_json.py
cargo run --release -p vogon-cli -- providers --json | python scripts/check_providers_json.py
cargo run --release -p vogon-cli -- init --force --output target/vogon-init-smoke/workflow.toml
cargo run --release -p vogon-cli -- check --json target/vogon-init-smoke/workflow.toml | python scripts/check_workflow_json.py --expected-workflow-name starter-workflow --expected-step-count 2
cargo run --release -p vogon-cli -- check --json fixtures/workflows/support-triage.toml | python scripts/check_workflow_json.py --expected-workflow-name support-triage --expected-step-count 2
cargo run --release -p vogon-cli -- verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json | python scripts/check_verify_json.py --expected-workflow-name support-triage --expect-match
cargo run --release -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
cargo run --release -p vogon-cli -- verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json
cargo run --release -p vogon-cli -- trace --jsonl fixtures/replays/support-triage.replay.json | python scripts/check_trace_jsonl.py --expected-provider deterministic --expected-model deterministic-echo --expected-step-count 2
cargo run --release -p vogon-cli -- run --cache-file target/vogon-cache-smoke.cache.json --cache-max-entries 1 fixtures/workflows/support-triage.toml
python scripts/check_cache_json.py target/vogon-cache-smoke.cache.json --expected-max-entries 1 --expected-entry-count 1
cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force
target/install-smoke/bin/vogon --version
target/install-smoke/bin/vogon doctor --json | python scripts/check_doctor_json.py
target/install-smoke/bin/vogon providers --json | python scripts/check_providers_json.py
target/install-smoke/bin/vogon init --force --output target/install-smoke-workflow.toml
target/install-smoke/bin/vogon check --json target/install-smoke-workflow.toml | python scripts/check_workflow_json.py --expected-workflow-name starter-workflow --expected-step-count 2
target/install-smoke/bin/vogon check --json fixtures/workflows/support-triage.toml | python scripts/check_workflow_json.py --expected-workflow-name support-triage --expected-step-count 2
target/install-smoke/bin/vogon run --cache-file target/install-smoke.cache.json --cache-max-entries 1 fixtures/workflows/support-triage.toml
python scripts/check_cache_json.py target/install-smoke.cache.json --expected-max-entries 1 --expected-entry-count 1
target/install-smoke/bin/vogon verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
target/install-smoke/bin/vogon verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json
docker build --tag vogon-runtime:smoke .
python scripts/check_container_image.py vogon-runtime:smoke
docker run --rm vogon-runtime:smoke --version
docker run --rm --read-only vogon-runtime:smoke --version
docker run --rm --read-only vogon-runtime:smoke doctor --json | python scripts/check_doctor_json.py
docker run --rm --read-only vogon-runtime:smoke providers --json | python scripts/check_providers_json.py
mkdir -p target/container-smoke
chmod 777 target/container-smoke
docker run --rm --read-only -v "$PWD/target/container-smoke:/work" vogon-runtime:smoke init --force --output /work/starter.toml
docker run --rm --read-only -v "$PWD/target/container-smoke:/work:ro" vogon-runtime:smoke check --json /work/starter.toml | python scripts/check_workflow_json.py --expected-workflow-name starter-workflow --expected-step-count 2
docker run --rm --read-only -v "$PWD:/work:ro" vogon-runtime:smoke check --json fixtures/workflows/support-triage.toml | python scripts/check_workflow_json.py --expected-workflow-name support-triage --expected-step-count 2
docker run --rm --read-only -v "$PWD:/work:ro" vogon-runtime:smoke verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json | python scripts/check_verify_json.py --expected-workflow-name support-triage --expect-match
docker run --rm --read-only -v "$PWD:/work:ro" vogon-runtime:smoke trace --jsonl fixtures/replays/support-triage.replay.json | python scripts/check_trace_jsonl.py --expected-provider deterministic --expected-model deterministic-echo --expected-step-count 2
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo package -p vogon-core --allow-dirty --offline --locked
cargo package --workspace --allow-dirty --no-verify --offline --locked
```

The package command validates the crate archives that would be prepared for
publication. Use `--offline` when working without registry access after
dependencies have already been fetched.

The Docker smoke commands require a running Docker daemon. They are also
enforced by the `Container image smoke` CI job.

Use `.env.example` as the public list of provider credential variables. Keep
all values blank in the committed example, and do not commit local `.env`
files.

CI also scans tracked text files for common committed secret patterns. The
scanner is intentionally conservative and complements manual review; it is not
a substitute for removing private prompts, credentials, or sensitive replay
data before opening a pull request.

### Windows target file locks

On Windows, local antivirus or indexers can briefly hold files in `target` while
Cargo is rebuilding. If a check fails with `os error 32` while removing an
object file, rerun the same command with a single build job:

```powershell
$env:CARGO_BUILD_JOBS='1'
cargo test --workspace --all-features --locked
```

Use the same environment variable for `cargo build` or other Cargo commands
that hit the same file-lock error.

## Pull Requests

- Keep changes focused and reviewable.
- Add or update tests for behavior changes.
- Update docs when user-facing behavior changes.
- Keep provider-specific code outside `vogon-core`.
- Do not commit secrets, credentials, or real customer data.

The `main` branch is protected. Pull requests must pass:

- `Rust workspace`
- `Container image smoke`
- `Windows release smoke`
- `Dependency review`
- `CodeQL Rust analysis`
- `Minimum supported Rust`
- `RustSec advisory audit` when Rust dependency files or the audit workflow
  change

The optional `Live Gemini Smoke` workflow can be run manually by maintainers
when `GEMINI_API_KEY` is configured as a repository or environment secret. It is
not required for ordinary pull requests because deterministic CI must not
depend on external provider availability.

The optional `Live Groq Smoke` workflow can be run manually by maintainers when
`GROQ_API_KEY` is configured. Use it after changes that affect Groq provider
configuration or OpenAI-compatible provider behavior.

The optional `Live Hugging Face Smoke` workflow can be run manually by
maintainers when `HF_TOKEN` is configured. Use it after changes that affect
Hugging Face provider configuration or OpenAI-compatible provider behavior.

The optional `Live OpenAI-Compatible Smoke` workflow can be run manually by
maintainers when `OPENAI_COMPATIBLE_API_KEY` is configured. Use it after changes
that affect generic OpenAI-compatible provider configuration, custom base URL
handling, or model override behavior.

The optional `Live OpenRouter Smoke` workflow can be run manually by
maintainers when `OPENROUTER_API_KEY` is configured. Use it after changes that
affect OpenRouter provider configuration or OpenAI-compatible provider
behavior.

Maintainers merge accepted pull requests with regular merge commits. Squash
merges are not used for this repository, and merged topic branches may remain
available for auditability.

## Commit Style

Use short imperative commit messages, for example:

```text
Add workspace scaffold
Implement deterministic echo adapter
Document replay format
```
