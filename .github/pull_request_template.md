## Summary

-

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo check -p vogon-cli --no-default-features`
- [ ] `python -m unittest scripts.test_write_spdx_sbom`
- [ ] `python -m unittest scripts.test_check_benchmark_output`
- [ ] `python -m unittest scripts.test_check_cargo_manifests`
- [ ] `python -m unittest scripts.test_check_changelog`
- [ ] `python -m unittest scripts.test_check_contributing_checklist`
- [ ] `python -m unittest scripts.test_check_container_policy`
- [ ] `python -m unittest scripts.test_check_deployment_checklist`
- [ ] `python -m unittest scripts.test_check_docs_links`
- [ ] `python -m unittest scripts.test_check_env_example`
- [ ] `python -m unittest scripts.test_check_issue_templates`
- [ ] `python -m unittest scripts.test_check_live_replay`
- [ ] `python -m unittest scripts.test_check_live_workflows`
- [ ] `python -m unittest scripts.test_check_package_verification_docs`
- [ ] `python -m unittest scripts.test_check_pr_template`
- [ ] `python -m unittest scripts.test_check_release_checklist`
- [ ] `python -m unittest scripts.test_check_release_workflow`
- [ ] `python -m unittest scripts.test_check_secrets`
- [ ] `python -m unittest scripts.test_check_workflow_policies`
- [ ] `python scripts/check_cargo_manifests.py --root .`
- [ ] `python scripts/check_changelog.py --root .`
- [ ] `python scripts/check_contributing_checklist.py --root .`
- [ ] `python scripts/check_deployment_checklist.py --root .`
- [ ] `python scripts/check_docs_links.py --root .`
- [ ] `python scripts/check_env_example.py --root .`
- [ ] `python scripts/check_issue_templates.py --root .`
- [ ] `python scripts/check_container_policy.py --root .`
- [ ] `python scripts/check_live_workflows.py --root .`
- [ ] `python scripts/check_package_verification_docs.py --root .`
- [ ] `python scripts/check_pr_template.py --root .`
- [ ] `python scripts/check_release_checklist.py --root .`
- [ ] `python scripts/check_release_workflow.py --root .`
- [ ] `python scripts/check_secrets.py --root .`
- [ ] `python scripts/check_workflow_policies.py --root .`
- [ ] `cargo +1.85.0 test --workspace --all-features --locked`
- [ ] `cargo bench -p vogon-core --bench runtime -- --iterations 100 | python scripts/check_benchmark_output.py --expected-iterations 100`
- [ ] `cargo build --release --workspace --all-features`
- [ ] `cargo run --release -p vogon-cli -- doctor --json`
- [ ] `cargo run --release -p vogon-cli -- init --force --output target/vogon-init-smoke/workflow.toml`
- [ ] `cargo run --release -p vogon-cli -- check --json target/vogon-init-smoke/workflow.toml`
- [ ] `cargo run --release -p vogon-cli -- check --json fixtures/workflows/support-triage.toml`
- [ ] `cargo run --release -p vogon-cli -- verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json`
- [ ] `cargo run --release -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json`
- [ ] `cargo run --release -p vogon-cli -- verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json`
- [ ] `cargo run --release -p vogon-cli -- trace --jsonl fixtures/replays/support-triage.replay.json`
- [ ] `cargo run --release -p vogon-cli -- run --cache-file target/vogon-cache-smoke.cache.json --cache-max-entries 1 fixtures/workflows/support-triage.toml`
- [ ] `python -c "import json; data = json.load(open('target/vogon-cache-smoke.cache.json', encoding='utf-8')); assert data['max_entries'] == 1; assert len(data['outputs']) == 1; assert len(data['insertion_order']) == 1"`
- [ ] `cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force`
- [ ] `target/install-smoke/bin/vogon --version`
- [ ] `target/install-smoke/bin/vogon doctor --json`
- [ ] `target/install-smoke/bin/vogon init --force --output target/install-smoke-workflow.toml`
- [ ] `target/install-smoke/bin/vogon check --json target/install-smoke-workflow.toml`
- [ ] `target/install-smoke/bin/vogon check --json fixtures/workflows/support-triage.toml`
- [ ] `target/install-smoke/bin/vogon run --cache-file target/install-smoke.cache.json --cache-max-entries 1 fixtures/workflows/support-triage.toml`
- [ ] `python -c "import json; data = json.load(open('target/install-smoke.cache.json', encoding='utf-8')); assert data['max_entries'] == 1; assert len(data['outputs']) == 1; assert len(data['insertion_order']) == 1"`
- [ ] `target/install-smoke/bin/vogon verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json`
- [ ] `target/install-smoke/bin/vogon verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json`
- [ ] `docker build --tag vogon-runtime:smoke .`
- [ ] `test "$(docker image inspect vogon-runtime:smoke --format '{{ index .Config.Labels "org.opencontainers.image.source" }}')" = "https://github.com/kaleab-kali/vogon-runtime"`
- [ ] `test "$(docker image inspect vogon-runtime:smoke --format '{{ index .Config.Labels "org.opencontainers.image.licenses" }}')" = "MIT"`
- [ ] `docker run --rm vogon-runtime:smoke --version`
- [ ] `test "$(docker run --rm --entrypoint id vogon-runtime:smoke -u)" = "10001"`
- [ ] `docker run --rm --read-only vogon-runtime:smoke --version`
- [ ] `docker run --rm --read-only vogon-runtime:smoke doctor --json`
- [ ] `mkdir -p target/container-smoke`
- [ ] `chmod 777 target/container-smoke`
- [ ] `docker run --rm --read-only -v "$PWD/target/container-smoke:/work" vogon-runtime:smoke init --force --output /work/starter.toml`
- [ ] `docker run --rm --read-only -v "$PWD/target/container-smoke:/work:ro" vogon-runtime:smoke check --json /work/starter.toml`
- [ ] `docker run --rm --read-only -v "$PWD:/work:ro" vogon-runtime:smoke check --json fixtures/workflows/support-triage.toml`
- [ ] `docker run --rm --read-only -v "$PWD:/work:ro" vogon-runtime:smoke verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json`
- [ ] `docker run --rm --read-only -v "$PWD:/work:ro" vogon-runtime:smoke trace --jsonl fixtures/replays/support-triage.replay.json`
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps`
- [ ] `cargo package --workspace --allow-dirty --no-verify --offline`
- [ ] Relevant CLI smoke test:
- [ ] RustSec advisory audit passed or is not affected by this change.

## Checklist

- [ ] The change is focused and reviewable.
- [ ] Tests were added or updated for behavior changes.
- [ ] Public docs were updated for user-facing behavior changes.
- [ ] Provider-specific code remains outside `vogon-core`.
- [ ] No secrets, credentials, private prompts, or sensitive replay data are included.

## Notes

-
