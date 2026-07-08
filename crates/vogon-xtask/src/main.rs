use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Read as _};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use toml::Value;

type TomlTable = toml::Table;

const EXPECTED_ENV_VARS: &[&str] = &[
    "GEMINI_API_KEY",
    "GROQ_API_KEY",
    "HF_TOKEN",
    "OPENAI_COMPATIBLE_API_KEY",
    "OPENROUTER_API_KEY",
];
const REPO_OWNER: &str = "kaleab-kali";
const REPO_NAME: &str = "vogon-runtime";
const MARKDOWN_SUFFIXES: &[&str] = &["md", "markdown"];
const BUG_ISSUE_REQUIRED_FIELDS: &[&str] = &[
    "actual",
    "checks",
    "component",
    "environment",
    "expected",
    "reproduce",
    "version",
];
const FEATURE_ISSUE_REQUIRED_FIELDS: &[&str] = &["area", "checks", "problem", "proposal"];
const REQUIRED_ISSUE_AREAS: &[&str] = &[
    "CLI",
    "Documentation",
    "Other",
    "Provider adapter",
    "Release artifact",
    "Replay verification",
    "Runtime",
];
const REQUIRED_ISSUE_CHECK_LABELS: &[&str] = &["removed secrets", "searched existing issues"];
const REQUIRED_BUG_VERSION_PLACEHOLDER: &str = "placeholder: \"vogon 0.1.1\"";
const MAX_SECRET_SCAN_TEXT_BYTES: u64 = 1_000_000;
const SENSITIVE_ARTIFACT_SUFFIXES: &[&str] = &[".cache.json"];
const PROVIDER_CREDENTIAL_VARS: &[&str] = &[
    "GEMINI_API_KEY",
    "GROQ_API_KEY",
    "HF_TOKEN",
    "OPENAI_COMPATIBLE_API_KEY",
    "OPENROUTER_API_KEY",
];
const PLACEHOLDER_VALUES: &[&str] = &[
    "",
    "...",
    "''",
    "\"\"",
    "<token>",
    "<api-key>",
    "<api_key>",
    "<secret>",
    "changeme",
    "change-me",
    "your-token",
    "your-api-key",
    "your_api_key",
];
const README_LOCAL_CHECKS_MARKER: &str = "Run local checks:";
const CONTRIBUTING_DEVELOPMENT_MARKER: &str = "## Development";
const RELEASE_VERIFICATION_MARKER: &str = "Run the full local verification set:";
const DEPLOYMENT_SMOKE_MARKER: &str = "Before publishing or deploying an image, run:";
const LIVE_WORKFLOW_GUIDANCE: &[(&str, &str)] = &[
    ("Live Gemini Smoke", "GEMINI_API_KEY"),
    ("Live Groq Smoke", "GROQ_API_KEY"),
    ("Live Hugging Face Smoke", "HF_TOKEN"),
    ("Live OpenAI-Compatible Smoke", "OPENAI_COMPATIBLE_API_KEY"),
    ("Live OpenRouter Smoke", "OPENROUTER_API_KEY"),
];
const PROVIDER_CREDENTIALS_MARKER: &str = "## Provider Credentials";
const DEPLOYMENT_PROVIDER_EXAMPLES: &[(&str, &str)] = &[
    ("gemini", "GEMINI_API_KEY"),
    ("openai-compatible", "OPENAI_COMPATIBLE_API_KEY"),
    ("groq", "GROQ_API_KEY"),
    ("hugging-face", "HF_TOKEN"),
    ("openrouter", "OPENROUTER_API_KEY"),
];
const EXPECTED_PROVIDER_JSON: &[ProviderJsonExpectation] = &[
    ProviderJsonExpectation {
        name: "deterministic",
        default_provider: true,
        credential_env: None,
        credential_configured: ExpectedProviderJsonValue::Null,
        default_base_url: None,
        default_model: None,
        documentation_url: Some(
            "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#deterministic",
        ),
        usage_url: None,
    },
    ProviderJsonExpectation {
        name: "gemini",
        default_provider: false,
        credential_env: Some("GEMINI_API_KEY"),
        credential_configured: ExpectedProviderJsonValue::BoolOrNull,
        default_base_url: None,
        default_model: Some("gemini-3.1-flash-lite"),
        documentation_url: Some("https://ai.google.dev/gemini-api/docs"),
        usage_url: Some("https://ai.google.dev/gemini-api/docs/pricing"),
    },
    ProviderJsonExpectation {
        name: "groq",
        default_provider: false,
        credential_env: Some("GROQ_API_KEY"),
        credential_configured: ExpectedProviderJsonValue::BoolOrNull,
        default_base_url: Some("https://api.groq.com/openai/v1"),
        default_model: Some("llama-3.1-8b-instant"),
        documentation_url: Some("https://console.groq.com/docs/openai"),
        usage_url: Some("https://console.groq.com/docs/rate-limits"),
    },
    ProviderJsonExpectation {
        name: "hugging-face",
        default_provider: false,
        credential_env: Some("HF_TOKEN"),
        credential_configured: ExpectedProviderJsonValue::BoolOrNull,
        default_base_url: Some("https://router.huggingface.co/v1"),
        default_model: Some("openai/gpt-oss-120b:fastest"),
        documentation_url: Some("https://huggingface.co/docs/inference-providers"),
        usage_url: Some("https://huggingface.co/docs/inference-providers/pricing"),
    },
    ProviderJsonExpectation {
        name: "openrouter",
        default_provider: false,
        credential_env: Some("OPENROUTER_API_KEY"),
        credential_configured: ExpectedProviderJsonValue::BoolOrNull,
        default_base_url: Some("https://openrouter.ai/api/v1"),
        default_model: Some("openrouter/free"),
        documentation_url: Some("https://openrouter.ai/docs"),
        usage_url: Some("https://openrouter.ai/pricing"),
    },
    ProviderJsonExpectation {
        name: "openai-compatible",
        default_provider: false,
        credential_env: Some("OPENAI_COMPATIBLE_API_KEY"),
        credential_configured: ExpectedProviderJsonValue::BoolOrNull,
        default_base_url: Some("https://router.huggingface.co/v1"),
        default_model: Some("openai/gpt-oss-120b:fastest"),
        documentation_url: Some(
            "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#openai-compatible",
        ),
        usage_url: None,
    },
];
const REQUIRED_README_COMMANDS: &[&str] = &[];
const DEFAULT_ARCHIVE_REQUIRED_FILES: &[&str] = &["README.md", "LICENSE"];
const REQUIRED_BENCHMARK_METRICS: &[&str] = &["elapsed_ms", "iterations", "iterations_per_second"];
const WORKFLOW_SUFFIXES: &[&str] = &["yml", "yaml"];
const ALLOWED_TOP_LEVEL_WRITE_SCOPES: &[&str] = &["security-events"];
const FLOATING_RUNNERS: &[&str] = &["ubuntu-latest", "windows-latest", "macos-latest"];
const MUTABLE_ACTION_REFS: &[&str] = &["main", "master", "latest", "head", "trunk"];
const REQUIRED_CI_WORKFLOW_SNIPPETS: &[(&str, &str)] = &[
    ("workflow name", "name: CI"),
    ("pull request trigger", "  pull_request:"),
    ("push main trigger", "  push:\n    branches:\n      - main"),
    (
        "read-only contents permission",
        "permissions:\n  contents: read",
    ),
    (
        "concurrency group",
        "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
    ),
    ("stale run cancellation", "  cancel-in-progress: true"),
    ("cargo network retry env", "env:\n  CARGO_NET_RETRY: 10"),
    ("Rust workspace job", "  rust:"),
    ("MSRV job", "  msrv:"),
    ("container smoke job", "  container-smoke:"),
    ("Windows release smoke job", "  windows-release-smoke:"),
    ("Rust workspace runner", "    runs-on: ubuntu-24.04"),
    ("Windows runner", "    runs-on: windows-2025-vs2026"),
    ("Rust workspace timeout", "    timeout-minutes: 30"),
    ("MSRV timeout", "    timeout-minutes: 20"),
    ("checkout action", "uses: actions/checkout@v7"),
    (
        "CI workflow validator",
        "cargo run -p vogon-xtask -- check-ci-workflow --root .",
    ),
    (
        "workflow policy validator",
        "cargo run -p vogon-xtask -- check-workflow-policies --root .",
    ),
    (
        "security workflow validator",
        "cargo run -p vogon-xtask -- check-security-workflows --root .",
    ),
    (
        "container policy validator",
        "cargo run -p vogon-xtask -- check-container-policy --root .",
    ),
    (
        "committed secret validator",
        "cargo run -p vogon-xtask -- check-secrets --root .",
    ),
    (
        "release workflow validator",
        "python3 scripts/check_release_workflow.py --root .",
    ),
    (
        "changelog validator",
        "cargo run -p vogon-xtask -- check-changelog --root .",
    ),
    (
        "contributing checklist validator",
        "cargo run -p vogon-xtask -- check-contributing-checklist --root .",
    ),
    (
        "deployment checklist validator",
        "cargo run -p vogon-xtask -- check-deployment-checklist --root .",
    ),
    (
        "deployment docs validator",
        "cargo run -p vogon-xtask -- check-deployment-docs --root .",
    ),
    (
        "pull request template validator",
        "cargo run -p vogon-xtask -- check-pr-template --root .",
    ),
    (
        "documentation link checker",
        "cargo run -p vogon-xtask -- check-docs-links --root .",
    ),
    (
        "issue template validator",
        "cargo run -p vogon-xtask -- check-issue-templates --root .",
    ),
    (
        "release checklist validator",
        "cargo run -p vogon-xtask -- check-release-checklist --root .",
    ),
    (
        "Cargo manifest validator",
        "cargo run -p vogon-xtask -- check-cargo-manifests --root .",
    ),
    (
        "provider env example validator",
        "cargo run -p vogon-xtask -- check-env-example --root .",
    ),
    (
        "Dependabot configuration validator",
        "cargo run -p vogon-xtask -- check-dependabot-config --root .",
    ),
    (
        "public status docs validator",
        "cargo run -p vogon-xtask -- check-public-status-docs --root .",
    ),
    (
        "package verification docs validator",
        "cargo run -p vogon-xtask -- check-package-verification-docs --root .",
    ),
    (
        "live workflow validator",
        "python3 scripts/check_live_workflows.py --root .",
    ),
    ("format check", "cargo fmt --all -- --check"),
    (
        "clippy check",
        "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
    ),
    (
        "workspace tests",
        "cargo test --workspace --all-features --locked",
    ),
    (
        "deterministic-only CLI build",
        "cargo check -p vogon-cli --no-default-features --locked",
    ),
    (
        "MSRV test",
        "cargo +1.85.0 test --workspace --all-features --locked",
    ),
    (
        "benchmark smoke",
        "cargo bench -p vogon-core --bench runtime --locked -- --iterations 100",
    ),
    (
        "benchmark output validator",
        "cargo run -p vogon-xtask -- check-benchmark-output --expected-iterations 100",
    ),
    (
        "release build",
        "cargo build --release --workspace --all-features --locked",
    ),
    (
        "release CLI doctor smoke",
        "./target/release/vogon doctor --json",
    ),
    (
        "doctor JSON validator",
        "cargo run -p vogon-xtask -- check-doctor-json",
    ),
    (
        "release CLI providers smoke",
        "./target/release/vogon providers --json",
    ),
    (
        "providers JSON validator",
        "cargo run -p vogon-xtask -- check-providers-json",
    ),
    (
        "release replay verification smoke",
        "./target/release/vogon verify",
    ),
    (
        "workflow check JSON validator",
        "cargo run -p vogon-xtask -- check-workflow-json",
    ),
    (
        "offline install smoke",
        "cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force",
    ),
    ("rustdoc warnings denied", "RUSTDOCFLAGS: -D warnings"),
    (
        "core package verification",
        "cargo package -p vogon-core --allow-dirty --offline --locked",
    ),
    (
        "workspace package smoke",
        "cargo package --workspace --allow-dirty --no-verify --offline --locked",
    ),
    (
        "container build smoke",
        "docker build --tag vogon-runtime:ci .",
    ),
    ("read-only container smoke", "docker run --rm --read-only"),
    (
        "Windows release build",
        "cargo build --release -p vogon-cli --locked",
    ),
    (
        "Windows replay verification smoke",
        ".\\target\\release\\vogon.exe verify",
    ),
];
const REQUIRED_CI_WORKFLOW_COUNTS: &[(&str, usize)] = &[
    ("uses: actions/checkout@v7", 4),
    ("runs-on: ubuntu-24.04", 3),
    ("timeout-minutes: 30", 3),
];
const SECURITY_WORKFLOW_REQUIREMENTS: &[(&str, &[(&str, &str)])] = &[
    (
        ".github/workflows/codeql.yml",
        &[
            ("workflow name", "name: CodeQL"),
            ("pull request trigger", "  pull_request:"),
            ("push main trigger", "  push:"),
            ("scheduled scan", r#"    - cron: "31 5 * * 2""#),
            ("manual dispatch trigger", "  workflow_dispatch:"),
            ("read-only contents permission", "  contents: read"),
            (
                "security events write permission",
                "  security-events: write",
            ),
            (
                "concurrency group",
                "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
            ),
            ("stale run cancellation", "  cancel-in-progress: true"),
            ("cargo network retry env", "env:\n  CARGO_NET_RETRY: 10"),
            ("ubuntu runner", "    runs-on: ubuntu-24.04"),
            ("job timeout", "    timeout-minutes: 30"),
            ("checkout action", "uses: actions/checkout@v7"),
            ("CodeQL init action", "uses: github/codeql-action/init@v4"),
            ("Rust language configuration", "          languages: rust"),
            ("no-build analysis mode", "          build-mode: none"),
            (
                "extended security queries",
                "          queries: security-extended,security-and-quality",
            ),
            (
                "CodeQL analyze action",
                "uses: github/codeql-action/analyze@v4",
            ),
        ],
    ),
    (
        ".github/workflows/security-audit.yml",
        &[
            ("workflow name", "name: Security Audit"),
            ("pull request trigger", "  pull_request:"),
            ("push main trigger", "  push:"),
            ("scheduled audit", r#"    - cron: "17 4 * * 1""#),
            ("manual dispatch trigger", "  workflow_dispatch:"),
            (
                "read-only contents permission",
                "permissions:\n  contents: read",
            ),
            (
                "concurrency group",
                "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
            ),
            ("dependency lockfile path", "      - Cargo.lock"),
            ("workspace manifest path", "      - Cargo.toml"),
            ("crate manifest path", r#"      - "crates/**/Cargo.toml""#),
            (
                "audit workflow path",
                "      - .github/workflows/security-audit.yml",
            ),
            ("ubuntu runner", "    runs-on: ubuntu-24.04"),
            ("job timeout", "    timeout-minutes: 10"),
            ("checkout action", "uses: actions/checkout@v7"),
            ("RustSec audit action", "uses: actions-rust-lang/audit@v1"),
            ("no issue creation", "          createIssues: false"),
        ],
    ),
    (
        ".github/workflows/dependency-review.yml",
        &[
            ("workflow name", "name: Dependency Review"),
            ("pull request trigger", "  pull_request:"),
            (
                "read-only contents permission",
                "permissions:\n  contents: read",
            ),
            (
                "concurrency group",
                "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
            ),
            ("stale run cancellation", "  cancel-in-progress: true"),
            ("ubuntu runner", "    runs-on: ubuntu-24.04"),
            ("job timeout", "    timeout-minutes: 10"),
            ("checkout action", "uses: actions/checkout@v7"),
            (
                "dependency review action",
                "uses: actions/dependency-review-action@v5",
            ),
            (
                "dependency review config file",
                "          config-file: ./.github/dependency-review-config.yml",
            ),
        ],
    ),
];
const DEPENDENCY_REVIEW_CONFIG_REQUIREMENTS: &[(&str, &str)] = &[
    ("high severity failure", "fail-on-severity: high"),
    ("license checks enabled", "license-check: true"),
    ("vulnerability checks enabled", "vulnerability-check: true"),
    ("license allowlist", "allow-licenses:"),
    ("Apache license allowed", "  - Apache-2.0"),
    ("BSD license allowed", "  - BSD-3-Clause"),
    ("CDLA permissive license allowed", "  - CDLA-Permissive-2.0"),
    ("ISC license allowed", "  - ISC"),
    ("MIT license allowed", "  - MIT"),
    ("Unicode license allowed", "  - Unicode-3.0"),
    ("Unlicense allowed", "  - Unlicense"),
];
const ALLOWED_UNRELEASED_CHANGELOG_SECTIONS: &[&str] = &[
    "Added",
    "Changed",
    "Deprecated",
    "Removed",
    "Fixed",
    "Security",
    "Documentation",
];
const EXPECTED_WORKSPACE_PACKAGE: &[(&str, ExpectedValue)] = &[
    ("edition", ExpectedValue::String("2024")),
    ("rust-version", ExpectedValue::String("1.85")),
    ("license", ExpectedValue::String("MIT")),
    (
        "repository",
        ExpectedValue::String("https://github.com/kaleab-kali/vogon-runtime"),
    ),
    (
        "homepage",
        ExpectedValue::String("https://github.com/kaleab-kali/vogon-runtime"),
    ),
    (
        "documentation",
        ExpectedValue::String("https://github.com/kaleab-kali/vogon-runtime/tree/main/docs"),
    ),
    (
        "authors",
        ExpectedValue::StringList(&["Vogon Runtime Contributors"]),
    ),
];
const REQUIRED_PACKAGE_FIELDS: &[&str] = &[
    "authors",
    "categories",
    "description",
    "documentation",
    "edition",
    "homepage",
    "keywords",
    "license",
    "name",
    "readme",
    "repository",
    "rust-version",
    "version",
];
const EXPECTED_CRATES: &[(&str, &str)] = &[
    ("vogon-adapters", "crates/vogon-adapters"),
    ("vogon-cli", "crates/vogon-cli"),
    ("vogon-core", "crates/vogon-core"),
    ("vogon-xtask", "crates/vogon-xtask"),
];
const SCHEMA_DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";
const IDENTIFIER_PATTERN_DESCRIPTION: &str = "ASCII letters, digits, underscores, and hyphens";
const EXPECTED_SCHEMAS: &[(&str, &str, &[&str])] = &[
    (
        "schemas/workflow.schema.json",
        "Vogon Workflow",
        &["name", "steps"],
    ),
    (
        "schemas/replay.schema.json",
        "Vogon Replay",
        &[
            "schema_version",
            "workflow_name",
            "runtime",
            "run_hash",
            "steps",
        ],
    ),
];
const EXPECTED_RELEASE_PROFILE: &[(&str, ExpectedValue)] = &[
    ("codegen-units", ExpectedValue::Integer(1)),
    ("lto", ExpectedValue::String("thin")),
    ("strip", ExpectedValue::String("symbols")),
];
const EXPECTED_WORKSPACE_RUST_LINTS: &[(&str, ExpectedValue)] =
    &[("unsafe_code", ExpectedValue::String("forbid"))];
const REQUIRED_PUBLIC_STATUS_SNIPPETS: &[(&str, &[&str])] = &[
    (
        "README.md",
        &[
            "Vogon Runtime's latest public release is `v0.1.1`; `v0.1.0` was the first\npublic release.",
            "The project is still in the `0.x` series, so command and\nlibrary APIs may change",
        ],
    ),
    (
        "SECURITY.md",
        &[
            "`v0.1.1` is the latest public release of Vogon Runtime; `v0.1.0` was the first\npublic release.",
            "shipped in\nfollow-up patch or minor releases",
        ],
    ),
    (
        "SUPPORT.md",
        &["Vogon Runtime is released open-source software in the `0.x` series."],
    ),
    (
        "CHANGELOG.md",
        &[
            "and this project follows semantic versioning.",
            "## [0.1.1] - 2026-07-08",
            "## [0.1.0] - 2026-07-08",
        ],
    ),
    ("docs/release.md", &["still in the `0.x` series"]),
];
const STALE_PUBLIC_STATUS_PHRASES: &[&str] = &[
    "Vogon Runtime is pre-release",
    "has not published a stable release yet",
    "until `v0.1.0` is tagged",
    "Vogon Runtime has a first public release, `v0.1.0`.",
    "once the first release is tagged",
    "public API is\npre-release",
];
const PACKAGE_VERIFICATION_COMMAND: &str =
    "cargo package --workspace --allow-dirty --no-verify --offline --locked";
const PACKAGE_VERIFICATION_RATIONALE_SNIPPETS: &[&str] = &[
    "Cargo can fail offline verification while resolving unpublished internal workspace crates",
    "preceding build, test, docs, install, and smoke commands",
];
const PACKAGE_VERIFICATION_DOCS: &[&str] = &["README.md", "docs/release.md"];
const REQUIRED_DOCKERIGNORE_ENTRIES: &[&str] = &[
    "/.git",
    "/.github",
    "/target",
    ".env",
    ".env.*",
    "!.env.example",
    "__pycache__/",
    "*.py[cod]",
    "*.cache.json",
];
const REQUIRED_DOCKERFILE_SNIPPETS: &[(&str, &str)] = &[
    (
        "current Rust build image",
        "FROM rust:1.96.1-bookworm AS build",
    ),
    (
        "cargo incremental builds disabled",
        "ENV CARGO_INCREMENTAL=0",
    ),
    ("cargo network retries configured", "ENV CARGO_NET_RETRY=10"),
    ("runtime stage", "FROM debian:bookworm-slim AS runtime"),
    (
        "minimal certificate install",
        "apt-get install -y --no-install-recommends ca-certificates",
    ),
    (
        "OCI title label",
        "org.opencontainers.image.title=\"Vogon Runtime\"",
    ),
    (
        "OCI description label",
        "org.opencontainers.image.description=\"Deterministic, replayable AI workflow runtime CLI.\"",
    ),
    (
        "OCI source label",
        "org.opencontainers.image.source=\"https://github.com/kaleab-kali/vogon-runtime\"",
    ),
    (
        "OCI documentation label",
        "org.opencontainers.image.documentation=\"https://github.com/kaleab-kali/vogon-runtime#readme\"",
    ),
    (
        "OCI license label",
        "org.opencontainers.image.licenses=\"MIT\"",
    ),
    (
        "OCI version label",
        "org.opencontainers.image.version=\"${VOGON_IMAGE_VERSION}\"",
    ),
    (
        "OCI revision label",
        "org.opencontainers.image.revision=\"${VOGON_IMAGE_REVISION}\"",
    ),
    (
        "default image version argument",
        "ARG VOGON_IMAGE_VERSION=dev",
    ),
    (
        "default image revision argument",
        "ARG VOGON_IMAGE_REVISION=unknown",
    ),
    ("apt package list cleanup", "rm -rf /var/lib/apt/lists/*"),
    (
        "non-root runtime user",
        "useradd --create-home --uid 10001 vogon",
    ),
    (
        "release binary copy",
        "COPY --from=build /workspace/target/release/vogon /usr/local/bin/vogon",
    ),
    ("non-root user activation", "USER vogon"),
    ("runtime workdir", "WORKDIR /work"),
    ("exec entrypoint", "ENTRYPOINT [\"vogon\"]"),
];
const EXPECTED_DEPENDABOT_UPDATES: &[(&str, &[(&str, &str)])] = &[
    (
        "cargo",
        &[
            ("directory", "/"),
            ("interval", "weekly"),
            ("open-pull-requests-limit", "5"),
            ("groups.cargo-minor-patch.patterns", "*"),
            ("groups.cargo-minor-patch.update-types", "minor,patch"),
            ("commit-message.prefix", "deps"),
        ],
    ),
    (
        "github-actions",
        &[
            ("directory", "/"),
            ("interval", "weekly"),
            ("open-pull-requests-limit", "5"),
            ("groups.github-actions-minor-patch.patterns", "*"),
            (
                "groups.github-actions-minor-patch.update-types",
                "minor,patch",
            ),
            ("commit-message.prefix", "ci"),
        ],
    ),
    (
        "docker",
        &[
            ("directory", "/"),
            ("interval", "weekly"),
            ("open-pull-requests-limit", "5"),
            ("groups.docker-minor-patch.patterns", "*"),
            ("groups.docker-minor-patch.update-types", "minor,patch"),
            ("commit-message.prefix", "deps"),
        ],
    ),
];

#[derive(Clone, Copy)]
enum ExpectedValue {
    Integer(i64),
    String(&'static str),
    StringList(&'static [&'static str]),
}

#[derive(Clone, Copy)]
enum ExpectedProviderJsonValue {
    Null,
    BoolOrNull,
}

struct ProviderJsonExpectation {
    name: &'static str,
    default_provider: bool,
    credential_env: Option<&'static str>,
    credential_configured: ExpectedProviderJsonValue,
    default_base_url: Option<&'static str>,
    default_model: Option<&'static str>,
    documentation_url: Option<&'static str>,
    usage_url: Option<&'static str>,
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage_and_exit();
    };

    let result = match command.as_str() {
        "check-archive-contents" => {
            let options = parse_archive_contents_args(args.collect());
            check_archive_contents(
                &options.archive_directory,
                &options.binary,
                &options.required_files,
            )
        }
        "check-benchmark-output" => {
            let options = parse_benchmark_output_args(args.collect());
            check_benchmark_output_from_stdin(options.expected_iterations)
        }
        "check-env-example" => {
            let root = parse_root(args.collect());
            check_env_example(&root)
        }
        "check-cargo-manifests" => {
            let root = parse_root(args.collect());
            check_cargo_manifests(&root)
        }
        "check-cargo-metadata-json" => {
            let options = parse_cargo_metadata_json_args(args.collect());
            check_cargo_metadata_json_file(&options.metadata_file, &options.expected_packages)
        }
        "check-ci-workflow" => {
            let root = parse_root(args.collect());
            check_ci_workflow(&root)
        }
        "check-changelog" => {
            let root = parse_root(args.collect());
            check_changelog(&root)
        }
        "check-container-policy" => {
            let root = parse_root(args.collect());
            check_container_policy(&root)
        }
        "check-dependabot-config" => {
            let root = parse_root(args.collect());
            check_dependabot_config(&root)
        }
        "check-docs-links" => {
            let root = parse_root(args.collect());
            check_docs_links(&root)
        }
        "check-issue-templates" => {
            let root = parse_root(args.collect());
            check_issue_templates(&root)
        }
        "check-contributing-checklist" => {
            let root = parse_root(args.collect());
            check_contributing_checklist(&root)
        }
        "check-deployment-checklist" => {
            let root = parse_root(args.collect());
            check_deployment_checklist(&root)
        }
        "check-deployment-docs" => {
            let root = parse_root(args.collect());
            check_deployment_docs(&root)
        }
        "check-doctor-json" => {
            ensure_no_args(args.collect());
            check_doctor_json_from_stdin()
        }
        "check-workflow-json" => {
            let options = parse_workflow_json_args(args.collect());
            check_workflow_json_from_stdin(&options)
        }
        "check-package-verification-docs" => {
            let root = parse_root(args.collect());
            check_package_verification_docs(&root)
        }
        "check-pr-template" => {
            let root = parse_root(args.collect());
            check_pr_template(&root)
        }
        "check-providers-json" => {
            ensure_no_args(args.collect());
            check_providers_json_from_stdin()
        }
        "check-public-status-docs" => {
            let root = parse_root(args.collect());
            check_public_status_docs(&root)
        }
        "check-release-checklist" => {
            let root = parse_root(args.collect());
            check_release_checklist(&root)
        }
        "check-schema-files" => {
            let root = parse_root(args.collect());
            check_schema_files(&root)
        }
        "check-sha256-file" => {
            let options = parse_sha256_file_args(args.collect());
            check_sha256_file(&options.artifact, options.checksum_file.as_deref())
        }
        "check-security-workflows" => {
            let root = parse_root(args.collect());
            check_security_workflows(&root)
        }
        "check-secrets" => {
            let root = parse_root(args.collect());
            check_secrets(&root)
        }
        "check-workflow-policies" => {
            let root = parse_root(args.collect());
            check_workflow_policies(&root)
        }
        _ => {
            eprintln!("unknown xtask command `{command}`");
            print_usage_and_exit();
        }
    };

    match result {
        Ok(()) => {}
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            std::process::exit(1);
        }
    }
}

fn parse_root(args: Vec<String>) -> PathBuf {
    match args.as_slice() {
        [] => env::current_dir().unwrap_or_else(|error| {
            eprintln!("failed to read current directory: {error}");
            std::process::exit(2);
        }),
        [flag, value] if flag == "--root" => PathBuf::from(value),
        _ => print_usage_and_exit(),
    }
}

fn ensure_no_args(args: Vec<String>) {
    if !args.is_empty() {
        print_usage_and_exit();
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "usage: cargo run -p vogon-xtask -- <check-archive-contents|check-benchmark-output|check-cargo-manifests|check-cargo-metadata-json|check-ci-workflow|check-changelog|check-container-policy|check-dependabot-config|check-docs-links|check-issue-templates|check-contributing-checklist|check-deployment-checklist|check-deployment-docs|check-doctor-json|check-env-example|check-package-verification-docs|check-pr-template|check-providers-json|check-public-status-docs|check-release-checklist|check-schema-files|check-security-workflows|check-secrets|check-sha256-file|check-workflow-json|check-workflow-policies> [--root PATH]"
    );
    std::process::exit(2);
}

struct BenchmarkOutputOptions {
    expected_iterations: i64,
}

struct CargoMetadataJsonOptions {
    metadata_file: PathBuf,
    expected_packages: Vec<String>,
}

struct WorkflowJsonOptions {
    expected_workflow_name: Option<String>,
    expected_step_count: Option<i64>,
}

struct Sha256FileOptions {
    artifact: PathBuf,
    checksum_file: Option<PathBuf>,
}

struct WorkflowBlock {
    line: usize,
    entries: BTreeMap<String, (String, usize)>,
}

struct WorkflowJob {
    name: String,
    line: usize,
    runs_on: Option<String>,
    runs_on_line: Option<usize>,
    timeout_minutes: Option<String>,
    timeout_line: Option<usize>,
}

fn parse_benchmark_output_args(args: Vec<String>) -> BenchmarkOutputOptions {
    match args.as_slice() {
        [flag, value] if flag == "--expected-iterations" => {
            let expected_iterations = value.parse::<i64>().unwrap_or_else(|_| {
                eprintln!("--expected-iterations must be an integer");
                std::process::exit(2);
            });
            if expected_iterations <= 0 {
                eprintln!("--expected-iterations must be greater than zero");
                std::process::exit(2);
            }
            BenchmarkOutputOptions {
                expected_iterations,
            }
        }
        _ => print_benchmark_output_usage_and_exit(),
    }
}

fn print_benchmark_output_usage_and_exit() -> ! {
    eprintln!(
        "usage: cargo run -p vogon-xtask -- check-benchmark-output --expected-iterations COUNT"
    );
    std::process::exit(2);
}

fn parse_cargo_metadata_json_args(args: Vec<String>) -> CargoMetadataJsonOptions {
    let Some((metadata_file, rest)) = args.split_first() else {
        print_cargo_metadata_json_usage_and_exit();
    };

    let mut expected_packages = Vec::new();
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--expected-workspace-package" => {
                let Some(value) = rest.get(index + 1) else {
                    print_cargo_metadata_json_usage_and_exit();
                };
                expected_packages.push(value.clone());
                index += 2;
            }
            _ => print_cargo_metadata_json_usage_and_exit(),
        }
    }

    CargoMetadataJsonOptions {
        metadata_file: PathBuf::from(metadata_file),
        expected_packages,
    }
}

fn print_cargo_metadata_json_usage_and_exit() -> ! {
    eprintln!(
        "usage: cargo run -p vogon-xtask -- check-cargo-metadata-json METADATA_FILE [--expected-workspace-package NAME ...]"
    );
    std::process::exit(2);
}

fn parse_workflow_json_args(args: Vec<String>) -> WorkflowJsonOptions {
    let mut expected_workflow_name = None;
    let mut expected_step_count = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--expected-workflow-name" => {
                let Some(value) = args.get(index + 1) else {
                    print_workflow_json_usage_and_exit();
                };
                expected_workflow_name = Some(value.clone());
                index += 2;
            }
            "--expected-step-count" => {
                let Some(value) = args.get(index + 1) else {
                    print_workflow_json_usage_and_exit();
                };
                let parsed = value.parse::<i64>().unwrap_or_else(|_| {
                    eprintln!("--expected-step-count must be an integer");
                    std::process::exit(2);
                });
                expected_step_count = Some(parsed);
                index += 2;
            }
            _ => print_workflow_json_usage_and_exit(),
        }
    }

    WorkflowJsonOptions {
        expected_workflow_name,
        expected_step_count,
    }
}

fn print_workflow_json_usage_and_exit() -> ! {
    eprintln!(
        "usage: cargo run -p vogon-xtask -- check-workflow-json [--expected-workflow-name NAME] [--expected-step-count COUNT]"
    );
    std::process::exit(2);
}

fn parse_sha256_file_args(args: Vec<String>) -> Sha256FileOptions {
    match args.as_slice() {
        [artifact] => Sha256FileOptions {
            artifact: PathBuf::from(artifact),
            checksum_file: None,
        },
        [artifact, checksum_file] => Sha256FileOptions {
            artifact: PathBuf::from(artifact),
            checksum_file: Some(PathBuf::from(checksum_file)),
        },
        _ => print_sha256_file_usage_and_exit(),
    }
}

fn print_sha256_file_usage_and_exit() -> ! {
    eprintln!("usage: cargo run -p vogon-xtask -- check-sha256-file ARTIFACT [CHECKSUM_FILE]");
    std::process::exit(2);
}

struct ArchiveContentsOptions {
    archive_directory: PathBuf,
    binary: String,
    required_files: Vec<String>,
}

fn parse_archive_contents_args(args: Vec<String>) -> ArchiveContentsOptions {
    let Some((directory, rest)) = args.split_first() else {
        print_archive_contents_usage_and_exit();
    };

    let mut binary = None;
    let mut required_files = Vec::new();
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--binary" => {
                let Some(value) = rest.get(index + 1) else {
                    print_archive_contents_usage_and_exit();
                };
                binary = Some(value.clone());
                index += 2;
            }
            "--required-file" => {
                let Some(value) = rest.get(index + 1) else {
                    print_archive_contents_usage_and_exit();
                };
                required_files.push(value.clone());
                index += 2;
            }
            _ => print_archive_contents_usage_and_exit(),
        }
    }

    let Some(binary) = binary else {
        print_archive_contents_usage_and_exit();
    };
    if required_files.is_empty() {
        required_files.extend(
            DEFAULT_ARCHIVE_REQUIRED_FILES
                .iter()
                .map(|file| (*file).to_owned()),
        );
    }

    ArchiveContentsOptions {
        archive_directory: PathBuf::from(directory),
        binary,
        required_files,
    }
}

fn print_archive_contents_usage_and_exit() -> ! {
    eprintln!(
        "usage: cargo run -p vogon-xtask -- check-archive-contents ARCHIVE_DIRECTORY --binary NAME [--required-file NAME ...]"
    );
    std::process::exit(2);
}

fn check_benchmark_output_from_stdin(expected_iterations: i64) -> Result<(), Vec<String>> {
    let mut output = String::new();
    io::stdin().read_to_string(&mut output).map_err(|error| {
        vec![format!(
            "failed to read benchmark output from stdin: {error}"
        )]
    })?;
    check_benchmark_output(&output, expected_iterations)
}

fn check_benchmark_output(output: &str, expected_iterations: i64) -> Result<(), Vec<String>> {
    let metrics = parse_benchmark_metrics(output);
    let mut errors = Vec::new();

    for metric in REQUIRED_BENCHMARK_METRICS {
        if !metrics.contains_key(*metric) {
            errors.push(format!("missing benchmark metric: {metric}"));
        }
    }

    if let Some(iterations) = parse_int_metric(&metrics, "iterations", &mut errors) {
        if iterations != expected_iterations {
            errors.push(format!(
                "benchmark iterations mismatch: expected {expected_iterations}, got {iterations}"
            ));
        }
    }

    if let Some(elapsed_ms) = parse_float_metric(&metrics, "elapsed_ms", &mut errors) {
        if elapsed_ms <= 0.0 {
            errors.push("benchmark elapsed_ms must be greater than zero".to_owned());
        }
    }

    if let Some(iterations_per_second) =
        parse_float_metric(&metrics, "iterations_per_second", &mut errors)
    {
        if iterations_per_second <= 0.0 {
            errors.push("benchmark iterations_per_second must be greater than zero".to_owned());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn parse_benchmark_metrics(output: &str) -> BTreeMap<String, String> {
    let required_metrics = REQUIRED_BENCHMARK_METRICS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut metrics = BTreeMap::new();
    for line in output.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if required_metrics.contains(name) {
            metrics.insert(name.to_owned(), value.trim().to_owned());
        }
    }
    metrics
}

fn parse_int_metric(
    metrics: &BTreeMap<String, String>,
    name: &str,
    errors: &mut Vec<String>,
) -> Option<i64> {
    let value = metrics.get(name)?;
    match value.parse::<i64>() {
        Ok(parsed) => {
            if parsed <= 0 {
                errors.push(format!("benchmark {name} must be greater than zero"));
            }
            Some(parsed)
        }
        Err(_) => {
            errors.push(format!("benchmark {name} must be an integer"));
            None
        }
    }
}

fn parse_float_metric(
    metrics: &BTreeMap<String, String>,
    name: &str,
    errors: &mut Vec<String>,
) -> Option<f64> {
    let value = metrics.get(name)?;
    match value.parse::<f64>() {
        Ok(parsed) => {
            if !parsed.is_finite() {
                errors.push(format!("benchmark {name} must be finite"));
                None
            } else {
                Some(parsed)
            }
        }
        Err(_) => {
            errors.push(format!("benchmark {name} must be a number"));
            None
        }
    }
}

fn check_cargo_metadata_json_file(
    metadata_file: &Path,
    expected_packages: &[String],
) -> Result<(), Vec<String>> {
    let output = fs::read_to_string(metadata_file)
        .map_err(|error| vec![format!("Cargo metadata JSON file cannot be read: {error}")])?;
    check_cargo_metadata_json(output.trim_start_matches('\u{feff}'), expected_packages)
}

fn check_cargo_metadata_json(
    output: &str,
    expected_packages: &[String],
) -> Result<(), Vec<String>> {
    let data = serde_json::from_str::<JsonValue>(output)
        .map_err(|error| vec![format!("Cargo metadata JSON is invalid: {error}")])?;
    let Some(data) = data.as_object() else {
        return Err(vec![
            "Cargo metadata JSON root must be an object".to_owned(),
        ]);
    };

    let mut errors = Vec::new();
    let mut package_ids = BTreeSet::new();
    let mut package_names_by_id = BTreeMap::new();

    let packages = match data.get("packages").and_then(JsonValue::as_array) {
        Some(packages) if !packages.is_empty() => packages.as_slice(),
        _ => {
            errors.push("Cargo metadata JSON packages must be a non-empty array".to_owned());
            &[]
        }
    };

    for (index, package) in packages.iter().enumerate() {
        let context = format!("Cargo metadata package {}", index + 1);
        let Some(package) = package.as_object() else {
            errors.push(format!("{context} must be an object"));
            continue;
        };
        let package_id = require_cargo_metadata_string(package, "id", &context, &mut errors);
        let package_name = require_cargo_metadata_string(package, "name", &context, &mut errors);
        require_cargo_metadata_string(package, "version", &context, &mut errors);
        require_cargo_metadata_string(package, "manifest_path", &context, &mut errors);
        if let Some(package_id) = package_id {
            package_ids.insert(package_id.to_owned());
            if let Some(package_name) = package_name {
                package_names_by_id.insert(package_id.to_owned(), package_name.to_owned());
            }
        }
    }

    let workspace_members = match data.get("workspace_members").and_then(JsonValue::as_array) {
        Some(workspace_members) if !workspace_members.is_empty() => workspace_members.as_slice(),
        _ => {
            errors
                .push("Cargo metadata JSON workspace_members must be a non-empty array".to_owned());
            &[]
        }
    };

    for (index, member_id) in workspace_members.iter().enumerate() {
        let Some(member_id) = member_id.as_str().filter(|value| !value.is_empty()) else {
            errors.push(format!(
                "Cargo metadata workspace member {} must be a non-empty string",
                index + 1
            ));
            continue;
        };
        if !package_ids.is_empty() && !package_ids.contains(member_id) {
            errors.push(format!(
                "Cargo metadata workspace member {} is missing from packages",
                index + 1
            ));
        }
    }

    let workspace_package_names = workspace_members
        .iter()
        .filter_map(JsonValue::as_str)
        .filter_map(|member_id| package_names_by_id.get(member_id))
        .cloned()
        .collect::<BTreeSet<_>>();
    for package_name in expected_packages {
        if !workspace_package_names.contains(package_name) {
            errors.push(format!(
                "Cargo metadata workspace package missing: expected {package_name}, got {}",
                json_string_array_display(workspace_package_names.iter().map(String::as_str))
            ));
        }
    }

    let nodes = match data.get("resolve").and_then(JsonValue::as_object) {
        Some(resolve) => match resolve.get("nodes").and_then(JsonValue::as_array) {
            Some(nodes) if !nodes.is_empty() => nodes.as_slice(),
            _ => {
                errors
                    .push("Cargo metadata JSON resolve.nodes must be a non-empty array".to_owned());
                &[]
            }
        },
        None => {
            errors.push("Cargo metadata JSON resolve must be an object".to_owned());
            errors.push("Cargo metadata JSON resolve.nodes must be a non-empty array".to_owned());
            &[]
        }
    };

    for (index, node) in nodes.iter().enumerate() {
        let context = format!("Cargo metadata resolve node {}", index + 1);
        let Some(node) = node.as_object() else {
            errors.push(format!("{context} must be an object"));
            continue;
        };
        let node_id = require_cargo_metadata_string(node, "id", &context, &mut errors);
        if !matches!(node.get("deps"), Some(JsonValue::Array(_))) {
            errors.push(format!("{context} deps must be an array"));
        }
        if let Some(node_id) = node_id {
            if !package_ids.is_empty() && !package_ids.contains(node_id) {
                errors.push(format!("{context} is missing from packages"));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn require_cargo_metadata_string<'a>(
    data: &'a serde_json::Map<String, JsonValue>,
    field: &str,
    context: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    let value = data.get(field).and_then(JsonValue::as_str);
    match value {
        Some(value) if !value.is_empty() => Some(value),
        _ => {
            errors.push(format!("{context} {field} must be a non-empty string"));
            None
        }
    }
}

fn json_string_array_display<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let array = values
        .map(|value| JsonValue::String(value.to_owned()))
        .collect::<Vec<_>>();
    serde_json::to_string(&array).unwrap_or_else(|_| "[]".to_owned())
}

fn check_providers_json_from_stdin() -> Result<(), Vec<String>> {
    let mut output = String::new();
    io::stdin()
        .read_to_string(&mut output)
        .map_err(|error| vec![format!("failed to read providers JSON from stdin: {error}")])?;
    check_providers_json(&output)
}

fn check_providers_json(output: &str) -> Result<(), Vec<String>> {
    let data = serde_json::from_str::<JsonValue>(output)
        .map_err(|error| vec![format!("providers JSON is invalid: {error}")])?;
    let Some(data) = data.as_object() else {
        return Err(vec!["providers JSON root must be an object".to_owned()]);
    };
    let Some(providers) = data.get("providers").and_then(JsonValue::as_array) else {
        return Err(vec!["providers must be an array".to_owned()]);
    };

    let mut errors = Vec::new();
    let mut providers_by_name = BTreeMap::new();
    for (index, provider) in providers.iter().enumerate() {
        let Some(provider) = provider.as_object() else {
            errors.push(format!("provider at index {index} must be an object"));
            continue;
        };
        let Some(name) = provider.get("name").and_then(JsonValue::as_str) else {
            errors.push(format!("provider at index {index} must have string name"));
            continue;
        };
        if providers_by_name.contains_key(name) {
            errors.push(format!("duplicate provider {name}"));
            continue;
        }
        providers_by_name.insert(name.to_owned(), provider);
    }

    let expected_names = EXPECTED_PROVIDER_JSON
        .iter()
        .map(|provider| provider.name)
        .collect::<BTreeSet<_>>();
    let actual_names = providers_by_name
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    for name in expected_names.difference(&actual_names) {
        errors.push(format!("providers must include {name}"));
    }
    for name in actual_names.difference(&expected_names) {
        errors.push(format!(
            "providers must not include unexpected provider {name}"
        ));
    }

    let mut default_count = 0;
    for expected in EXPECTED_PROVIDER_JSON {
        let Some(provider) = providers_by_name.get(expected.name) else {
            continue;
        };
        if !matches!(provider.get("enabled"), Some(JsonValue::Bool(_))) {
            errors.push(format!(
                "provider {} enabled must be boolean",
                expected.name
            ));
        }
        if provider.get("default") == Some(&JsonValue::Bool(true)) {
            default_count += 1;
        }
        validate_provider_json_bool_field(
            &mut errors,
            expected.name,
            provider,
            "default",
            expected.default_provider,
        );
        validate_provider_json_string_or_null_field(
            &mut errors,
            expected.name,
            provider,
            "credential_env",
            expected.credential_env,
        );
        validate_provider_json_credential_configured(&mut errors, expected.name, provider);
        validate_provider_json_string_or_null_field(
            &mut errors,
            expected.name,
            provider,
            "default_base_url",
            expected.default_base_url,
        );
        validate_provider_json_string_or_null_field(
            &mut errors,
            expected.name,
            provider,
            "default_model",
            expected.default_model,
        );
        validate_provider_json_string_or_null_field(
            &mut errors,
            expected.name,
            provider,
            "documentation_url",
            expected.documentation_url,
        );
        validate_provider_json_string_or_null_field(
            &mut errors,
            expected.name,
            provider,
            "usage_url",
            expected.usage_url,
        );
    }

    if default_count != 1 {
        errors.push(format!(
            "exactly one provider must be default, found {default_count}"
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_provider_json_bool_field(
    errors: &mut Vec<String>,
    name: &str,
    provider: &serde_json::Map<String, JsonValue>,
    field: &str,
    expected: bool,
) {
    let actual = provider.get(field);
    if actual != Some(&JsonValue::Bool(expected)) {
        errors.push(format!(
            "provider {name} {field} mismatch: expected {}, got {}",
            JsonValue::Bool(expected),
            json_value_display(actual)
        ));
    }
}

fn validate_provider_json_string_or_null_field(
    errors: &mut Vec<String>,
    name: &str,
    provider: &serde_json::Map<String, JsonValue>,
    field: &str,
    expected: Option<&str>,
) {
    let expected_value =
        expected.map_or(JsonValue::Null, |value| JsonValue::String(value.to_owned()));
    let actual = provider.get(field);
    if actual != Some(&expected_value) {
        errors.push(format!(
            "provider {name} {field} mismatch: expected {}, got {}",
            expected_value,
            json_value_display(actual)
        ));
    }
}

fn validate_provider_json_credential_configured(
    errors: &mut Vec<String>,
    name: &str,
    provider: &serde_json::Map<String, JsonValue>,
) {
    let actual = provider.get("credential_configured");
    let expected = EXPECTED_PROVIDER_JSON
        .iter()
        .find(|expected| expected.name == name)
        .map(|expected| expected.credential_configured);
    match expected {
        Some(ExpectedProviderJsonValue::Null) if actual != Some(&JsonValue::Null) => {
            errors.push(format!(
                "provider {name} credential_configured mismatch: expected null, got {}",
                json_value_display(actual)
            ));
        }
        Some(ExpectedProviderJsonValue::BoolOrNull)
            if !matches!(actual, Some(JsonValue::Bool(_)) | Some(JsonValue::Null)) =>
        {
            errors.push(format!(
                "provider {name} credential_configured must be boolean or null, got {}",
                json_value_display(actual)
            ));
        }
        None => {}
        _ => {}
    }
}

fn json_value_display(value: Option<&JsonValue>) -> String {
    value.cloned().unwrap_or(JsonValue::Null).to_string()
}

fn check_workflow_json_from_stdin(options: &WorkflowJsonOptions) -> Result<(), Vec<String>> {
    let mut output = String::new();
    io::stdin().read_to_string(&mut output).map_err(|error| {
        vec![format!(
            "failed to read workflow check JSON from stdin: {error}"
        )]
    })?;
    check_workflow_json(
        &output,
        options.expected_workflow_name.as_deref(),
        options.expected_step_count,
    )
}

fn check_workflow_json(
    output: &str,
    expected_workflow_name: Option<&str>,
    expected_step_count: Option<i64>,
) -> Result<(), Vec<String>> {
    let data = serde_json::from_str::<JsonValue>(output)
        .map_err(|error| vec![format!("workflow check JSON is invalid: {error}")])?;
    let Some(data) = data.as_object() else {
        return Err(vec![
            "workflow check JSON root must be an object".to_owned(),
        ]);
    };

    let mut errors = Vec::new();
    let workflow_name = data.get("workflow_name");
    match workflow_name.and_then(JsonValue::as_str) {
        Some(name) if !name.is_empty() => {
            if let Some(expected) = expected_workflow_name.filter(|expected| name != *expected) {
                errors.push(format!(
                    "workflow check JSON workflow_name mismatch: expected {}, got {}",
                    expected,
                    json_value_display(workflow_name)
                ));
            }
        }
        _ => errors.push("workflow check JSON workflow_name must be a non-empty string".to_owned()),
    }

    let step_count = data.get("step_count");
    match step_count.and_then(JsonValue::as_i64) {
        Some(count) if count > 0 => {
            if let Some(expected) = expected_step_count.filter(|expected| count != *expected) {
                errors.push(format!(
                    "workflow check JSON step_count mismatch: expected {}, got {}",
                    expected,
                    json_value_display(step_count)
                ));
            }
        }
        _ => errors.push("workflow check JSON step_count must be a positive integer".to_owned()),
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_doctor_json_from_stdin() -> Result<(), Vec<String>> {
    let mut output = String::new();
    io::stdin()
        .read_to_string(&mut output)
        .map_err(|error| vec![format!("failed to read doctor JSON from stdin: {error}")])?;
    check_doctor_json(&output)
}

fn check_doctor_json(output: &str) -> Result<(), Vec<String>> {
    let data = serde_json::from_str::<JsonValue>(output)
        .map_err(|error| vec![format!("doctor JSON is invalid: {error}")])?;
    let Some(data) = data.as_object() else {
        return Err(vec!["doctor JSON root must be an object".to_owned()]);
    };

    let mut errors = Vec::new();
    if data.get("status").and_then(JsonValue::as_str) != Some("ok") {
        errors.push("doctor status must be ok".to_owned());
    }

    match data.get("checks").and_then(JsonValue::as_array) {
        Some(checks) => {
            let has_runtime_check = checks.iter().any(|check| {
                check.as_object().is_some_and(|check| {
                    check.get("name").and_then(JsonValue::as_str) == Some("deterministic_runtime")
                        && check.get("status").and_then(JsonValue::as_str) == Some("ok")
                })
            });
            if !has_runtime_check {
                errors.push("doctor checks must include ok deterministic_runtime".to_owned());
            }
        }
        None => errors.push("doctor checks must be an array".to_owned()),
    }

    let Some(providers) = data.get("providers").and_then(JsonValue::as_array) else {
        errors.push("doctor providers must be an array".to_owned());
        return Err(errors);
    };

    let providers_by_name = providers
        .iter()
        .filter_map(|provider| {
            let provider = provider.as_object()?;
            let name = provider.get("name").and_then(JsonValue::as_str)?;
            Some((name.to_owned(), provider))
        })
        .collect::<BTreeMap<_, _>>();

    for expected in EXPECTED_PROVIDER_JSON {
        let Some(provider) = providers_by_name.get(expected.name) else {
            errors.push(format!("doctor providers must include {}", expected.name));
            continue;
        };
        match expected.usage_url {
            Some(expected_url) => {
                if provider.get("usage_url").and_then(JsonValue::as_str) != Some(expected_url) {
                    errors.push(format!(
                        "doctor provider {} usage_url mismatch: expected {expected_url}, got {}",
                        expected.name,
                        json_value_display(provider.get("usage_url"))
                    ));
                }
            }
            None => {
                if provider.get("usage_url") != Some(&JsonValue::Null) {
                    errors.push(format!(
                        "doctor provider {} usage_url must be null, got {}",
                        expected.name,
                        json_value_display(provider.get("usage_url"))
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_contributing_checklist(root: &Path) -> Result<(), Vec<String>> {
    let readme = root.join("README.md");
    let contributing = root.join("CONTRIBUTING.md");
    if !readme.is_file() {
        return Err(vec!["README.md: missing README local checks".to_owned()]);
    }
    if !contributing.is_file() {
        return Err(vec![
            "CONTRIBUTING.md: missing contributor documentation".to_owned(),
        ]);
    }

    let readme_commands = extract_shell_commands(&readme, README_LOCAL_CHECKS_MARKER)?;
    let contributing_commands =
        extract_shell_commands(&contributing, CONTRIBUTING_DEVELOPMENT_MARKER)?;
    let contributing_text = fs::read_to_string(&contributing)
        .map_err(|error| vec![format!("{}: {error}", contributing.display())])?;
    let mut errors = Vec::new();

    if readme_commands.is_empty() {
        errors.push("README.md: missing local check command block".to_owned());
    }
    if contributing_commands.is_empty() {
        errors.push("CONTRIBUTING.md: missing development command block".to_owned());
    }

    let readme_command_set = readme_commands.iter().collect::<BTreeSet<_>>();
    for command in REQUIRED_README_COMMANDS {
        if !readme_command_set.contains(&command.to_string()) {
            errors.push(format!(
                "README.md: missing required local check `{command}`"
            ));
        }
    }

    let contributing_command_set = contributing_commands.iter().collect::<BTreeSet<_>>();
    for command in readme_commands {
        if !contributing_command_set.contains(&command) {
            errors.push(format!(
                "CONTRIBUTING.md: missing README local check `{command}`"
            ));
        }
    }

    for (workflow_name, secret_name) in LIVE_WORKFLOW_GUIDANCE {
        if !contributing_text.contains(workflow_name) {
            errors.push(format!(
                "CONTRIBUTING.md: missing `{workflow_name}` guidance"
            ));
        }
        if !contributing_text.contains(secret_name) {
            errors.push(format!(
                "CONTRIBUTING.md: missing `{secret_name}` live smoke secret guidance"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_pr_template(root: &Path) -> Result<(), Vec<String>> {
    let readme = root.join("README.md");
    let pr_template = root.join(".github/pull_request_template.md");
    if !readme.is_file() {
        return Err(vec!["README.md: missing README local checks".to_owned()]);
    }
    if !pr_template.is_file() {
        return Err(vec![
            ".github/pull_request_template.md: missing pull request template".to_owned(),
        ]);
    }

    let readme_commands = extract_shell_commands(&readme, README_LOCAL_CHECKS_MARKER)?;
    let template_commands = extract_pr_template_commands(&pr_template)?;
    let mut errors = Vec::new();

    if readme_commands.is_empty() {
        errors.push("README.md: missing local check command block".to_owned());
    }
    if template_commands.is_empty() {
        errors.push(
            ".github/pull_request_template.md: missing verification command checklist".to_owned(),
        );
    }

    let template_command_set = template_commands.iter().collect::<BTreeSet<_>>();
    for command in readme_commands {
        if !template_command_set.contains(&command) {
            errors.push(format!(
                ".github/pull_request_template.md: missing README local check `{command}`"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_deployment_checklist(root: &Path) -> Result<(), Vec<String>> {
    let readme = root.join("README.md");
    let release_doc = root.join("docs").join("release.md");
    let deployment_doc = root.join("docs").join("deployment.md");
    if !readme.is_file() {
        return Err(vec!["README.md: missing README local checks".to_owned()]);
    }
    if !release_doc.is_file() {
        return Err(vec![
            "docs/release.md: missing release process documentation".to_owned(),
        ]);
    }
    if !deployment_doc.is_file() {
        return Err(vec![
            "docs/deployment.md: missing deployment documentation".to_owned(),
        ]);
    }

    let readme_commands = extract_shell_commands(&readme, README_LOCAL_CHECKS_MARKER)?;
    let release_commands = extract_shell_commands(&release_doc, RELEASE_VERIFICATION_MARKER)?;
    let deployment_commands = extract_shell_commands(&deployment_doc, DEPLOYMENT_SMOKE_MARKER)?;
    let mut errors = Vec::new();

    if readme_commands.is_empty() {
        errors.push("README.md: missing local check command block".to_owned());
    }
    if release_commands.is_empty() {
        errors.push("docs/release.md: missing release verification command block".to_owned());
    }
    if deployment_commands.is_empty() {
        errors.push("docs/deployment.md: missing deployment smoke command block".to_owned());
    }

    let readme_command_set = readme_commands.iter().collect::<BTreeSet<_>>();
    let release_command_set = release_commands.iter().collect::<BTreeSet<_>>();
    for command in deployment_commands {
        if !readme_command_set.contains(&command) {
            errors.push(format!(
                "README.md: missing deployment smoke command `{command}`"
            ));
        }
        if !release_command_set.contains(&command) {
            errors.push(format!(
                "docs/release.md: missing deployment smoke command `{command}`"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_deployment_docs(root: &Path) -> Result<(), Vec<String>> {
    let path = root.join("docs").join("deployment.md");
    if !path.exists() {
        return Err(vec![
            "docs/deployment.md: missing deployment documentation".to_owned(),
        ]);
    }

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => return Err(vec![format!("docs/deployment.md: {error}")]),
    };
    let provider_section = markdown_section(&text, PROVIDER_CREDENTIALS_MARKER);
    if provider_section.is_empty() {
        return Err(vec![
            "docs/deployment.md: missing Provider Credentials section".to_owned(),
        ]);
    }

    let mut errors = Vec::new();
    for (provider, env_var) in DEPLOYMENT_PROVIDER_EXAMPLES {
        if !provider_section.contains(&format!("-e {env_var}")) {
            errors.push(format!(
                "docs/deployment.md: missing container env example for {env_var}"
            ));
        }
        if !provider_section.contains(&format!("--provider {provider}")) {
            errors.push(format!(
                "docs/deployment.md: missing container run example for provider `{provider}`"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_release_checklist(root: &Path) -> Result<(), Vec<String>> {
    let readme = root.join("README.md");
    let release_doc = root.join("docs").join("release.md");
    if !readme.is_file() {
        return Err(vec!["README.md: missing README local checks".to_owned()]);
    }
    if !release_doc.is_file() {
        return Err(vec![
            "docs/release.md: missing release process documentation".to_owned(),
        ]);
    }

    let readme_commands = extract_shell_commands(&readme, README_LOCAL_CHECKS_MARKER)?;
    let release_commands = extract_shell_commands(&release_doc, RELEASE_VERIFICATION_MARKER)?;
    let mut errors = Vec::new();

    if readme_commands.is_empty() {
        errors.push("README.md: missing local check command block".to_owned());
    }
    if release_commands.is_empty() {
        errors.push("docs/release.md: missing release verification command block".to_owned());
    }

    let release_command_set = release_commands.iter().collect::<BTreeSet<_>>();
    for command in readme_commands {
        if !release_command_set.contains(&command) {
            errors.push(format!(
                "docs/release.md: missing README local check `{command}`"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_public_status_docs(root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for (relative_path, snippets) in REQUIRED_PUBLIC_STATUS_SNIPPETS {
        let path = root.join(relative_path);
        if !path.is_file() {
            errors.push(format!("{relative_path}: missing public status document"));
            continue;
        }

        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                errors.push(format!("{relative_path}: {error}"));
                continue;
            }
        };

        for snippet in *snippets {
            if !text.contains(snippet) {
                errors.push(format!(
                    "{relative_path}: missing \"{}\"",
                    single_line(snippet)
                ));
            }
        }
        for phrase in STALE_PUBLIC_STATUS_PHRASES {
            if text.contains(phrase) {
                errors.push(format!(
                    "{relative_path}: stale status phrase \"{}\"",
                    single_line(phrase)
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn markdown_section(text: &str, marker: &str) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let Some(start) = lines.iter().position(|line| *line == marker) else {
        return String::new();
    };

    let mut section_lines = Vec::new();
    for line in lines.iter().skip(start + 1) {
        if line.starts_with("## ") {
            break;
        }
        section_lines.push(*line);
    }
    section_lines.join("\n")
}

fn check_docs_links(root: &Path) -> Result<(), Vec<String>> {
    let root = fs::canonicalize(root).unwrap_or_else(|_| normalize_path(root));
    let mut errors = Vec::new();
    for markdown_file in markdown_files(&root) {
        let links = match extract_markdown_links(&markdown_file) {
            Ok(links) => links,
            Err(error) => {
                errors.push(format!(
                    "{}: {error}",
                    relative_path(&root, &markdown_file)
                        .unwrap_or_else(|| markdown_file.display().to_string())
                ));
                continue;
            }
        };

        for link in links {
            match resolve_repository_link(&root, &markdown_file, &link.target) {
                Ok(Some(resolved)) => {
                    if !resolved.exists() {
                        let source = relative_path(&root, &markdown_file)
                            .unwrap_or_else(|| markdown_file.display().to_string());
                        let target = relative_path(&root, &resolved)
                            .unwrap_or_else(|| resolved.display().to_string());
                        errors.push(format!(
                            "{source}:{}: missing link target `{}` -> `{target}`",
                            link.line, link.target
                        ));
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    let source = relative_path(&root, &markdown_file)
                        .unwrap_or_else(|| markdown_file.display().to_string());
                    errors.push(format!("{source}:{}: {error}", link.line));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MarkdownLink {
    line: usize,
    target: String,
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_markdown_files(root, root, &mut paths);
    paths.sort();
    paths
}

fn collect_markdown_files(root: &Path, directory: &Path, paths: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        if relative
            .components()
            .any(|component| matches!(component, Component::Normal(name) if name == ".git" || name == "target"))
        {
            continue;
        }
        if path.is_dir() {
            collect_markdown_files(root, &path, paths);
        } else if path.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| {
                    MARKDOWN_SUFFIXES.contains(&extension.to_ascii_lowercase().as_str())
                })
                .unwrap_or(false)
        {
            paths.push(path);
        }
    }
}

fn extract_markdown_links(path: &Path) -> Result<Vec<MarkdownLink>, std::io::Error> {
    let text = fs::read_to_string(path)?;
    let mut links = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for target in markdown_link_targets(line) {
            links.push(MarkdownLink {
                line: index + 1,
                target,
            });
        }
    }
    Ok(links)
}

fn markdown_link_targets(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut targets = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'[' || (index > 0 && bytes[index - 1] == b'!') {
            index += 1;
            continue;
        }

        let Some(label_end) = find_matching_bracket(bytes, index) else {
            index += 1;
            continue;
        };
        if label_end + 1 >= bytes.len() || bytes[label_end + 1] != b'(' {
            index += 1;
            continue;
        }

        let target_start = label_end + 2;
        let Some(relative_target_end) = line[target_start..].find(')') else {
            index += 1;
            continue;
        };
        let target_end = target_start + relative_target_end;
        let target = normalize_markdown_target(&line[target_start..target_end]);
        if !target.is_empty() {
            targets.push(target);
        }
        index = target_end + 1;
    }

    targets
}

fn find_matching_bracket(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0;
    for (index, byte) in bytes.iter().enumerate().skip(start) {
        match byte {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn normalize_markdown_target(raw_target: &str) -> String {
    let mut target = raw_target.trim();
    if target.is_empty() {
        return String::new();
    }
    if target.starts_with('<') && target.ends_with('>') {
        target = target[1..target.len() - 1].trim();
    }
    target.split_whitespace().next().unwrap_or("").to_owned()
}

fn resolve_repository_link(
    root: &Path,
    source: &Path,
    target: &str,
) -> Result<Option<PathBuf>, String> {
    let target_without_anchor = target.split('#').next().unwrap_or("");
    if target_without_anchor.is_empty() {
        return Ok(None);
    }

    if target_without_anchor.starts_with("http://") || target_without_anchor.starts_with("https://")
    {
        return resolve_github_repository_link(root, target_without_anchor);
    }
    if target_without_anchor.contains("://") {
        return Ok(None);
    }

    if target_without_anchor.starts_with('/') {
        return safe_join(root, root, target_without_anchor.trim_start_matches('/'));
    }

    safe_join(root, source.parent().unwrap_or(root), target_without_anchor)
}

fn resolve_github_repository_link(root: &Path, target: &str) -> Result<Option<PathBuf>, String> {
    let Some(path) = target
        .strip_prefix("https://github.com/")
        .or_else(|| target.strip_prefix("http://github.com/"))
    else {
        return Ok(None);
    };
    let path = path.split('?').next().unwrap_or(path);
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .map(percent_decode)
        .collect::<Vec<_>>();
    if parts.len() < 5 {
        return Ok(None);
    }
    if parts[0] != REPO_OWNER || parts[1] != REPO_NAME {
        return Ok(None);
    }
    if !matches!(parts[2].as_str(), "blob" | "tree") || parts[3] != "main" {
        return Ok(None);
    }

    let relative = parts[4..].join("/");
    safe_join(root, root, &relative)
}

fn safe_join(root: &Path, base: &Path, target: &str) -> Result<Option<PathBuf>, String> {
    let root = normalize_path(root);
    let base = normalize_path(base);
    let resolved = normalize_path(&base.join(target));
    if resolved == root || resolved.starts_with(&root) {
        Ok(Some(resolved))
    } else {
        Err(format!(
            "link target escapes repository root: {}",
            resolved.display()
        ))
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[index + 1..index + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    decoded.push(byte);
                    index += 3;
                    continue;
                }
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn check_issue_templates(root: &Path) -> Result<(), Vec<String>> {
    let template_dir = root.join(".github").join("ISSUE_TEMPLATE");
    let config_path = template_dir.join("config.yml");
    let bug_path = template_dir.join("bug_report.yml");
    let feature_path = template_dir.join("feature_request.yml");
    let mut errors = Vec::new();

    errors.extend(check_issue_template_config(root, &config_path));
    errors.extend(check_issue_form(
        root,
        &bug_path,
        "Bug report",
        "title: \"Bug: \"",
        "- bug",
        BUG_ISSUE_REQUIRED_FIELDS,
    ));
    errors.extend(check_bug_version_placeholder(root, &bug_path));
    errors.extend(check_issue_form(
        root,
        &feature_path,
        "Feature request",
        "title: \"Feature: \"",
        "- enhancement",
        FEATURE_ISSUE_REQUIRED_FIELDS,
    ));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_issue_template_config(root: &Path, path: &Path) -> Vec<String> {
    let relative = issue_relative_path(root, path);
    if !path.exists() {
        return vec![format!("{relative}: missing issue template config")];
    }

    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => return vec![format!("{relative}: {error}")],
    };
    let mut errors = Vec::new();
    if !text.contains("blank_issues_enabled: false") {
        errors.push(format!("{relative}: blank issues must stay disabled"));
    }
    if !text.contains("https://github.com/kaleab-kali/vogon-runtime/security/advisories/new") {
        errors.push(format!(
            "{relative}: missing private vulnerability reporting link"
        ));
    }
    errors
}

fn check_bug_version_placeholder(root: &Path, path: &Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }
    let relative = issue_relative_path(root, path);
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => return vec![format!("{relative}: {error}")],
    };
    if text.contains(REQUIRED_BUG_VERSION_PLACEHOLDER) {
        Vec::new()
    } else {
        vec![format!(
            "{relative}: version placeholder must match the latest public release"
        )]
    }
}

fn check_issue_form(
    root: &Path,
    path: &Path,
    expected_name: &str,
    expected_title: &str,
    expected_label: &str,
    required_fields: &[&str],
) -> Vec<String> {
    let relative = issue_relative_path(root, path);
    if !path.exists() {
        return vec![format!("{relative}: missing issue form")];
    }

    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => return vec![format!("{relative}: {error}")],
    };
    let lines = text.lines().collect::<Vec<_>>();
    let mut errors = Vec::new();

    if !text.contains(&format!("name: {expected_name}")) {
        errors.push(format!(
            "{relative}: missing expected name `{expected_name}`"
        ));
    }
    if !text.contains(expected_title) {
        errors.push(format!("{relative}: missing expected title prefix"));
    }
    if !text.contains(expected_label) {
        errors.push(format!(
            "{relative}: missing expected label `{expected_label}`"
        ));
    }

    let field_ids = issue_field_ids(&lines);
    for field_id in required_fields {
        if !field_ids.contains(*field_id) {
            errors.push(format!("{relative}: missing required field `{field_id}`"));
        }
    }

    errors.extend(check_issue_dropdown_options(&relative, &lines));
    errors.extend(check_issue_before_submit(&relative, &lines));
    errors
}

fn issue_field_ids(lines: &[&str]) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for line in lines {
        let stripped = line.trim();
        if let Some(id) = stripped.strip_prefix("id: ") {
            ids.insert(id.trim().to_owned());
        }
    }
    ids
}

fn check_issue_dropdown_options(relative: &str, lines: &[&str]) -> Vec<String> {
    let options = issue_dropdown_options(lines, &["component", "area"]);
    if options.is_empty() {
        return vec![format!(
            "{relative}: missing component or area dropdown options"
        )];
    }

    let missing = REQUIRED_ISSUE_AREAS
        .iter()
        .filter(|area| !options.contains(**area))
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "{relative}: dropdown options missing {}",
            missing.join(", ")
        )]
    }
}

fn issue_dropdown_options(lines: &[&str], field_ids: &[&str]) -> BTreeSet<String> {
    let mut options = BTreeSet::new();
    let mut in_target_dropdown = false;
    let mut in_options = false;

    for line in lines {
        let stripped = line.trim();
        if let Some(id) = stripped.strip_prefix("id: ") {
            in_target_dropdown = field_ids.contains(&id.trim());
            in_options = false;
            continue;
        }
        if in_target_dropdown && stripped == "options:" {
            in_options = true;
            continue;
        }
        if in_options && stripped.starts_with("- ") {
            options.insert(stripped.trim_start_matches("- ").trim().to_owned());
            continue;
        }
        if in_options && !stripped.is_empty() && !line.starts_with("        ") {
            in_options = false;
        }
    }

    options
}

fn check_issue_before_submit(relative: &str, lines: &[&str]) -> Vec<String> {
    let labels = lines
        .iter()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("- label: ")
                .map(|label| label.to_ascii_lowercase())
        })
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    for required_label in REQUIRED_ISSUE_CHECK_LABELS {
        if !labels.iter().any(|label| label.contains(required_label)) {
            errors.push(format!(
                "{relative}: missing required before-submit check `{required_label}`"
            ));
        }
    }
    errors
}

fn issue_relative_path(root: &Path, path: &Path) -> String {
    relative_path(root, path).unwrap_or_else(|| slash_path(path))
}

fn check_secrets(root: &Path) -> Result<(), Vec<String>> {
    let tracked_files = match secret_scan_files(root) {
        Ok(files) => files,
        Err(error) => return Err(vec![format!("git ls-files: {error}")]),
    };
    let mut findings = Vec::new();

    for path in tracked_files {
        if is_sensitive_artifact(&path) {
            findings.push(format_file_finding(
                root,
                &path,
                "committed sensitive cache artifact",
            ));
        }

        let Some(text) = read_secret_scan_text(&path) else {
            continue;
        };

        for (index, line) in text.lines().enumerate() {
            let line_number = index + 1;
            for pattern_name in secret_pattern_findings(line) {
                findings.push(format_secret_finding(
                    root,
                    &path,
                    line_number,
                    pattern_name,
                ));
            }
            if let Some(provider_assignment) = find_provider_secret_assignment(line) {
                findings.push(format_secret_finding(
                    root,
                    &path,
                    line_number,
                    &provider_assignment,
                ));
            }
        }
    }

    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

fn secret_scan_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    if root.join(".git").exists() {
        let output = Command::new("git")
            .arg("ls-files")
            .current_dir(root)
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Ok(stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| root.join(line))
            .collect());
    }

    let mut files = Vec::new();
    collect_secret_scan_files(root, root, &mut files);
    files.sort();
    Ok(files)
}

fn collect_secret_scan_files(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        if relative
            .components()
            .any(|component| matches!(component, Component::Normal(name) if name == ".git" || name == "target"))
        {
            continue;
        }
        if path.is_dir() {
            collect_secret_scan_files(root, &path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

fn is_sensitive_artifact(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    SENSITIVE_ARTIFACT_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn read_secret_scan_text(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > MAX_SECRET_SCAN_TEXT_BYTES {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    if bytes.contains(&0) {
        return None;
    }
    String::from_utf8(bytes).ok()
}

fn secret_pattern_findings(line: &str) -> Vec<&'static str> {
    let mut findings = Vec::new();
    if contains_prefixed_token(line, &["A3T", "AKIA", "ASIA"], 16, 16, is_upper_alnum) {
        findings.push("AWS access key id");
    }
    if contains_prefixed_token(
        line,
        &["ghp_", "gho_", "ghu_", "ghs_", "ghr_"],
        36,
        usize::MAX,
        |character| character.is_ascii_alphanumeric() || character == '_',
    ) {
        findings.push("GitHub token");
    }
    if contains_prefixed_token(line, &["AIza"], 35, 35, |character| {
        character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
    }) {
        findings.push("Google API key");
    }
    if contains_prefixed_token(line, &["gsk_"], 30, usize::MAX, |character| {
        character.is_ascii_alphanumeric()
    }) {
        findings.push("Groq API key");
    }
    if contains_prefixed_token(line, &["hf_"], 30, usize::MAX, |character| {
        character.is_ascii_alphanumeric()
    }) {
        findings.push("Hugging Face token");
    }
    if contains_prefixed_token(line, &["sk-"], 20, usize::MAX, |character| {
        character.is_ascii_alphanumeric()
    }) {
        findings.push("OpenAI API key");
    }
    if contains_prefixed_token(line, &["sk-or-v1-"], 32, usize::MAX, |character| {
        character.is_ascii_alphanumeric()
    }) {
        findings.push("OpenRouter API key");
    }
    if contains_prefixed_token(
        line,
        &["xoxb-", "xoxa-", "xoxp-", "xoxr-", "xoxs-"],
        20,
        usize::MAX,
        |character| character.is_ascii_alphanumeric() || character == '-',
    ) {
        findings.push("Slack token");
    }
    findings
}

fn contains_prefixed_token(
    line: &str,
    prefixes: &[&str],
    min_tail: usize,
    max_tail: usize,
    is_tail_character: fn(char) -> bool,
) -> bool {
    for prefix in prefixes {
        let mut search_start = 0;
        while let Some(relative_index) = line[search_start..].find(prefix) {
            let start = search_start + relative_index;
            let end_of_prefix = start + prefix.len();
            if !has_token_boundary_before(line, start) {
                search_start = end_of_prefix;
                continue;
            }

            let mut tail_len = 0;
            let mut token_end = end_of_prefix;
            for (offset, character) in line[end_of_prefix..].char_indices() {
                if !is_tail_character(character) {
                    break;
                }
                tail_len += 1;
                token_end = end_of_prefix + offset + character.len_utf8();
            }
            if tail_len >= min_tail
                && tail_len <= max_tail
                && has_token_boundary_after(line, token_end)
            {
                return true;
            }
            search_start = end_of_prefix;
        }
    }
    false
}

fn is_upper_alnum(character: char) -> bool {
    character.is_ascii_uppercase() || character.is_ascii_digit()
}

fn has_token_boundary_before(line: &str, start: usize) -> bool {
    line[..start]
        .chars()
        .next_back()
        .map(|character| !is_word_character(character))
        .unwrap_or(true)
}

fn has_token_boundary_after(line: &str, end: usize) -> bool {
    line[end..]
        .chars()
        .next()
        .map(|character| !is_word_character(character))
        .unwrap_or(true)
}

fn is_word_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn find_provider_secret_assignment(line: &str) -> Option<String> {
    for name in PROVIDER_CREDENTIAL_VARS {
        let mut search_start = 0;
        while let Some(relative_index) = line[search_start..].find(name) {
            let start = search_start + relative_index;
            let end = start + name.len();
            if line[..start]
                .chars()
                .next_back()
                .map(|character| {
                    character.is_ascii_uppercase()
                        || character.is_ascii_digit()
                        || matches!(character, '_' | '{')
                })
                .unwrap_or(false)
            {
                search_start = end;
                continue;
            }

            let after_name = &line[end..];
            let trimmed = after_name.trim_start();
            let separator = trimmed.chars().next()?;
            if !matches!(separator, ':' | '=') {
                search_start = end;
                continue;
            }
            let value = trimmed[separator.len_utf8()..]
                .trim_start()
                .split(|character: char| character.is_whitespace() || character == '#')
                .next()
                .unwrap_or("");
            let normalized = normalize_assignment_value(value);
            if is_allowed_placeholder_value(&normalized) {
                return None;
            }
            return Some(format!("committed {name} value"));
        }
    }
    None
}

fn normalize_assignment_value(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(',')
        .trim_matches('"')
        .trim_matches('\'')
        .to_owned()
}

fn is_allowed_placeholder_value(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    PLACEHOLDER_VALUES.contains(&lowered.as_str())
        || value.starts_with("${{")
        || value.starts_with('$')
        || value.contains("...")
        || lowered.starts_with("your-")
        || lowered.starts_with('<')
}

fn format_secret_finding(
    root: &Path,
    path: &Path,
    line_number: usize,
    pattern_name: &str,
) -> String {
    let relative = relative_path(root, path).unwrap_or_else(|| slash_path(path));
    format!("{relative}:{line_number}: possible {pattern_name}")
}

fn format_file_finding(root: &Path, path: &Path, pattern_name: &str) -> String {
    let relative = relative_path(root, path).unwrap_or_else(|| slash_path(path));
    format!("{relative}: possible {pattern_name}")
}

fn check_package_verification_docs(root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for relative_path in PACKAGE_VERIFICATION_DOCS {
        let path = root.join(relative_path);
        if !path.is_file() {
            errors.push(format!(
                "{relative_path}: missing package verification documentation"
            ));
            continue;
        }

        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                errors.push(format!("{relative_path}: {error}"));
                continue;
            }
        };
        let normalized_text = single_line(&text);
        if !text.contains(PACKAGE_VERIFICATION_COMMAND) {
            errors.push(format!("{relative_path}: missing offline package command"));
        }
        if !PACKAGE_VERIFICATION_RATIONALE_SNIPPETS
            .iter()
            .all(|snippet| normalized_text.contains(snippet))
        {
            errors.push(format!(
                "{relative_path}: missing package verification rationale"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_container_policy(root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    errors.extend(check_dockerfile(root));
    errors.extend(check_dockerignore(root));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_dependabot_config(root: &Path) -> Result<(), Vec<String>> {
    let path = root.join(".github").join("dependabot.yml");
    if !path.is_file() {
        return Err(vec![
            ".github/dependabot.yml: missing Dependabot configuration".to_owned(),
        ]);
    }

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => return Err(vec![format!(".github/dependabot.yml: {error}")]),
    };
    let updates = parse_dependabot_update_blocks(&text);
    let mut errors = Vec::new();

    if !text.starts_with("version: 2\n") {
        errors.push(".github/dependabot.yml: missing version 2 declaration".to_owned());
    }

    for (ecosystem, expected_config) in EXPECTED_DEPENDABOT_UPDATES {
        let Some(config) = updates.get(*ecosystem) else {
            errors.push(format!(
                ".github/dependabot.yml: missing {ecosystem} updates"
            ));
            continue;
        };

        for (key, expected_value) in *expected_config {
            if config.get(*key).map(String::as_str) != Some(*expected_value) {
                errors.push(format!(
                    ".github/dependabot.yml: {ecosystem} `{key}` must be '{}'",
                    expected_value
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn parse_dependabot_update_blocks(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut updates = BTreeMap::<String, BTreeMap<String, String>>::new();
    let mut current_ecosystem: Option<String> = None;
    let mut in_schedule = false;
    let mut in_commit_message = false;
    let mut in_groups = false;
    let mut current_group: Option<String> = None;
    let mut in_group_patterns = false;
    let mut in_group_update_types = false;

    for raw_line in text.lines() {
        let line = raw_line.trim_end();
        let stripped = line.trim();
        if stripped.is_empty() {
            continue;
        }

        if let Some(ecosystem) = stripped.strip_prefix("- package-ecosystem:") {
            let ecosystem = ecosystem.trim().to_owned();
            updates.entry(ecosystem.clone()).or_default();
            current_ecosystem = Some(ecosystem);
            in_schedule = false;
            in_commit_message = false;
            in_groups = false;
            current_group = None;
            in_group_patterns = false;
            in_group_update_types = false;
            continue;
        }

        let Some(ecosystem) = current_ecosystem.as_ref() else {
            continue;
        };

        match stripped {
            "schedule:" => {
                in_schedule = true;
                in_commit_message = false;
                in_groups = false;
                continue;
            }
            "commit-message:" => {
                in_commit_message = true;
                in_schedule = false;
                in_groups = false;
                current_group = None;
                continue;
            }
            "groups:" => {
                in_groups = true;
                in_schedule = false;
                in_commit_message = false;
                current_group = None;
                continue;
            }
            _ => {}
        }

        if in_groups
            && stripped.ends_with(':')
            && !matches!(stripped, "patterns:" | "update-types:")
        {
            current_group = Some(stripped.trim_end_matches(':').to_owned());
            in_group_patterns = false;
            in_group_update_types = false;
            continue;
        }
        if in_groups && stripped == "patterns:" {
            let Some(group) = current_group.as_ref() else {
                continue;
            };
            in_group_patterns = true;
            in_group_update_types = false;
            updates
                .entry(ecosystem.clone())
                .or_default()
                .insert(format!("groups.{group}.patterns"), String::new());
            continue;
        }
        if in_groups && stripped == "update-types:" {
            let Some(group) = current_group.as_ref() else {
                continue;
            };
            in_group_patterns = false;
            in_group_update_types = true;
            updates
                .entry(ecosystem.clone())
                .or_default()
                .insert(format!("groups.{group}.update-types"), String::new());
            continue;
        }
        if in_groups && stripped.starts_with("- ") {
            let Some(group) = current_group.as_ref() else {
                continue;
            };
            let value = stripped
                .trim_start_matches("- ")
                .trim()
                .trim_matches('"')
                .to_owned();
            let suffix = if in_group_patterns {
                "patterns"
            } else if in_group_update_types {
                "update-types"
            } else {
                continue;
            };
            let key = format!("groups.{group}.{suffix}");
            let config = updates.entry(ecosystem.clone()).or_default();
            let existing = config.get(&key).cloned().unwrap_or_default();
            config.insert(
                key,
                if existing.is_empty() {
                    value
                } else {
                    format!("{existing},{value}")
                },
            );
            continue;
        }
        if stripped.ends_with(':') && !matches!(stripped, "schedule:" | "commit-message:") {
            in_schedule = false;
            in_commit_message = false;
        }

        let Some((key, value)) = stripped.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        let config = updates.entry(ecosystem.clone()).or_default();
        if key == "directory" {
            config.insert("directory".to_owned(), value.to_owned());
        } else if key == "open-pull-requests-limit" {
            config.insert("open-pull-requests-limit".to_owned(), value.to_owned());
        } else if in_schedule && key == "interval" {
            config.insert("interval".to_owned(), value.to_owned());
        } else if in_commit_message && key == "prefix" {
            config.insert("commit-message.prefix".to_owned(), value.to_owned());
        }
    }

    updates
}

fn check_dockerfile(root: &Path) -> Vec<String> {
    let path = root.join("Dockerfile");
    if !path.is_file() {
        return vec!["Dockerfile: missing container build file".to_owned()];
    }

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => return vec![format!("Dockerfile: {error}")],
    };
    let lines = text.lines().collect::<Vec<_>>();
    let mut errors = Vec::new();

    for (description, snippet) in REQUIRED_DOCKERFILE_SNIPPETS {
        if !text.contains(snippet) {
            errors.push(format!("Dockerfile: missing {description}"));
        }
    }

    for (index, line) in lines.iter().enumerate() {
        let stripped = line.trim();
        let Some(image) = dockerfile_from_image(stripped) else {
            continue;
        };
        if image_reference_uses_latest(image) {
            errors.push(format!(
                "Dockerfile:{}: base image `{image}` must not use latest",
                index + 1
            ));
        }
        if !image_reference_has_tag_or_digest(image) {
            errors.push(format!(
                "Dockerfile:{}: base image `{image}` must include a tag or digest",
                index + 1
            ));
        }
    }

    errors
}

fn dockerfile_from_image(line: &str) -> Option<&str> {
    let mut parts = line.split_whitespace();
    let directive = parts.next()?;
    if !directive.eq_ignore_ascii_case("FROM") {
        return None;
    }
    parts.next()
}

fn image_reference_uses_latest(image: &str) -> bool {
    let image = image.split_once('@').map_or(image, |(name, _)| name);
    image
        .rsplit_once('/')
        .map_or(image, |(_, last_segment)| last_segment)
        .rsplit_once(':')
        .is_some_and(|(_, tag)| tag == "latest")
}

fn image_reference_has_tag_or_digest(image: &str) -> bool {
    if image.contains('@') {
        return true;
    }
    image
        .rsplit_once('/')
        .map_or(image, |(_, last_segment)| last_segment)
        .contains(':')
}

fn check_dockerignore(root: &Path) -> Vec<String> {
    let path = root.join(".dockerignore");
    if !path.is_file() {
        return vec![".dockerignore: missing container build context ignore file".to_owned()];
    }

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => return vec![format!(".dockerignore: {error}")],
    };
    let entries = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.trim_start().starts_with('#'))
        .collect::<BTreeSet<_>>();
    REQUIRED_DOCKERIGNORE_ENTRIES
        .iter()
        .filter(|entry| !entries.contains(**entry))
        .map(|entry| format!(".dockerignore: missing {entry}"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn single_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_shell_commands(path: &Path, marker: &str) -> Result<Vec<String>, Vec<String>> {
    let text =
        fs::read_to_string(path).map_err(|error| vec![format!("{}: {error}", path.display())])?;
    let lines = text.lines().collect::<Vec<_>>();
    let Some(marker_index) = lines.iter().position(|line| *line == marker) else {
        return Ok(Vec::new());
    };

    let mut in_block = false;
    let mut commands = Vec::new();
    for line in lines.iter().skip(marker_index + 1) {
        let stripped = line.trim();
        if stripped.starts_with("```") {
            if in_block {
                return Ok(commands);
            }
            in_block = matches!(stripped, "```sh" | "```shell" | "```bash");
            continue;
        }
        if in_block && !stripped.is_empty() {
            commands.push(stripped.to_owned());
        }
    }

    Ok(commands)
}

fn extract_pr_template_commands(path: &Path) -> Result<Vec<String>, Vec<String>> {
    let text =
        fs::read_to_string(path).map_err(|error| vec![format!("{}: {error}", path.display())])?;
    let mut commands = Vec::new();
    for line in text.lines() {
        let stripped = line.trim();
        if let Some(command) = stripped
            .strip_prefix("- [ ] `")
            .and_then(|line| line.strip_suffix('`'))
        {
            commands.push(command.to_owned());
        }
    }
    Ok(commands)
}

fn check_changelog(root: &Path) -> Result<(), Vec<String>> {
    let path = root.join("CHANGELOG.md");
    if !path.is_file() {
        return Err(vec!["CHANGELOG.md: missing changelog".to_owned()]);
    }

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => return Err(vec![format!("{}: {error}", path.display())]),
    };
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let mut errors = Vec::new();

    if lines.first().map(String::as_str) != Some("# Changelog") {
        errors.push("CHANGELOG.md: first line must be `# Changelog`".to_owned());
    }
    if !text.contains("https://keepachangelog.com/en/1.1.0/") {
        errors.push("CHANGELOG.md: missing Keep a Changelog 1.1.0 reference".to_owned());
    }
    if !text.to_lowercase().contains("semantic versioning") {
        errors.push("CHANGELOG.md: missing semantic versioning note".to_owned());
    }

    let Some(unreleased_start) = lines.iter().position(|line| line == "## [Unreleased]") else {
        errors.push("CHANGELOG.md: missing `## [Unreleased]` section".to_owned());
        return Err(errors);
    };

    let next_heading = next_release_heading(&lines, unreleased_start + 1);
    let unreleased_lines = &lines[unreleased_start + 1..next_heading];
    errors.extend(check_unreleased_changelog_section(
        unreleased_lines,
        next_heading < lines.len(),
    ));
    errors.extend(check_changelog_release_headings(&lines[next_heading..]));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn next_release_heading(lines: &[String], start: usize) -> usize {
    for (index, line) in lines.iter().enumerate().skip(start) {
        if line.starts_with("## ") && line != "## [Unreleased]" {
            return index;
        }
    }
    lines.len()
}

fn check_unreleased_changelog_section(lines: &[String], has_release: bool) -> Vec<String> {
    let section_names = lines
        .iter()
        .filter_map(|line| line.strip_prefix("### "))
        .collect::<Vec<_>>();

    if section_names.is_empty() {
        if has_release && !lines.iter().any(|line| !line.trim().is_empty()) {
            return Vec::new();
        }
        return vec![
            "CHANGELOG.md: `## [Unreleased]` must contain at least one subsection".to_owned(),
        ];
    }

    let mut errors = Vec::new();
    for section_name in &section_names {
        if !ALLOWED_UNRELEASED_CHANGELOG_SECTIONS.contains(section_name) {
            errors.push(format!(
                "CHANGELOG.md: unsupported Unreleased subsection `{section_name}`"
            ));
        }
    }
    for section_name in section_names {
        if !changelog_section_has_entry(lines, section_name) {
            errors.push(format!(
                "CHANGELOG.md: Unreleased `{section_name}` subsection has no entries"
            ));
        }
    }

    errors
}

fn check_changelog_release_headings(lines: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    for line in lines {
        if line.starts_with("## ") && (!line.starts_with("## [") || !line.contains(" - ")) {
            errors.push(format!(
                "CHANGELOG.md: release heading `{line}` must include a version and date"
            ));
        }
    }
    errors
}

fn changelog_section_has_entry(lines: &[String], section_name: &str) -> bool {
    let heading = format!("### {section_name}");
    let mut in_section = false;
    for line in lines {
        if line == &heading {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("### ") {
            return false;
        }
        if in_section && line.starts_with("- ") {
            return true;
        }
    }
    false
}

fn check_cargo_manifests(root: &Path) -> Result<(), Vec<String>> {
    let workspace_path = root.join("Cargo.toml");
    if !workspace_path.is_file() {
        return Err(vec!["Cargo.toml: missing workspace manifest".to_owned()]);
    }

    let mut errors = Vec::new();
    let workspace = match read_toml_manifest(&workspace_path) {
        Ok(workspace) => workspace,
        Err(error) => return Err(vec![error]),
    };

    let workspace_package = nested_table(&workspace, &["workspace", "package"]);
    if workspace_package.is_none() {
        errors.push("Cargo.toml: missing [workspace.package]".to_owned());
    }
    errors.extend(check_workspace_package(workspace_package));

    let members = nested_value(&workspace, &["workspace", "members"]);
    if !matches_string_list(members, expected_crate_dirs().as_slice()) {
        errors.push(format!(
            "Cargo.toml: workspace members must be {}",
            expected_crate_dirs().join(", ")
        ));
    }

    let release_profile = nested_table(&workspace, &["profile", "release"]);
    if release_profile.is_none() {
        errors.push("Cargo.toml: missing [profile.release]".to_owned());
    }
    errors.extend(check_expected_table(
        "Cargo.toml: release profile",
        release_profile,
        EXPECTED_RELEASE_PROFILE,
    ));

    let workspace_rust_lints = nested_table(&workspace, &["workspace", "lints", "rust"]);
    if workspace_rust_lints.is_none() {
        errors.push("Cargo.toml: missing [workspace.lints.rust]".to_owned());
    }
    errors.extend(check_expected_table(
        "Cargo.toml: workspace rust lint",
        workspace_rust_lints,
        EXPECTED_WORKSPACE_RUST_LINTS,
    ));

    let workspace_deps = nested_table(&workspace, &["workspace", "dependencies"]);
    if workspace_deps.is_none() {
        errors.push("Cargo.toml: missing [workspace.dependencies]".to_owned());
    }

    let mut crate_versions = BTreeMap::new();
    for (crate_name, crate_dir) in EXPECTED_CRATES {
        let manifest_path = root.join(crate_dir).join("Cargo.toml");
        if !manifest_path.is_file() {
            errors.push(format!("{crate_dir}/Cargo.toml: missing crate manifest"));
            continue;
        }

        let manifest = match read_toml_manifest(&manifest_path) {
            Ok(manifest) => manifest,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };

        let package = nested_table(&manifest, &["package"]);
        let relative_path = format!("{crate_dir}/Cargo.toml");
        let Some(package) = package else {
            errors.push(format!("{relative_path}: missing [package]"));
            continue;
        };

        errors.extend(check_crate_package(
            root,
            &manifest_path,
            crate_name,
            package,
        ));
        errors.extend(check_crate_lints(&relative_path, &manifest));
        if let Some(version) = package.get("version").and_then(Value::as_str) {
            crate_versions.insert(*crate_name, version.to_owned());
        }
    }

    if crate_versions.values().collect::<BTreeSet<_>>().len() > 1 {
        errors.push("Cargo.toml: workspace crate versions must match".to_owned());
    }

    for crate_name in ["vogon-adapters", "vogon-core"] {
        let dependency_version = workspace_deps
            .and_then(|deps| deps.get(crate_name))
            .and_then(Value::as_table)
            .and_then(|dependency| dependency.get("version"))
            .and_then(Value::as_str);
        if let Some(crate_version) = crate_versions.get(crate_name) {
            if dependency_version != Some(crate_version.as_str()) {
                errors.push(format!(
                    "Cargo.toml: workspace dependency `{crate_name}` version must match crate version {crate_version}"
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn read_toml_manifest(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

fn check_workspace_package(package: Option<&TomlTable>) -> Vec<String> {
    let Some(package) = package else {
        return EXPECTED_WORKSPACE_PACKAGE
            .iter()
            .map(|(key, expected)| {
                format!(
                    "Cargo.toml: workspace package `{key}` must be {}",
                    expected.python_repr()
                )
            })
            .collect();
    };

    check_expected_table(
        "Cargo.toml: workspace package",
        Some(package),
        EXPECTED_WORKSPACE_PACKAGE,
    )
}

fn check_expected_table(
    prefix: &str,
    table: Option<&TomlTable>,
    expected_values: &[(&str, ExpectedValue)],
) -> Vec<String> {
    let mut errors = Vec::new();
    for (key, expected) in expected_values {
        let actual = table.and_then(|table| table.get(*key));
        if !expected.matches(actual) {
            errors.push(format!(
                "{prefix} `{key}` must be {}",
                expected.python_repr()
            ));
        }
    }
    errors
}

fn check_crate_package(
    root: &Path,
    manifest_path: &Path,
    expected_name: &str,
    package: &TomlTable,
) -> Vec<String> {
    let relative_path = slash_path(manifest_path.strip_prefix(root).unwrap_or(manifest_path));
    let mut errors = Vec::new();

    for field in REQUIRED_PACKAGE_FIELDS {
        if !package.contains_key(*field) {
            errors.push(format!("{relative_path}: package missing `{field}`"));
        }
    }

    if package.get("name").and_then(Value::as_str) != Some(expected_name) {
        errors.push(format!(
            "{relative_path}: package name must be `{expected_name}`"
        ));
    }

    for (workspace_field, _) in EXPECTED_WORKSPACE_PACKAGE {
        let uses_workspace = package
            .get(*workspace_field)
            .and_then(Value::as_table)
            .and_then(|metadata| metadata.get("workspace"))
            .and_then(Value::as_bool)
            == Some(true);
        if !uses_workspace {
            errors.push(format!(
                "{relative_path}: package `{workspace_field}` must use workspace metadata"
            ));
        }
    }

    if let Some(readme) = package.get("readme").and_then(Value::as_str) {
        let readme_path = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(readme);
        if !readme_path.is_file() {
            errors.push(format!(
                "{relative_path}: readme path `{readme}` does not exist"
            ));
        }
    }

    for list_field in ["keywords", "categories"] {
        let value = package.get(list_field);
        if !is_string_list(value) {
            errors.push(format!(
                "{relative_path}: package `{list_field}` must be a string list"
            ));
        } else if matches!(value.and_then(Value::as_array), Some(items) if items.is_empty()) {
            errors.push(format!(
                "{relative_path}: package `{list_field}` must not be empty"
            ));
        }
    }

    let description = package.get("description").and_then(Value::as_str);
    if !matches!(description, Some(description) if !description.trim().is_empty()) {
        errors.push(format!(
            "{relative_path}: package `description` must not be empty"
        ));
    }

    errors
}

fn check_crate_lints(relative_path: &str, manifest: &Value) -> Vec<String> {
    let uses_workspace = nested_table(manifest, &["lints"])
        .and_then(|lints| lints.get("workspace"))
        .and_then(Value::as_bool)
        == Some(true);
    if uses_workspace {
        Vec::new()
    } else {
        vec![format!(
            "{relative_path}: crate lints must use workspace policy"
        )]
    }
}

fn nested_table<'a>(value: &'a Value, path: &[&str]) -> Option<&'a TomlTable> {
    nested_value(value, path).and_then(Value::as_table)
}

fn nested_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.as_table()?.get(*segment)?;
    }
    Some(current)
}

fn expected_crate_dirs() -> Vec<&'static str> {
    EXPECTED_CRATES
        .iter()
        .map(|(_, crate_dir)| *crate_dir)
        .collect()
}

fn matches_string_list(value: Option<&Value>, expected: &[&str]) -> bool {
    value.and_then(Value::as_array).is_some_and(|items| {
        items.len() == expected.len()
            && items
                .iter()
                .map(Value::as_str)
                .zip(expected.iter().copied())
                .all(|(actual, expected)| actual == Some(expected))
    })
}

fn is_string_list(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_array)
        .is_some_and(|items| items.iter().all(|item| matches!(item, Value::String(_))))
}

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn relative_path(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root).ok().map(slash_path)
}

fn relative_display(root: &Path, path: &Path) -> String {
    relative_path(root, path).unwrap_or_else(|| slash_path(path))
}

impl ExpectedValue {
    fn matches(self, value: Option<&Value>) -> bool {
        match self {
            Self::Integer(expected) => value.and_then(Value::as_integer) == Some(expected),
            Self::String(expected) => value.and_then(Value::as_str) == Some(expected),
            Self::StringList(expected) => matches_string_list(value, expected),
        }
    }

    fn python_repr(self) -> String {
        match self {
            Self::Integer(value) => value.to_string(),
            Self::String(value) => format!("'{value}'"),
            Self::StringList(values) => {
                let values = values
                    .iter()
                    .map(|value| format!("'{value}'"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{values}]")
            }
        }
    }
}

fn check_env_example(root: &Path) -> Result<(), Vec<String>> {
    let path = root.join(".env.example");
    if !path.is_file() {
        return Err(vec![".env.example is missing".to_owned()]);
    }

    let assignments = parse_assignments(&path)?;
    let expected: BTreeSet<&str> = EXPECTED_ENV_VARS.iter().copied().collect();
    let actual: BTreeSet<&str> = assignments.keys().map(String::as_str).collect();

    let mut errors = Vec::new();
    let missing = expected
        .difference(&actual)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    if !missing.is_empty() {
        errors.push(format!(
            ".env.example is missing provider variable(s): {missing}"
        ));
    }

    let unexpected = actual
        .difference(&expected)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    if !unexpected.is_empty() {
        errors.push(format!(
            ".env.example contains unexpected variable(s): {unexpected}"
        ));
    }

    let populated = assignments
        .iter()
        .filter_map(|(name, value)| (!value.is_empty()).then_some(name.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    if !populated.is_empty() {
        errors.push(format!(
            ".env.example must keep committed values blank: {populated}"
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_ci_workflow(root: &Path) -> Result<(), Vec<String>> {
    let path = root.join(".github").join("workflows").join("ci.yml");
    if !path.is_file() {
        return Err(vec![
            ".github/workflows/ci.yml: missing CI workflow".to_owned(),
        ]);
    }

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => return Err(vec![format!(".github/workflows/ci.yml: {error}")]),
    };

    let mut errors = Vec::new();
    for (description, snippet) in REQUIRED_CI_WORKFLOW_SNIPPETS {
        if !text.contains(snippet) {
            errors.push(format!(".github/workflows/ci.yml: missing {description}"));
        }
    }

    for (snippet, expected_count) in REQUIRED_CI_WORKFLOW_COUNTS {
        let actual_count = text.matches(snippet).count();
        if actual_count < *expected_count {
            errors.push(format!(
                ".github/workflows/ci.yml: expected at least {expected_count} occurrence(s) of `{snippet}`, found {actual_count}",
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_workflow_policies(root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for workflow_file in workflow_policy_files(root) {
        errors.extend(check_workflow_policy_file(root, &workflow_file));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn workflow_policy_files(root: &Path) -> Vec<PathBuf> {
    let workflows_dir = root.join(".github").join("workflows");
    let Ok(entries) = fs::read_dir(workflows_dir) else {
        return Vec::new();
    };

    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| WORKFLOW_SUFFIXES.contains(&extension))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn check_workflow_policy_file(root: &Path, path: &Path) -> Vec<String> {
    let relative_path = relative_display(root, path);
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => return vec![format!("{relative_path}: {error}")],
    };
    let lines = text.lines().collect::<Vec<_>>();
    let mut errors = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        let line_number = line_index + 1;
        let stripped = line.trim();
        if stripped.starts_with("pull_request_target:") {
            errors.push(format!(
                "{relative_path}:{line_number}: pull_request_target is not allowed"
            ));
        }
        if matches!(stripped, "permissions: read-all" | "permissions: write-all") {
            errors.push(format!(
                "{relative_path}:{line_number}: broad workflow permissions are not allowed"
            ));
        }

        if let Some(raw_reference) = workflow_uses_reference(line) {
            if let Some(error) =
                check_workflow_action_reference(&relative_path, line_number, raw_reference)
            {
                errors.push(error);
            }
            if is_checkout_action_reference(raw_reference)
                && !checkout_disables_persisted_credentials(&lines, line_index)
            {
                errors.push(format!(
                    "{relative_path}:{line_number}: checkout must set persist-credentials: false"
                ));
            }
        }
    }

    let Some(permissions) = parse_workflow_top_level_block(&lines, "permissions") else {
        errors.push(format!(
            "{relative_path}: missing top-level permissions block"
        ));
        return errors;
    };

    if let Some(concurrency) = parse_workflow_top_level_block(&lines, "concurrency") {
        if let Some(jobs_line) = first_workflow_top_level_key_line(&lines, "jobs:") {
            if concurrency.line > jobs_line {
                errors.push(format!(
                    "{relative_path}:{}: top-level concurrency must be before jobs",
                    concurrency.line
                ));
            }
        }

        if !concurrency.entries.contains_key("group") {
            errors.push(format!(
                "{relative_path}:{}: top-level concurrency must include group",
                concurrency.line
            ));
        }
        if !concurrency.entries.contains_key("cancel-in-progress") {
            errors.push(format!(
                "{relative_path}:{}: top-level concurrency must include cancel-in-progress",
                concurrency.line
            ));
        }
    } else {
        errors.push(format!(
            "{relative_path}: missing top-level concurrency block"
        ));
    }

    errors.extend(check_workflow_jobs(&relative_path, &lines));

    if let Some(jobs_line) = first_workflow_top_level_key_line(&lines, "jobs:") {
        if permissions.line > jobs_line {
            errors.push(format!(
                "{relative_path}:{}: top-level permissions must be before jobs",
                permissions.line
            ));
        }
    }

    match permissions.entries.get("contents") {
        Some((level, line_number)) if level == "read" => {}
        Some((_level, line_number)) => errors.push(format!(
            "{relative_path}:{line_number}: top-level contents permission must be read"
        )),
        None => errors.push(format!(
            "{relative_path}:{}: top-level permissions must include contents",
            permissions.line
        )),
    }

    for (scope, (level, line_number)) in &permissions.entries {
        if level == "write" && !ALLOWED_TOP_LEVEL_WRITE_SCOPES.contains(&scope.as_str()) {
            errors.push(format!(
                "{relative_path}:{line_number}: top-level {scope} write permission must be job-scoped"
            ));
        }
    }

    errors
}

fn workflow_uses_reference(line: &str) -> Option<&str> {
    let stripped = line.trim_start();
    stripped
        .strip_prefix("- uses:")
        .or_else(|| stripped.strip_prefix("uses:"))
        .map(str::trim)
        .filter(|reference| !reference.is_empty())
}

fn check_workflow_action_reference(
    relative_path: &str,
    line_number: usize,
    raw_reference: &str,
) -> Option<String> {
    let reference = raw_reference.trim().trim_matches(&['"', '\''][..]);
    if reference.starts_with("./") || reference.starts_with("docker://") {
        return None;
    }

    if reference.contains("${{") {
        return Some(format!(
            "{relative_path}:{line_number}: action references must not use expressions"
        ));
    }

    let Some((action, action_ref)) = reference.rsplit_once('@') else {
        return Some(format!(
            "{relative_path}:{line_number}: external action references must include an explicit ref"
        ));
    };
    if action.is_empty() || action_ref.is_empty() {
        return Some(format!(
            "{relative_path}:{line_number}: action reference must include action and ref"
        ));
    }

    let normalized_ref = action_ref.to_ascii_lowercase();
    if MUTABLE_ACTION_REFS.contains(&normalized_ref.as_str())
        || normalized_ref.starts_with("refs/heads/")
    {
        return Some(format!(
            "{relative_path}:{line_number}: action reference `{reference}` uses a mutable ref"
        ));
    }

    None
}

fn is_checkout_action_reference(raw_reference: &str) -> bool {
    let reference = raw_reference.trim().trim_matches(&['"', '\''][..]);
    reference
        .rsplit_once('@')
        .map(|(action, _)| action.eq_ignore_ascii_case("actions/checkout"))
        .unwrap_or(false)
}

fn checkout_disables_persisted_credentials(lines: &[&str], uses_line_index: usize) -> bool {
    let uses_line = lines[uses_line_index];
    let step_indent = leading_whitespace_len(uses_line);

    for line in lines.iter().skip(uses_line_index + 1) {
        let stripped = line.trim();
        let current_indent = leading_whitespace_len(line);

        if current_indent == step_indent && stripped.starts_with("- ") {
            break;
        }
        if current_indent < step_indent && !stripped.is_empty() {
            break;
        }
        if stripped == "persist-credentials: false" {
            return true;
        }
    }

    false
}

fn parse_workflow_top_level_block(lines: &[&str], key: &str) -> Option<WorkflowBlock> {
    let header = format!("{key}:");
    let start_index = lines.iter().position(|line| *line == header)?;
    let mut entries = BTreeMap::new();

    for (child_index, child) in lines.iter().enumerate().skip(start_index + 1) {
        if is_workflow_top_level_key(child) {
            break;
        }
        let Some((name, value)) = parse_workflow_block_entry(child) else {
            continue;
        };
        entries.insert(name.to_owned(), (value.to_owned(), child_index + 1));
    }

    Some(WorkflowBlock {
        line: start_index + 1,
        entries,
    })
}

fn parse_workflow_block_entry(line: &str) -> Option<(&str, &str)> {
    if !line.starts_with(char::is_whitespace) {
        return None;
    }
    let stripped = line.trim();
    let (key, value) = stripped.split_once(':')?;
    let key = key.trim();
    let value = value.trim();
    if !is_identifier(key) || value.is_empty() {
        return None;
    }
    Some((key, value))
}

fn check_workflow_jobs(relative_path: &str, lines: &[&str]) -> Vec<String> {
    let Some(jobs_line_index) = lines.iter().position(|line| *line == "jobs:") else {
        return Vec::new();
    };

    let mut errors = Vec::new();
    for job in parse_workflow_jobs(lines, jobs_line_index + 1) {
        match (&job.runs_on, job.runs_on_line) {
            (None, _) => errors.push(format!(
                "{relative_path}:{}: job `{}` missing runs-on",
                job.line, job.name
            )),
            (Some(runner), Some(line_number)) if FLOATING_RUNNERS.contains(&runner.as_str()) => {
                errors.push(format!(
                    "{relative_path}:{line_number}: job `{}` uses floating runner `{runner}`",
                    job.name
                ));
            }
            _ => {}
        }

        match (&job.timeout_minutes, job.timeout_line) {
            (None, _) => errors.push(format!(
                "{relative_path}:{}: job `{}` missing timeout-minutes",
                job.line, job.name
            )),
            (Some(value), Some(line_number)) => match value.parse::<i64>() {
                Ok(timeout) if (1..=60).contains(&timeout) => {}
                Ok(_) => errors.push(format!(
                    "{relative_path}:{line_number}: job `{}` timeout-minutes must be between 1 and 60",
                    job.name
                )),
                Err(_) => errors.push(format!(
                    "{relative_path}:{line_number}: job `{}` timeout-minutes must be an integer",
                    job.name
                )),
            },
            _ => {}
        }
    }

    errors
}

fn parse_workflow_jobs(lines: &[&str], start_index: usize) -> Vec<WorkflowJob> {
    let mut jobs = Vec::new();
    let mut index = start_index;
    while index < lines.len() {
        let line = lines[index];
        if is_workflow_top_level_key(line) {
            break;
        }

        let Some(name) = workflow_job_name(line) else {
            index += 1;
            continue;
        };

        let line_number = index + 1;
        let mut runs_on = None;
        let mut runs_on_line = None;
        let mut timeout_minutes = None;
        let mut timeout_line = None;
        index += 1;

        while index < lines.len() {
            let child = lines[index];
            if is_workflow_top_level_key(child) || workflow_job_name(child).is_some() {
                break;
            }

            let stripped = child.trim();
            if leading_whitespace_len(child) == 4 {
                if let Some(value) = stripped.strip_prefix("runs-on:") {
                    runs_on = Some(value.trim().trim_matches(&['"', '\''][..]).to_owned());
                    runs_on_line = Some(index + 1);
                }
                if let Some(value) = stripped.strip_prefix("timeout-minutes:") {
                    timeout_minutes = Some(value.trim().trim_matches(&['"', '\''][..]).to_owned());
                    timeout_line = Some(index + 1);
                }
            }

            index += 1;
        }

        jobs.push(WorkflowJob {
            name: name.to_owned(),
            line: line_number,
            runs_on,
            runs_on_line,
            timeout_minutes,
            timeout_line,
        });
    }

    jobs
}

fn workflow_job_name(line: &str) -> Option<&str> {
    if !line.starts_with("  ") || line.starts_with("    ") {
        return None;
    }
    let stripped = line.trim();
    let name = stripped.strip_suffix(':')?;
    if is_identifier(name) {
        Some(name)
    } else {
        None
    }
}

fn first_workflow_top_level_key_line(lines: &[&str], key: &str) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .find_map(|(index, line)| (*line == key).then_some(index + 1))
}

fn is_workflow_top_level_key(line: &str) -> bool {
    !line.is_empty()
        && !line.starts_with([' ', '\t'])
        && line
            .split_once(':')
            .map(|(key, _)| is_identifier(key))
            .unwrap_or(false)
}

fn leading_whitespace_len(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn check_security_workflows(root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for (relative_path, requirements) in SECURITY_WORKFLOW_REQUIREMENTS {
        let path = root.join(relative_path);
        if !path.is_file() {
            errors.push(format!("{relative_path}: missing security workflow"));
            continue;
        }

        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                errors.push(format!("{relative_path}: {error}"));
                continue;
            }
        };
        for (description, snippet) in *requirements {
            if !text.contains(snippet) {
                errors.push(format!("{relative_path}: missing {description}"));
            }
        }
    }

    let config_path = root.join(".github").join("dependency-review-config.yml");
    if !config_path.is_file() {
        errors.push(
            ".github/dependency-review-config.yml: missing dependency review policy".to_owned(),
        );
    } else {
        let config_text = match fs::read_to_string(&config_path) {
            Ok(text) => text,
            Err(error) => {
                errors.push(format!(".github/dependency-review-config.yml: {error}"));
                String::new()
            }
        };
        for (description, snippet) in DEPENDENCY_REVIEW_CONFIG_REQUIREMENTS {
            if !config_text.contains(snippet) {
                errors.push(format!(
                    ".github/dependency-review-config.yml: missing {description}"
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_sha256_file(artifact: &Path, checksum_file: Option<&Path>) -> Result<(), Vec<String>> {
    let checksum_path = checksum_file
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(format!("{}.sha256", artifact.to_string_lossy())));

    let artifact_bytes = match fs::read(artifact) {
        Ok(bytes) => bytes,
        Err(error) => return Err(vec![format!("Artifact cannot be read: {error}")]),
    };

    let checksum_output = match fs::read_to_string(&checksum_path) {
        Ok(text) => text.trim_start_matches('\u{feff}').to_owned(),
        Err(error) => return Err(vec![format!("Checksum file cannot be read: {error}")]),
    };

    let errors = check_sha256_output(
        artifact
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
        &artifact_bytes,
        &checksum_output,
    );
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_sha256_output(
    artifact_name: &str,
    artifact_bytes: &[u8],
    checksum_output: &str,
) -> Vec<String> {
    let lines = checksum_output.lines().collect::<Vec<_>>();
    if lines.len() != 1 || lines[0].trim().is_empty() {
        return vec!["Checksum file must contain exactly one checksum line".to_owned()];
    }

    let Some((digest, recorded_name)) = lines[0].split_once(char::is_whitespace) else {
        return vec![
            "Checksum line must contain a SHA-256 digest and artifact filename".to_owned(),
        ];
    };
    let recorded_name = recorded_name.trim_start();
    if recorded_name.is_empty() {
        return vec![
            "Checksum line must contain a SHA-256 digest and artifact filename".to_owned(),
        ];
    }
    let recorded_name = recorded_name.strip_prefix('*').unwrap_or(recorded_name);

    let mut errors = Vec::new();
    if !is_sha256_hex(digest) {
        errors.push("Checksum digest must be 64 hexadecimal characters".to_owned());
    }

    if recorded_name != artifact_name {
        errors.push(format!(
            "Checksum filename mismatch: expected {artifact_name}, got {recorded_name}"
        ));
    }

    let actual_digest = hex_sha256(artifact_bytes);
    let expected_digest = digest.to_ascii_lowercase();
    if expected_digest != actual_digest {
        errors.push(format!(
            "Checksum digest mismatch: expected {expected_digest}, got {actual_digest}"
        ));
    }

    errors
}

fn check_archive_contents(
    archive_directory: &Path,
    binary: &str,
    required_files: &[String],
) -> Result<(), Vec<String>> {
    if !archive_directory.is_dir() {
        return Err(vec![format!(
            "Archive directory is missing or is not a directory: {}",
            archive_directory.display()
        )]);
    }

    let mut errors = Vec::new();
    let mut expected_entries = BTreeSet::new();
    expected_entries.insert(binary.to_owned());
    expected_entries.extend(required_files.iter().cloned());

    let binary_path = archive_directory.join(binary);
    if !binary_path.exists() {
        errors.push(format!("Packaged archive binary is missing: {binary}"));
    } else if !binary_path.is_file() {
        errors.push(format!(
            "Packaged archive binary is not a regular file: {binary}"
        ));
    }

    for required_file in required_files {
        let path = archive_directory.join(required_file);
        if !path.exists() {
            errors.push(format!(
                "Packaged archive required file is missing: {required_file}"
            ));
        } else if !path.is_file() {
            errors.push(format!(
                "Packaged archive required file is not a regular file: {required_file}"
            ));
        }
    }

    let mut entries = match fs::read_dir(archive_directory) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        Err(error) => {
            errors.push(format!(
                "Archive directory cannot be read: {}: {error}",
                archive_directory.display()
            ));
            Vec::new()
        }
    };
    entries.sort();
    for entry in entries {
        if !expected_entries.contains(&entry) {
            errors.push(format!(
                "Packaged archive contains unexpected entry: {entry}"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_schema_files(root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for (relative_path, title, required_fields) in EXPECTED_SCHEMAS {
        let path = root.join(relative_path);
        if !path.is_file() {
            errors.push(format!("{relative_path}: missing schema file"));
            continue;
        }

        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                errors.push(format!("{relative_path}: {error}"));
                continue;
            }
        };
        let schema = match serde_json::from_str::<JsonValue>(&text) {
            Ok(value) => value,
            Err(error) => {
                errors.push(format!("{relative_path}: invalid JSON: {error}"));
                continue;
            }
        };
        let Some(schema) = schema.as_object() else {
            errors.push(format!("{relative_path}: schema root must be an object"));
            continue;
        };

        errors.extend(check_schema_document(
            relative_path,
            schema,
            title,
            required_fields,
        ));
    }

    errors.extend(check_workflow_fixtures(root));
    errors.extend(check_replay_fixtures(root));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_schema_document(
    relative_path: &str,
    schema: &serde_json::Map<String, JsonValue>,
    title: &str,
    required_fields: &[&str],
) -> Vec<String> {
    let mut errors = Vec::new();

    if json_string(schema.get("$schema")) != Some(SCHEMA_DRAFT) {
        errors.push(format!(
            "{relative_path}: schema draft must be '{}'",
            SCHEMA_DRAFT
        ));
    }
    if json_string(schema.get("title")) != Some(title) {
        errors.push(format!("{relative_path}: title must be '{title}'"));
    }
    if json_string(schema.get("type")) != Some("object") {
        errors.push(format!("{relative_path}: root type must be 'object'"));
    }
    if schema.get("additionalProperties") != Some(&JsonValue::Bool(false)) {
        errors.push(format!(
            "{relative_path}: root additionalProperties must be false"
        ));
    }
    if !json_string_array_equals(schema.get("required"), required_fields) {
        errors.push(format!(
            "{relative_path}: required fields must match documented format"
        ));
    }

    let Some(properties) = schema.get("properties").and_then(JsonValue::as_object) else {
        errors.push(format!("{relative_path}: missing root properties"));
        return errors;
    };
    for field in required_fields {
        if !properties.contains_key(*field) {
            errors.push(format!("{relative_path}: missing `{field}` property"));
        }
    }

    errors
}

fn check_workflow_fixtures(root: &Path) -> Vec<String> {
    let workflows_dir = root.join("fixtures").join("workflows");
    if !workflows_dir.is_dir() {
        return vec!["fixtures/workflows: missing workflow fixtures".to_owned()];
    }

    let mut errors = Vec::new();
    let fixture_paths = match sorted_files_with_extension(&workflows_dir, "toml") {
        Ok(paths) => paths,
        Err(error) => return vec![format!("fixtures/workflows: {error}")],
    };
    if fixture_paths.is_empty() {
        errors.push("fixtures/workflows: missing workflow fixture files".to_owned());
    }

    for path in fixture_paths {
        let relative_path = relative_display(root, &path);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                errors.push(format!("{relative_path}: {error}"));
                continue;
            }
        };
        let workflow = match text.parse::<TomlTable>() {
            Ok(table) => table,
            Err(error) => {
                errors.push(format!("{relative_path}: invalid TOML: {error}"));
                continue;
            }
        };

        errors.extend(check_workflow_document(&relative_path, &workflow));
    }

    errors
}

fn check_workflow_document(relative_path: &str, workflow: &TomlTable) -> Vec<String> {
    let mut errors = Vec::new();
    for field in sorted_unknown_fields(workflow.keys(), &["name", "steps"]) {
        errors.push(format!("{relative_path}: unknown workflow field `{field}`"));
    }

    if !is_identifier_toml(workflow.get("name")) {
        errors.push(format!(
            "{relative_path}: workflow name must use {IDENTIFIER_PATTERN_DESCRIPTION}"
        ));
    }

    let Some(steps) = workflow.get("steps").and_then(Value::as_array) else {
        errors.push(format!(
            "{relative_path}: workflow steps must be a non-empty list"
        ));
        return errors;
    };
    if steps.is_empty() {
        errors.push(format!(
            "{relative_path}: workflow steps must be a non-empty list"
        ));
        return errors;
    }

    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_table() else {
            errors.push(format!(
                "{relative_path}: workflow step {index} must be an object"
            ));
            continue;
        };
        for field in sorted_unknown_fields(step.keys(), &["id", "prompt"]) {
            errors.push(format!(
                "{relative_path}: workflow step {index} has unknown field `{field}`"
            ));
        }
        if !is_identifier_toml(step.get("id")) {
            errors.push(format!(
                "{relative_path}: workflow step {index} id must use {IDENTIFIER_PATTERN_DESCRIPTION}"
            ));
        }
        if !is_non_empty_toml_string(step.get("prompt")) {
            errors.push(format!(
                "{relative_path}: workflow step {index} prompt must be non-empty"
            ));
        }
    }

    errors
}

fn check_replay_fixtures(root: &Path) -> Vec<String> {
    let replays_dir = root.join("fixtures").join("replays");
    if !replays_dir.is_dir() {
        return vec!["fixtures/replays: missing replay fixtures".to_owned()];
    }

    let mut errors = Vec::new();
    let fixture_paths = match sorted_files_with_suffix(&replays_dir, ".replay.json") {
        Ok(paths) => paths,
        Err(error) => return vec![format!("fixtures/replays: {error}")],
    };
    if fixture_paths.is_empty() {
        errors.push("fixtures/replays: missing replay fixture files".to_owned());
    }

    for path in fixture_paths {
        let relative_path = relative_display(root, &path);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                errors.push(format!("{relative_path}: {error}"));
                continue;
            }
        };
        let replay = match serde_json::from_str::<JsonValue>(&text) {
            Ok(JsonValue::Object(object)) => object,
            Ok(_) => {
                errors.push(format!("{relative_path}: replay fixture must be an object"));
                continue;
            }
            Err(error) => {
                errors.push(format!("{relative_path}: invalid JSON: {error}"));
                continue;
            }
        };

        errors.extend(check_replay_document(&relative_path, &replay));
    }

    errors
}

fn check_replay_document(
    relative_path: &str,
    replay: &serde_json::Map<String, JsonValue>,
) -> Vec<String> {
    let mut errors = Vec::new();
    let allowed_fields = [
        "schema_version",
        "workflow_name",
        "runtime",
        "run_hash",
        "steps",
    ];

    for field in sorted_unknown_fields(replay.keys(), &allowed_fields) {
        errors.push(format!("{relative_path}: unknown replay field `{field}`"));
    }
    for field in allowed_fields {
        if !replay.contains_key(field) {
            errors.push(format!("{relative_path}: missing replay field `{field}`"));
        }
    }

    if replay.get("schema_version").and_then(JsonValue::as_i64) != Some(1) {
        errors.push(format!("{relative_path}: replay schema_version must be 1"));
    }
    if !is_identifier_json(replay.get("workflow_name")) {
        errors.push(format!(
            "{relative_path}: replay workflow_name must use {IDENTIFIER_PATTERN_DESCRIPTION}"
        ));
    }
    if !is_sha256_json(replay.get("run_hash")) {
        errors.push(format!(
            "{relative_path}: replay run_hash must be lowercase sha256"
        ));
    }

    match replay.get("runtime").and_then(JsonValue::as_object) {
        Some(runtime) => errors.extend(check_runtime_metadata(relative_path, runtime)),
        None => errors.push(format!("{relative_path}: replay runtime must be an object")),
    }

    let Some(steps) = replay.get("steps").and_then(JsonValue::as_array) else {
        errors.push(format!(
            "{relative_path}: replay steps must be a non-empty list"
        ));
        return errors;
    };
    if steps.is_empty() {
        errors.push(format!(
            "{relative_path}: replay steps must be a non-empty list"
        ));
        return errors;
    }

    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_object() else {
            errors.push(format!(
                "{relative_path}: replay step {index} must be an object"
            ));
            continue;
        };
        errors.extend(check_replay_step(relative_path, index, step));
    }

    errors
}

fn check_runtime_metadata(
    relative_path: &str,
    runtime: &serde_json::Map<String, JsonValue>,
) -> Vec<String> {
    let mut errors = Vec::new();
    let allowed_fields = [
        "provider",
        "adapter",
        "adapter_version",
        "model",
        "cache_identity",
        "parameters",
    ];
    let required_fields = ["provider", "adapter", "adapter_version", "cache_identity"];

    for field in sorted_unknown_fields(runtime.keys(), &allowed_fields) {
        errors.push(format!("{relative_path}: unknown runtime field `{field}`"));
    }
    for field in required_fields {
        if !is_non_empty_json_string(runtime.get(field)) {
            errors.push(format!(
                "{relative_path}: runtime `{field}` must be non-empty"
            ));
        }
    }

    if runtime.contains_key("model") && !is_non_empty_json_string(runtime.get("model")) {
        errors.push(format!(
            "{relative_path}: runtime `model` must be non-empty when present"
        ));
    }

    match runtime.get("parameters") {
        None => {}
        Some(JsonValue::Object(parameters))
            if parameters
                .iter()
                .any(|(key, value)| key.is_empty() || !is_non_empty_json_string(Some(value))) =>
        {
            errors.push(format!(
                "{relative_path}: runtime parameters must use non-empty string keys and values"
            ));
        }
        Some(JsonValue::Object(_)) => {}
        Some(_) => errors.push(format!(
            "{relative_path}: runtime `parameters` must be an object"
        )),
    }

    errors
}

fn check_replay_step(
    relative_path: &str,
    index: usize,
    step: &serde_json::Map<String, JsonValue>,
) -> Vec<String> {
    let mut errors = Vec::new();
    for field in sorted_unknown_fields(
        step.keys(),
        &["step_id", "input_hash", "output_hash", "output"],
    ) {
        errors.push(format!(
            "{relative_path}: replay step {index} has unknown field `{field}`"
        ));
    }
    if !is_identifier_json(step.get("step_id")) {
        errors.push(format!(
            "{relative_path}: replay step {index} id must use {IDENTIFIER_PATTERN_DESCRIPTION}"
        ));
    }
    if !is_sha256_json(step.get("input_hash")) {
        errors.push(format!(
            "{relative_path}: replay step {index} input_hash must be lowercase sha256"
        ));
    }
    if !is_sha256_json(step.get("output_hash")) {
        errors.push(format!(
            "{relative_path}: replay step {index} output_hash must be lowercase sha256"
        ));
    }
    if !matches!(step.get("output"), Some(JsonValue::String(_))) {
        errors.push(format!(
            "{relative_path}: replay step {index} output must be a string"
        ));
    }
    errors
}

fn sorted_files_with_extension(
    dir: &Path,
    extension: &str,
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn sorted_files_with_suffix(dir: &Path, suffix: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if slash_path(&path).ends_with(suffix) {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn sorted_unknown_fields<'a>(
    fields: impl Iterator<Item = &'a String>,
    allowed_fields: &[&str],
) -> Vec<&'a str> {
    let allowed = allowed_fields.iter().copied().collect::<BTreeSet<_>>();
    let mut unknown = fields
        .map(String::as_str)
        .filter(|field| !allowed.contains(*field))
        .collect::<Vec<_>>();
    unknown.sort_unstable();
    unknown
}

fn json_string(value: Option<&JsonValue>) -> Option<&str> {
    value.and_then(JsonValue::as_str)
}

fn json_string_array_equals(value: Option<&JsonValue>, expected: &[&str]) -> bool {
    let Some(values) = value.and_then(JsonValue::as_array) else {
        return false;
    };
    values.len() == expected.len()
        && values
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.as_str() == Some(*expected))
}

fn is_identifier_toml(value: Option<&Value>) -> bool {
    value.and_then(Value::as_str).is_some_and(is_identifier)
}

fn is_identifier_json(value: Option<&JsonValue>) -> bool {
    value.and_then(JsonValue::as_str).is_some_and(is_identifier)
}

fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn is_non_empty_toml_string(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|value| !value.is_empty())
}

fn is_non_empty_json_string(value: Option<&JsonValue>) -> bool {
    value
        .and_then(JsonValue::as_str)
        .is_some_and(|value| !value.is_empty())
}

fn is_sha256_json(value: Option<&JsonValue>) -> bool {
    value.and_then(JsonValue::as_str).is_some_and(is_sha256)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| matches!(character, '0'..='9' | 'a'..='f'))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

fn parse_assignments(path: &Path) -> Result<BTreeMap<String, String>, Vec<String>> {
    let text = fs::read_to_string(path).map_err(|error| vec![error.to_string()])?;
    let mut assignments = BTreeMap::new();

    for (index, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            return Err(vec![format!(
                "{}:{}: expected KEY=VALUE assignment",
                path.display(),
                index + 1
            )]);
        };
        assignments.insert(name.trim().to_owned(), value.trim().to_owned());
    }

    Ok(assignments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn accepts_contributing_doc_with_readme_checks_and_extra_commands() {
        let root = temp_root("contributing-accepts");
        write_contributing_docs(
            &root,
            &["cargo test", "python scripts/check_docs_links.py --root ."],
            &[
                "cargo test",
                "python scripts/check_docs_links.py --root .",
                "docker build --tag vogon-runtime:smoke .",
            ],
            live_guidance_text(),
        );

        assert_eq!(check_contributing_checklist(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_contributing_doc_command() {
        let root = temp_root("contributing-missing-command");
        write_contributing_docs(
            &root,
            &["cargo test", "cargo clippy -- -D warnings"],
            &["cargo test"],
            live_guidance_text(),
        );

        let errors = check_contributing_checklist(&root).unwrap_err();

        assert_eq!(
            errors,
            ["CONTRIBUTING.md: missing README local check `cargo clippy -- -D warnings`",]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_live_workflow_guidance() {
        let root = temp_root("contributing-missing-live-guidance");
        write_contributing_docs(
            &root,
            &["cargo test"],
            &["cargo test"],
            &live_guidance_text().replace(
                "- `Live OpenAI-Compatible Smoke` uses `OPENAI_COMPATIBLE_API_KEY`.\n",
                "",
            ),
        );

        let errors = check_contributing_checklist(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "CONTRIBUTING.md: missing `Live OpenAI-Compatible Smoke` guidance",
                "CONTRIBUTING.md: missing `OPENAI_COMPATIBLE_API_KEY` live smoke secret guidance",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_contributing_command_blocks() {
        let root = temp_root("contributing-missing-blocks");
        fs::write(root.join("README.md"), "# README\n").unwrap();
        fs::write(
            root.join("CONTRIBUTING.md"),
            format!("# Contributing\n{}", live_guidance_text()),
        )
        .unwrap();

        let errors = check_contributing_checklist(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "README.md: missing local check command block",
                "CONTRIBUTING.md: missing development command block",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_ci_workflow_contract() {
        let root = temp_root("ci-workflow-accepts");
        write_ci_workflow(&root, ci_workflow_text());

        assert_eq!(check_ci_workflow(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_ci_workflow() {
        let root = temp_root("ci-workflow-missing");

        let errors = check_ci_workflow(&root).unwrap_err();

        assert_eq!(errors, [".github/workflows/ci.yml: missing CI workflow"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_required_ci_command() {
        let root = temp_root("ci-workflow-missing-command");
        write_ci_workflow(
            &root,
            &ci_workflow_text().replace(
                "cargo run -p vogon-xtask -- check-ci-workflow --root .",
                "python3 scripts/check_other_workflow.py --root .",
            ),
        );

        let errors = check_ci_workflow(&root).unwrap_err();

        assert!(
            errors
                .iter()
                .any(|error| error == ".github/workflows/ci.yml: missing CI workflow validator")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_required_ci_occurrence_count() {
        let root = temp_root("ci-workflow-missing-count");
        write_ci_workflow(
            &root,
            &ci_workflow_text().replace("uses: actions/checkout@v7", "uses: actions/checkout@v6"),
        );

        let errors = check_ci_workflow(&root).unwrap_err();

        assert!(errors.iter().any(|error| error
            == ".github/workflows/ci.yml: expected at least 4 occurrence(s) of `uses: actions/checkout@v7`, found 0"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_expected_security_workflows() {
        let root = temp_root("security-workflows-accepts");
        write_security_workflows(&root, None, None, None, None);

        assert_eq!(check_security_workflows(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_security_workflows() {
        let root = temp_root("security-workflows-missing");

        let errors = check_security_workflows(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                ".github/workflows/codeql.yml: missing security workflow",
                ".github/workflows/security-audit.yml: missing security workflow",
                ".github/workflows/dependency-review.yml: missing security workflow",
                ".github/dependency-review-config.yml: missing dependency review policy",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_codeql_extended_queries() {
        let root = temp_root("security-workflows-missing-codeql-query");
        write_security_workflows(
            &root,
            Some(codeql_workflow_text().replace(
                "          queries: security-extended,security-and-quality\n",
                "",
            )),
            None,
            None,
            None,
        );

        assert_eq!(
            check_security_workflows(&root).unwrap_err(),
            [".github/workflows/codeql.yml: missing extended security queries"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_rustsec_schedule() {
        let root = temp_root("security-workflows-missing-rustsec-schedule");
        write_security_workflows(
            &root,
            None,
            Some(security_audit_workflow_text().replace("    - cron: \"17 4 * * 1\"\n", "")),
            None,
            None,
        );

        assert_eq!(
            check_security_workflows(&root).unwrap_err(),
            [".github/workflows/security-audit.yml: missing scheduled audit"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_dependency_review_config_reference() {
        let root = temp_root("security-workflows-missing-review-config-reference");
        write_security_workflows(
            &root,
            None,
            None,
            Some(dependency_review_workflow_text().replace(
                "          config-file: ./.github/dependency-review-config.yml",
                "          fail-on-severity: high",
            )),
            None,
        );

        assert_eq!(
            check_security_workflows(&root).unwrap_err(),
            [".github/workflows/dependency-review.yml: missing dependency review config file"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_dependency_review_concurrency() {
        let root = temp_root("security-workflows-missing-review-concurrency");
        write_security_workflows(
            &root,
            None,
            None,
            Some(dependency_review_workflow_text().replace(
                "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}\n  cancel-in-progress: true\n\n",
                "",
            )),
            None,
        );

        assert_eq!(
            check_security_workflows(&root).unwrap_err(),
            [
                ".github/workflows/dependency-review.yml: missing concurrency group",
                ".github/workflows/dependency-review.yml: missing stale run cancellation",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_disabled_dependency_review_license_check() {
        let root = temp_root("security-workflows-disabled-license-check");
        write_security_workflows(
            &root,
            None,
            None,
            None,
            Some(
                dependency_review_config_text()
                    .replace("license-check: true", "license-check: false"),
            ),
        );

        assert_eq!(
            check_security_workflows(&root).unwrap_err(),
            [".github/dependency-review-config.yml: missing license checks enabled"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_removed_dependency_review_license_allowlist_entry() {
        let root = temp_root("security-workflows-missing-license");
        write_security_workflows(
            &root,
            None,
            None,
            None,
            Some(dependency_review_config_text().replace("  - CDLA-Permissive-2.0\n", "")),
        );

        assert_eq!(
            check_security_workflows(&root).unwrap_err(),
            [".github/dependency-review-config.yml: missing CDLA permissive license allowed"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_matching_sha256_output() {
        let artifact_bytes = b"release artifact";
        let digest = hex_sha256(artifact_bytes);

        assert_eq!(
            check_sha256_output(
                "vogon.tar.gz",
                artifact_bytes,
                &format!("{digest}  vogon.tar.gz\n"),
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn accepts_binary_marker_from_sha256sum() {
        let artifact_bytes = b"release artifact";
        let digest = hex_sha256(artifact_bytes);

        assert_eq!(
            check_sha256_output(
                "vogon.tar.gz",
                artifact_bytes,
                &format!("{digest} *vogon.tar.gz\n"),
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn accepts_artifact_and_default_checksum_paths() {
        let root = temp_root("sha256-default-path");
        let artifact = root.join("vogon.zip");
        fs::write(&artifact, b"release artifact").unwrap();
        let digest = hex_sha256(&fs::read(&artifact).unwrap());
        fs::write(
            PathBuf::from(format!("{}.sha256", artifact.to_string_lossy())),
            format!("{digest}  vogon.zip"),
        )
        .unwrap();

        assert_eq!(check_sha256_file(&artifact, None), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_sha256_artifact() {
        let root = temp_root("sha256-missing-artifact");

        let errors = check_sha256_file(&root.join("missing.tar.gz"), None).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("Artifact cannot be read:"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_bad_sha256_format() {
        assert_eq!(
            check_sha256_output("vogon.tar.gz", b"release artifact", "not-a-checksum\n"),
            ["Checksum line must contain a SHA-256 digest and artifact filename"]
        );
    }

    #[test]
    fn reports_extra_sha256_lines() {
        assert_eq!(
            check_sha256_output("vogon.tar.gz", b"release artifact", "first\nsecond\n"),
            ["Checksum file must contain exactly one checksum line"]
        );
    }

    #[test]
    fn reports_invalid_sha256_digest() {
        let actual_digest = hex_sha256(b"release artifact");

        assert_eq!(
            check_sha256_output("vogon.tar.gz", b"release artifact", "abc  vogon.tar.gz\n"),
            [
                "Checksum digest must be 64 hexadecimal characters",
                &format!("Checksum digest mismatch: expected abc, got {actual_digest}"),
            ]
        );
    }

    #[test]
    fn reports_sha256_filename_mismatch() {
        let artifact_bytes = b"release artifact";
        let digest = hex_sha256(artifact_bytes);

        assert_eq!(
            check_sha256_output(
                "vogon.tar.gz",
                artifact_bytes,
                &format!("{digest}  other.tar.gz\n"),
            ),
            ["Checksum filename mismatch: expected vogon.tar.gz, got other.tar.gz"]
        );
    }

    #[test]
    fn reports_sha256_digest_mismatch() {
        let wrong_digest = hex_sha256(b"other artifact");
        let actual_digest = hex_sha256(b"release artifact");

        assert_eq!(
            check_sha256_output(
                "vogon.tar.gz",
                b"release artifact",
                &format!("{wrong_digest}  vogon.tar.gz\n"),
            ),
            [format!(
                "Checksum digest mismatch: expected {wrong_digest}, got {actual_digest}"
            )]
        );
    }

    #[test]
    fn accepts_least_privilege_workflow_with_job_scoped_write() {
        let root = temp_root("workflow-policy-accepts");
        write_workflow_policy_file(
            &root,
            "release.yml",
            &[
                "name: Release",
                "on:",
                "  workflow_dispatch:",
                "permissions:",
                "  contents: read",
                "concurrency:",
                "  group: ${{ github.workflow }}-${{ github.ref }}",
                "  cancel-in-progress: true",
                "jobs:",
                "  publish:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 10",
                "    steps:",
                "      - uses: actions/checkout@v7",
                "        with:",
                "          persist-credentials: false",
                "      - uses: github/codeql-action/analyze@v4",
                "      - uses: docker://alpine:3.20",
                "      - uses: ./github/actions/local-check",
                "    permissions:",
                "      contents: write",
            ],
        );

        assert_eq!(check_workflow_policies(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_top_level_workflow_permissions() {
        let root = temp_root("workflow-policy-missing-permissions");
        write_workflow_policy_file(
            &root,
            "ci.yml",
            &["name: CI", "on:", "  pull_request:", "jobs:"],
        );

        assert_eq!(
            check_workflow_policies(&root).unwrap_err(),
            [".github/workflows/ci.yml: missing top-level permissions block"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_pull_request_target_and_broad_permissions() {
        let root = temp_root("workflow-policy-broad-permissions");
        write_workflow_policy_file(
            &root,
            "ci.yml",
            &[
                "name: CI",
                "on:",
                "  pull_request_target:",
                "permissions: write-all",
                "concurrency:",
                "  group: ${{ github.workflow }}-${{ github.ref }}",
                "  cancel-in-progress: true",
                "jobs:",
                "  test:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 10",
            ],
        );

        let errors = check_workflow_policies(&root).unwrap_err();

        assert!(
            errors
                .iter()
                .any(|error| error
                    == ".github/workflows/ci.yml:3: pull_request_target is not allowed")
        );
        assert!(errors.iter().any(|error| {
            error == ".github/workflows/ci.yml:4: broad workflow permissions are not allowed"
        }));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_top_level_write_permissions_except_security_events() {
        let root = temp_root("workflow-policy-write-permissions");
        write_workflow_policy_file(
            &root,
            "ci.yml",
            &[
                "name: CI",
                "on:",
                "  push:",
                "permissions:",
                "  contents: write",
                "  security-events: write",
                "concurrency:",
                "  group: ${{ github.workflow }}-${{ github.ref }}",
                "  cancel-in-progress: true",
                "jobs:",
                "  test:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 10",
            ],
        );

        let errors = check_workflow_policies(&root).unwrap_err();

        assert!(errors.iter().any(|error| {
            error == ".github/workflows/ci.yml:5: top-level contents permission must be read"
        }));
        assert!(errors.iter().any(|error| {
            error
                == ".github/workflows/ci.yml:5: top-level contents write permission must be job-scoped"
        }));
        assert!(!errors.iter().any(|error| {
            error
                == ".github/workflows/ci.yml:6: top-level security-events write permission must be job-scoped"
        }));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_floating_runner_and_missing_timeout() {
        let root = temp_root("workflow-policy-floating-runner");
        write_workflow_policy_file(
            &root,
            "ci.yml",
            &[
                "name: CI",
                "on:",
                "  pull_request:",
                "permissions:",
                "  contents: read",
                "concurrency:",
                "  group: ${{ github.workflow }}-${{ github.ref }}",
                "  cancel-in-progress: true",
                "jobs:",
                "  test:",
                "    runs-on: ubuntu-latest",
            ],
        );

        let errors = check_workflow_policies(&root).unwrap_err();

        assert!(errors.iter().any(|error| {
            error == ".github/workflows/ci.yml:11: job `test` uses floating runner `ubuntu-latest`"
        }));
        assert!(errors.iter().any(|error| {
            error == ".github/workflows/ci.yml:10: job `test` missing timeout-minutes"
        }));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_timeout_values() {
        let root = temp_root("workflow-policy-invalid-timeout");
        write_workflow_policy_file(
            &root,
            "ci.yml",
            &[
                "name: CI",
                "on:",
                "  pull_request:",
                "permissions:",
                "  contents: read",
                "concurrency:",
                "  group: ${{ github.workflow }}-${{ github.ref }}",
                "  cancel-in-progress: true",
                "jobs:",
                "  slow:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 90",
                "  invalid:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: soon",
            ],
        );

        let errors = check_workflow_policies(&root).unwrap_err();

        assert!(errors.iter().any(|error| {
            error == ".github/workflows/ci.yml:12: job `slow` timeout-minutes must be between 1 and 60"
        }));
        assert!(errors.iter().any(|error| {
            error == ".github/workflows/ci.yml:15: job `invalid` timeout-minutes must be an integer"
        }));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_unpinned_and_mutable_action_refs() {
        let root = temp_root("workflow-policy-action-refs");
        write_workflow_policy_file(
            &root,
            "ci.yml",
            &[
                "name: CI",
                "on:",
                "  pull_request:",
                "permissions:",
                "  contents: read",
                "concurrency:",
                "  group: ${{ github.workflow }}-${{ github.ref }}",
                "  cancel-in-progress: true",
                "jobs:",
                "  test:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 10",
                "    steps:",
                "      - uses: actions/checkout",
                "      - uses: github/codeql-action/analyze@main",
                "      - uses: actions/cache@refs/heads/main",
                "      - uses: actions/upload-artifact@${{ inputs.ref }}",
            ],
        );

        let errors = check_workflow_policies(&root).unwrap_err();

        assert!(errors.iter().any(|error| {
            error
                == ".github/workflows/ci.yml:14: external action references must include an explicit ref"
        }));
        assert!(errors.iter().any(|error| {
            error
                == ".github/workflows/ci.yml:15: action reference `github/codeql-action/analyze@main` uses a mutable ref"
        }));
        assert!(errors.iter().any(|error| {
            error
                == ".github/workflows/ci.yml:16: action reference `actions/cache@refs/heads/main` uses a mutable ref"
        }));
        assert!(errors.iter().any(|error| {
            error == ".github/workflows/ci.yml:17: action references must not use expressions"
        }));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_checkout_with_persisted_credentials() {
        let root = temp_root("workflow-policy-checkout-credentials");
        write_workflow_policy_file(
            &root,
            "checkout.yml",
            &[
                "name: Checkout",
                "on:",
                "  pull_request:",
                "permissions:",
                "  contents: read",
                "concurrency:",
                "  group: ${{ github.workflow }}-${{ github.ref }}",
                "  cancel-in-progress: true",
                "jobs:",
                "  missing:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 10",
                "    steps:",
                "      - uses: actions/checkout@v7",
                "  enabled:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 10",
                "    steps:",
                "      - uses: actions/checkout@v7",
                "        with:",
                "          persist-credentials: true",
            ],
        );

        assert_eq!(
            check_workflow_policies(&root).unwrap_err(),
            [
                ".github/workflows/checkout.yml:14: checkout must set persist-credentials: false",
                ".github/workflows/checkout.yml:19: checkout must set persist-credentials: false",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_missing_and_incomplete_concurrency_policy() {
        let root = temp_root("workflow-policy-concurrency");
        write_workflow_policy_file(
            &root,
            "missing.yml",
            &[
                "name: Missing",
                "on:",
                "  pull_request:",
                "permissions:",
                "  contents: read",
                "jobs:",
                "  test:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 10",
            ],
        );
        write_workflow_policy_file(
            &root,
            "incomplete.yml",
            &[
                "name: Incomplete",
                "on:",
                "  pull_request:",
                "permissions:",
                "  contents: read",
                "concurrency:",
                "  group: ${{ github.workflow }}-${{ github.ref }}",
                "jobs:",
                "  test:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 10",
            ],
        );
        write_workflow_policy_file(
            &root,
            "late.yml",
            &[
                "name: Late",
                "on:",
                "  pull_request:",
                "permissions:",
                "  contents: read",
                "jobs:",
                "  test:",
                "    runs-on: ubuntu-24.04",
                "    timeout-minutes: 10",
                "concurrency:",
                "  group: ${{ github.workflow }}-${{ github.ref }}",
                "  cancel-in-progress: true",
            ],
        );

        let errors = check_workflow_policies(&root).unwrap_err();

        assert!(errors.iter().any(|error| {
            error == ".github/workflows/missing.yml: missing top-level concurrency block"
        }));
        assert!(errors.iter().any(|error| {
            error
                == ".github/workflows/incomplete.yml:6: top-level concurrency must include cancel-in-progress"
        }));
        assert!(errors.iter().any(|error| {
            error == ".github/workflows/late.yml:10: top-level concurrency must be before jobs"
        }));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_pr_template_with_readme_checks_and_extra_commands() {
        let root = temp_root("pr-template-accepts");
        write_pr_template_docs(
            &root,
            &["cargo test", "python scripts/check_docs_links.py --root ."],
            &[
                "cargo test",
                "python scripts/check_docs_links.py --root .",
                "docker build --tag vogon-runtime:smoke .",
            ],
        );

        assert_eq!(check_pr_template(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_pr_template_command() {
        let root = temp_root("pr-template-missing-command");
        write_pr_template_docs(
            &root,
            &["cargo test", "cargo clippy -- -D warnings"],
            &["cargo test"],
        );

        let errors = check_pr_template(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                ".github/pull_request_template.md: missing README local check `cargo clippy -- -D warnings`",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_pr_template_command_blocks() {
        let root = temp_root("pr-template-missing-blocks");
        fs::create_dir(root.join(".github")).unwrap();
        fs::write(root.join("README.md"), "# README\n").unwrap();
        fs::write(
            root.join(".github/pull_request_template.md"),
            "## Verification\n\n- [ ] Relevant CLI smoke test:\n",
        )
        .unwrap();

        let errors = check_pr_template(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "README.md: missing local check command block",
                ".github/pull_request_template.md: missing verification command checklist",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_release_doc_with_readme_checks_and_extra_commands() {
        let root = temp_root("release-checklist-accepts");
        write_release_docs(
            &root,
            &["cargo test", "python scripts/check_docs_links.py --root ."],
            &[
                "cargo test",
                "python scripts/check_docs_links.py --root .",
                "docker build --tag vogon-runtime:smoke .",
            ],
        );

        assert_eq!(check_release_checklist(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_release_doc_command() {
        let root = temp_root("release-checklist-missing-command");
        write_release_docs(
            &root,
            &["cargo test", "cargo clippy -- -D warnings"],
            &["cargo test"],
        );

        let errors = check_release_checklist(&root).unwrap_err();

        assert_eq!(
            errors,
            ["docs/release.md: missing README local check `cargo clippy -- -D warnings`",]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_release_checklist_command_blocks() {
        let root = temp_root("release-checklist-missing-blocks");
        fs::create_dir(root.join("docs")).unwrap();
        fs::write(root.join("README.md"), "# README\n").unwrap();
        fs::write(root.join("docs").join("release.md"), "# Release\n").unwrap();

        let errors = check_release_checklist(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "README.md: missing local check command block",
                "docs/release.md: missing release verification command block",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_docs_with_deployment_commands_in_readme_and_release() {
        let root = temp_root("deployment-checklist-accepts");
        write_deployment_docs(
            &root,
            &[
                "docker build --tag vogon-runtime:smoke .",
                "docker run --rm vogon-runtime:smoke --version",
            ],
            &[
                "cargo test",
                "docker build --tag vogon-runtime:smoke .",
                "docker run --rm vogon-runtime:smoke --version",
            ],
            &[
                "cargo test",
                "docker build --tag vogon-runtime:smoke .",
                "docker run --rm vogon-runtime:smoke --version",
            ],
        );

        assert_eq!(check_deployment_checklist(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_deployment_commands_missing_from_readme_and_release() {
        let root = temp_root("deployment-checklist-missing-commands");
        write_deployment_docs(
            &root,
            &[
                "docker build --tag vogon-runtime:smoke .",
                "docker run --rm vogon-runtime:smoke --version",
            ],
            &["docker build --tag vogon-runtime:smoke ."],
            &["cargo test"],
        );

        let errors = check_deployment_checklist(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "docs/release.md: missing deployment smoke command `docker build --tag vogon-runtime:smoke .`",
                "README.md: missing deployment smoke command `docker run --rm vogon-runtime:smoke --version`",
                "docs/release.md: missing deployment smoke command `docker run --rm vogon-runtime:smoke --version`",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_deployment_checklist_command_blocks() {
        let root = temp_root("deployment-checklist-missing-blocks");
        fs::create_dir(root.join("docs")).unwrap();
        fs::write(root.join("README.md"), "# README\n").unwrap();
        fs::write(root.join("docs").join("release.md"), "# Release\n").unwrap();
        fs::write(root.join("docs").join("deployment.md"), "# Deployment\n").unwrap();

        let errors = check_deployment_checklist(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "README.md: missing local check command block",
                "docs/release.md: missing release verification command block",
                "docs/deployment.md: missing deployment smoke command block",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_all_provider_deployment_examples() {
        let root = temp_root("deployment-docs-accepts");
        write_deployment_doc(&root, &provider_credentials_section());

        assert_eq!(check_deployment_docs(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_provider_credentials_section() {
        let root = temp_root("deployment-docs-missing-section");
        write_deployment_doc(&root, "## Runtime Notes\n");

        let errors = check_deployment_docs(&root).unwrap_err();

        assert_eq!(
            errors,
            ["docs/deployment.md: missing Provider Credentials section"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_provider_env_and_run_example() {
        let root = temp_root("deployment-docs-missing-example");
        write_deployment_doc(
            &root,
            &provider_credentials_section()
                .replace("-e GROQ_API_KEY", "-e OTHER_KEY")
                .replace("--provider openrouter", "--provider deterministic"),
        );

        let errors = check_deployment_docs(&root).unwrap_err();

        assert!(errors.contains(
            &"docs/deployment.md: missing container env example for GROQ_API_KEY".to_owned()
        ));
        assert!(
            errors.contains(
                &"docs/deployment.md: missing container run example for provider `openrouter`"
                    .to_owned()
            )
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_current_public_status_docs() {
        let root = temp_root("public-status-accepts");
        write_status_docs(&root, None, None);

        assert_eq!(check_public_status_docs(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_public_status_document() {
        let root = temp_root("public-status-missing-doc");
        write_status_docs(&root, None, None);
        fs::remove_file(root.join("SUPPORT.md")).unwrap();

        let errors = check_public_status_docs(&root).unwrap_err();

        assert!(errors.contains(&"SUPPORT.md: missing public status document".to_owned()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_stale_public_status_wording() {
        let root = temp_root("public-status-stale-wording");
        write_status_docs(
            &root,
            Some(
                "# README\n\nVogon Runtime is pre-release. The current codebase is a small Rust workspace.\n",
            ),
            None,
        );

        let errors = check_public_status_docs(&root).unwrap_err();

        assert!(errors.contains(
            &"README.md: stale status phrase \"Vogon Runtime is pre-release\"".to_owned()
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_first_release_wording() {
        let root = temp_root("public-status-missing-wording");
        write_status_docs(
            &root,
            None,
            Some("# Security\n\nSecurity fixes are handled.\n"),
        );

        let errors = check_public_status_docs(&root).unwrap_err();

        assert!(errors.contains(&"SECURITY.md: missing \"`v0.1.1` is the latest public release of Vogon Runtime; `v0.1.0` was the first public release.\"".to_owned()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_documented_package_verification_rationale() {
        let root = temp_root("package-verification-accepts");
        write_package_verification_docs(&root, PACKAGE_VERIFICATION_COMMAND, None);

        assert_eq!(check_package_verification_docs(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_package_command() {
        let root = temp_root("package-verification-missing-command");
        write_package_verification_docs(&root, "cargo package --workspace --offline", None);

        let errors = check_package_verification_docs(&root).unwrap_err();

        assert!(errors.contains(&"README.md: missing offline package command".to_owned()));
        assert!(errors.contains(&"docs/release.md: missing offline package command".to_owned()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_package_verification_rationale() {
        let root = temp_root("package-verification-missing-rationale");
        write_package_verification_docs(
            &root,
            PACKAGE_VERIFICATION_COMMAND,
            Some("Run this after the other checks."),
        );

        let errors = check_package_verification_docs(&root).unwrap_err();

        assert!(errors.contains(&"README.md: missing package verification rationale".to_owned()));
        assert!(
            errors.contains(&"docs/release.md: missing package verification rationale".to_owned())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_hardened_container_files() {
        let root = temp_root("container-policy-accepts");
        write_container_files(&root, None);

        assert_eq!(check_container_policy(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_container_files() {
        let root = temp_root("container-policy-missing-files");

        let errors = check_container_policy(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "Dockerfile: missing container build file",
                ".dockerignore: missing container build context ignore file",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_latest_and_untagged_base_images() {
        let root = temp_root("container-policy-base-images");
        write_container_files(&root, None);
        let dockerfile = root.join("Dockerfile");
        fs::write(
            &dockerfile,
            fs::read_to_string(&dockerfile)
                .unwrap()
                .replace("rust:1.96.1-bookworm", "rust")
                .replace("debian:bookworm-slim", "debian:latest"),
        )
        .unwrap();

        let errors = check_container_policy(&root).unwrap_err();

        assert!(
            errors.contains(
                &"Dockerfile:3: base image `rust` must include a tag or digest".to_owned()
            )
        );
        assert!(
            errors.contains(
                &"Dockerfile:15: base image `debian:latest` must not use latest".to_owned()
            )
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_runtime_hardening() {
        let root = temp_root("container-policy-runtime-hardening");
        write_container_files(&root, None);
        let dockerfile = root.join("Dockerfile");
        fs::write(
            &dockerfile,
            fs::read_to_string(&dockerfile)
                .unwrap()
                .replace("USER vogon", ""),
        )
        .unwrap();

        let errors = check_container_policy(&root).unwrap_err();

        assert_eq!(errors, ["Dockerfile: missing non-root user activation"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_oci_metadata_label() {
        let root = temp_root("container-policy-oci-label");
        write_container_files(&root, None);
        let dockerfile = root.join("Dockerfile");
        fs::write(
            &dockerfile,
            fs::read_to_string(&dockerfile)
                .unwrap()
                .replace("    org.opencontainers.image.licenses=\"MIT\" \\\n", ""),
        )
        .unwrap();

        let errors = check_container_policy(&root).unwrap_err();

        assert_eq!(errors, ["Dockerfile: missing OCI license label"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_build_context_ignores() {
        let root = temp_root("container-policy-dockerignore");
        write_container_files(&root, Some("/.git\n"));

        let errors = check_container_policy(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                ".dockerignore: missing !.env.example",
                ".dockerignore: missing *.cache.json",
                ".dockerignore: missing *.py[cod]",
                ".dockerignore: missing .env",
                ".dockerignore: missing .env.*",
                ".dockerignore: missing /.github",
                ".dockerignore: missing /target",
                ".dockerignore: missing __pycache__/",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_expected_dependabot_config() {
        let root = temp_root("dependabot-accepts");
        write_dependabot_config(&root, &dependabot_config_text());

        assert_eq!(check_dependabot_config(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_dependabot_config() {
        let root = temp_root("dependabot-missing-config");

        let errors = check_dependabot_config(&root).unwrap_err();

        assert_eq!(
            errors,
            [".github/dependabot.yml: missing Dependabot configuration"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_docker_updates() {
        let root = temp_root("dependabot-missing-docker");
        write_dependabot_config(
            &root,
            &dependabot_config_text().replace(&docker_update_text(), ""),
        );

        let errors = check_dependabot_config(&root).unwrap_err();

        assert_eq!(errors, [".github/dependabot.yml: missing docker updates"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_weakened_dependabot_update_schedule() {
        let root = temp_root("dependabot-weakened-schedule");
        write_dependabot_config(
            &root,
            &dependabot_config_text().replacen("interval: weekly", "interval: monthly", 1),
        );

        let errors = check_dependabot_config(&root).unwrap_err();

        assert_eq!(
            errors,
            [".github/dependabot.yml: cargo `interval` must be 'weekly'"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_wrong_dependabot_commit_prefix() {
        let root = temp_root("dependabot-wrong-prefix");
        write_dependabot_config(
            &root,
            &dependabot_config_text().replace("prefix: ci", "prefix: deps"),
        );

        let errors = check_dependabot_config(&root).unwrap_err();

        assert_eq!(
            errors,
            [".github/dependabot.yml: github-actions `commit-message.prefix` must be 'ci'",]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_dependabot_update_group() {
        let root = temp_root("dependabot-missing-group");
        write_dependabot_config(
            &root,
            &dependabot_config_text().replace(&cargo_group_text(), ""),
        );

        let errors = check_dependabot_config(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                ".github/dependabot.yml: cargo `groups.cargo-minor-patch.patterns` must be '*'",
                ".github/dependabot.yml: cargo `groups.cargo-minor-patch.update-types` must be 'minor,patch'",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn extracts_nested_badge_link_without_image_target() {
        let links = markdown_link_targets(
            "[![CI](https://example.com/badge.svg)](https://github.com/kaleab-kali/vogon-runtime/actions/workflows/ci.yml)",
        );

        assert_eq!(
            links,
            ["https://github.com/kaleab-kali/vogon-runtime/actions/workflows/ci.yml"]
        );
    }

    #[test]
    fn accepts_relative_absolute_and_repo_blob_links() {
        let root = temp_root("docs-links-accepts");
        let docs = root.join("docs");
        fs::create_dir(&docs).unwrap();
        fs::write(docs.join("guide.md"), "# Guide\n").unwrap();
        fs::write(
            root.join("README.md"),
            [
                "[Guide](docs/guide.md)",
                "[Root guide](/docs/guide.md)",
                "[GitHub guide](https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/guide.md)",
                "[Anchor](#local-heading)",
                "[External](https://example.com/docs)",
            ]
            .join("\n"),
        )
        .unwrap();

        assert_eq!(check_docs_links(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_repository_link_targets() {
        let root = temp_root("docs-links-missing");
        fs::write(root.join("README.md"), "[Missing](docs/missing.md)\n").unwrap();

        let errors = check_docs_links(&root).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("README.md:1"));
        assert!(errors[0].contains("docs/missing.md"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_valid_issue_templates() {
        let root = temp_root("issue-template-accepts");
        write_issue_templates(&root);

        assert_eq!(check_issue_templates(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_issue_template_config_guards() {
        let root = temp_root("issue-template-config");
        write_issue_templates(&root);
        fs::write(
            root.join(".github")
                .join("ISSUE_TEMPLATE")
                .join("config.yml"),
            "blank_issues_enabled: true\n",
        )
        .unwrap();

        let errors = check_issue_templates(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                ".github/ISSUE_TEMPLATE/config.yml: blank issues must stay disabled",
                ".github/ISSUE_TEMPLATE/config.yml: missing private vulnerability reporting link",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_bug_reproduction_field_and_secret_check() {
        let root = temp_root("issue-template-bug-fields");
        write_issue_templates(&root);
        fs::write(
            root.join(".github")
                .join("ISSUE_TEMPLATE")
                .join("bug_report.yml"),
            valid_issue_form(
                "Bug report",
                "title: \"Bug: \"",
                "- bug",
                &[
                    "version",
                    "component",
                    "expected",
                    "actual",
                    "environment",
                    "checks",
                ],
                false,
                None,
                "vogon 0.1.1",
            ),
        )
        .unwrap();

        let errors = check_issue_templates(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                ".github/ISSUE_TEMPLATE/bug_report.yml: missing required field `reproduce`",
                ".github/ISSUE_TEMPLATE/bug_report.yml: missing required before-submit check `removed secrets`",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_feature_dropdown_option() {
        let root = temp_root("issue-template-feature-options");
        write_issue_templates(&root);
        fs::write(
            root.join(".github")
                .join("ISSUE_TEMPLATE")
                .join("feature_request.yml"),
            valid_issue_form(
                "Feature request",
                "title: \"Feature: \"",
                "- enhancement",
                &["problem", "proposal", "area", "checks"],
                true,
                Some(&["CLI", "Runtime"]),
                "vogon 0.1.1",
            ),
        )
        .unwrap();

        let errors = check_issue_templates(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                ".github/ISSUE_TEMPLATE/feature_request.yml: dropdown options missing Documentation, Other, Provider adapter, Release artifact, Replay verification",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_stale_bug_version_placeholder() {
        let root = temp_root("issue-template-stale-version");
        write_issue_templates(&root);
        fs::write(
            root.join(".github")
                .join("ISSUE_TEMPLATE")
                .join("bug_report.yml"),
            valid_issue_form(
                "Bug report",
                "title: \"Bug: \"",
                "- bug",
                &[
                    "version",
                    "component",
                    "expected",
                    "actual",
                    "reproduce",
                    "environment",
                    "checks",
                ],
                true,
                None,
                "vogon 0.1.0",
            ),
        )
        .unwrap();

        let errors = check_issue_templates(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                ".github/ISSUE_TEMPLATE/bug_report.yml: version placeholder must match the latest public release",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_secret_like_values() {
        let root = temp_root("secrets-token-patterns");
        fs::write(
            root.join("README.md"),
            format!(
                "token=sk-{}\ngroq=gsk_{}\n",
                "abcdefghijklmnopqrstuvwxyz",
                "A".repeat(30)
            ),
        )
        .unwrap();

        let findings = check_secrets(&root).unwrap_err();

        assert_eq!(
            findings,
            [
                "README.md:1: possible OpenAI API key",
                "README.md:2: possible Groq API key",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_short_test_placeholders() {
        let root = temp_root("secrets-placeholders");
        fs::write(
            root.join("docs.md"),
            [
                "api_key=sk-test-123".to_owned(),
                "token=secret-key".to_owned(),
                format!("{}=", "OPENROUTER_API_KEY"),
            ]
            .join("\n"),
        )
        .unwrap();

        assert_eq!(check_secrets(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_provider_env_assignments_with_real_values() {
        let root = temp_root("secrets-provider-values");
        fs::write(
            root.join(".env"),
            [
                format!("{}=real-provider-secret", "GEMINI_API_KEY"),
                format!("{}: another-provider-secret", "OPENROUTER_API_KEY"),
            ]
            .join("\n"),
        )
        .unwrap();

        let findings = check_secrets(&root).unwrap_err();

        assert_eq!(
            findings,
            [
                ".env:1: possible committed GEMINI_API_KEY value",
                ".env:2: possible committed OPENROUTER_API_KEY value",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_provider_env_placeholders_and_secret_refs() {
        let root = temp_root("secrets-provider-placeholders");
        fs::write(
            root.join("workflow.yml"),
            [
                format!("{}=...", "GEMINI_API_KEY"),
                format!("{}=", "GROQ_API_KEY"),
                format!("{}: ${{{{ secrets.HF_TOKEN }}}}", "HF_TOKEN"),
                format!("{}=\"$OPENROUTER_API_KEY\"", "OPENROUTER_API_KEY"),
                format!("{}=<api-key>", "OPENAI_COMPATIBLE_API_KEY"),
            ]
            .join("\n"),
        )
        .unwrap();

        assert_eq!(check_secrets(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_committed_cache_artifacts() {
        let root = temp_root("secrets-cache-artifact");
        fs::write(root.join("target-output.cache.json"), "{\"outputs\": {}}").unwrap();

        let findings = check_secrets(&root).unwrap_err();

        assert_eq!(
            findings,
            ["target-output.cache.json: possible committed sensitive cache artifact"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_non_cache_json_files() {
        let root = temp_root("secrets-non-cache-json");
        fs::write(root.join("fixture.replay.json"), "{\"outputs\": {}}").unwrap();

        assert_eq!(check_secrets(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn skips_binary_and_large_files() {
        let root = temp_root("secrets-skips-binary-large");
        fs::write(root.join("image.bin"), b"\0sk-abcdefghijklmnopqrstuvwxyz").unwrap();
        fs::write(
            root.join("large.txt"),
            "x".repeat(MAX_SECRET_SCAN_TEXT_BYTES as usize + 1),
        )
        .unwrap();

        assert_eq!(check_secrets(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_valid_changelog() {
        let root = temp_root("changelog-accepts");
        write_changelog(
            &root,
            r#"# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once the first release is tagged.

## [Unreleased]

### Added

- Initial feature.
"#,
        );

        assert_eq!(check_changelog(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_empty_unreleased_after_dated_release() {
        let root = temp_root("changelog-empty-unreleased");
        write_changelog(
            &root,
            r#"# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once the first release is tagged.

## [Unreleased]

## [0.1.0] - 2026-07-08

### Added

- Initial feature.
"#,
        );

        assert_eq!(check_changelog(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_changelog_structure() {
        let root = temp_root("changelog-missing-structure");
        write_changelog(&root, "# Changes\n\n## Next\n");

        let errors = check_changelog(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "CHANGELOG.md: first line must be `# Changelog`",
                "CHANGELOG.md: missing Keep a Changelog 1.1.0 reference",
                "CHANGELOG.md: missing semantic versioning note",
                "CHANGELOG.md: missing `## [Unreleased]` section",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_empty_and_unsupported_unreleased_subsections() {
        let root = temp_root("changelog-empty-subsections");
        write_changelog(
            &root,
            r#"# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once the first release is tagged.

## [Unreleased]

### Internal

### Fixed

## [0.1.0] - 2026-07-08
"#,
        );

        let errors = check_changelog(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "CHANGELOG.md: unsupported Unreleased subsection `Internal`",
                "CHANGELOG.md: Unreleased `Internal` subsection has no entries",
                "CHANGELOG.md: Unreleased `Fixed` subsection has no entries",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_release_heading_without_date() {
        let root = temp_root("changelog-release-heading");
        write_changelog(
            &root,
            r#"# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once the first release is tagged.

## [Unreleased]

## [0.1.0]

### Added

- Initial feature.
"#,
        );

        let errors = check_changelog(&root).unwrap_err();

        assert_eq!(
            errors,
            ["CHANGELOG.md: release heading `## [0.1.0]` must include a version and date",]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_valid_workspace_manifests() {
        let root = temp_root("cargo-accepts");
        write_workspace(&root, WorkspaceOptions::default());

        assert_eq!(check_cargo_manifests(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_workspace_package_metadata() {
        let root = temp_root("cargo-workspace-metadata");
        write_workspace(
            &root,
            WorkspaceOptions {
                workspace_package: Some("edition = \"2024\"\n"),
                ..WorkspaceOptions::default()
            },
        );

        let errors = check_cargo_manifests(&root).unwrap_err();

        assert!(errors.contains(&"Cargo.toml: workspace package `license` must be 'MIT'".into()));
        assert!(errors.contains(
            &"Cargo.toml: workspace package `repository` must be 'https://github.com/kaleab-kali/vogon-runtime'"
                .into()
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_crate_metadata() {
        let root = temp_root("cargo-crate-metadata");
        write_workspace(&root, WorkspaceOptions::default());
        let manifest = root.join("crates/vogon-core/Cargo.toml");
        fs::write(
            &manifest,
            fs::read_to_string(&manifest).unwrap().replace(
                "description = \"Core deterministic workflow runtime for Vogon Runtime.\"\n",
                "",
            ),
        )
        .unwrap();

        let errors = check_cargo_manifests(&root).unwrap_err();

        assert!(
            errors.contains(&"crates/vogon-core/Cargo.toml: package missing `description`".into())
        );
        assert!(errors.contains(
            &"crates/vogon-core/Cargo.toml: package `description` must not be empty".into()
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_internal_dependency_version_mismatch() {
        let root = temp_root("cargo-dependency-version");
        write_workspace(
            &root,
            WorkspaceOptions {
                adapters_dependency_version: "9.9.9",
                ..WorkspaceOptions::default()
            },
        );

        let errors = check_cargo_manifests(&root).unwrap_err();

        assert!(errors.contains(
            &"Cargo.toml: workspace dependency `vogon-adapters` version must match crate version 0.1.0"
                .into()
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_weakened_release_profile() {
        let root = temp_root("cargo-release-profile");
        write_workspace(
            &root,
            WorkspaceOptions {
                release_profile: Some(
                    &release_profile_text().replace("lto = \"thin\"", "lto = false"),
                ),
                ..WorkspaceOptions::default()
            },
        );

        let errors = check_cargo_manifests(&root).unwrap_err();

        assert!(errors.contains(&"Cargo.toml: release profile `lto` must be 'thin'".into()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_workspace_unsafe_lint() {
        let root = temp_root("cargo-workspace-lint");
        write_workspace(
            &root,
            WorkspaceOptions {
                workspace_lints: Some(""),
                ..WorkspaceOptions::default()
            },
        );

        let errors = check_cargo_manifests(&root).unwrap_err();

        assert!(errors.contains(&"Cargo.toml: missing [workspace.lints.rust]".into()));
        assert!(
            errors
                .contains(&"Cargo.toml: workspace rust lint `unsafe_code` must be 'forbid'".into())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_crate_that_does_not_use_workspace_lints() {
        let root = temp_root("cargo-crate-lint");
        write_workspace(
            &root,
            WorkspaceOptions {
                crate_lints: Some(""),
                ..WorkspaceOptions::default()
            },
        );

        let errors = check_cargo_manifests(&root).unwrap_err();

        assert!(errors.contains(
            &"crates/vogon-core/Cargo.toml: crate lints must use workspace policy".into()
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_blank_expected_provider_variables() {
        let root = temp_root("accepts");
        let contents = EXPECTED_ENV_VARS
            .iter()
            .map(|name| format!("{name}="))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(root.join(".env.example"), contents).unwrap();

        assert_eq!(check_env_example(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_unexpected_and_populated_values() {
        let root = temp_root("reports");
        let populated_gemini_key = format!("{}=populated", EXPECTED_ENV_VARS[0]);
        fs::write(
            root.join(".env.example"),
            [
                populated_gemini_key.as_str(),
                "GROQ_API_KEY=",
                "HF_TOKEN=",
                "OPENROUTER_API_KEY=",
                "EXTRA_KEY=",
            ]
            .join("\n"),
        )
        .unwrap();

        let errors = check_env_example(&root).unwrap_err();
        assert_eq!(errors.len(), 3);
        assert!(errors[0].contains("OPENAI_COMPATIBLE_API_KEY"));
        assert!(errors[1].contains("EXTRA_KEY"));
        assert!(errors[2].contains("GEMINI_API_KEY"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_malformed_assignment_lines() {
        let root = temp_root("malformed");
        fs::write(root.join(".env.example"), "GEMINI_API_KEY\n").unwrap();

        let errors = check_env_example(&root).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("expected KEY=VALUE assignment"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_expected_linux_archive_contents() {
        let root = temp_root("archive-linux");
        write_archive_entry(&root, "vogon", "binary");
        write_archive_entry(&root, "README.md", "readme");
        write_archive_entry(&root, "LICENSE", "license");

        assert_eq!(
            check_archive_contents(&root, "vogon", &default_archive_required_files()),
            Ok(())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_expected_windows_archive_contents() {
        let root = temp_root("archive-windows");
        write_archive_entry(&root, "vogon.exe", "binary");
        write_archive_entry(&root, "README.md", "readme");
        write_archive_entry(&root, "LICENSE", "license");

        assert_eq!(
            check_archive_contents(&root, "vogon.exe", &default_archive_required_files()),
            Ok(())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_custom_required_archive_files() {
        let root = temp_root("archive-custom");
        write_archive_entry(&root, "vogon", "binary");
        write_archive_entry(&root, "NOTICE", "notice");

        assert_eq!(
            check_archive_contents(&root, "vogon", &["NOTICE".to_owned()]),
            Ok(())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_archive_directory() {
        let root = temp_root("archive-missing-dir");

        let errors = check_archive_contents(
            &root.join("missing"),
            "vogon",
            &default_archive_required_files(),
        )
        .unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("Archive directory is missing or is not a directory:"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_archive_binary_and_required_files() {
        let root = temp_root("archive-missing-files");

        let errors =
            check_archive_contents(&root, "vogon", &default_archive_required_files()).unwrap_err();

        assert_eq!(
            errors,
            [
                "Packaged archive binary is missing: vogon",
                "Packaged archive required file is missing: README.md",
                "Packaged archive required file is missing: LICENSE",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_archive_directories_where_files_are_expected() {
        let root = temp_root("archive-directories");
        fs::create_dir(root.join("vogon")).unwrap();
        fs::create_dir(root.join("README.md")).unwrap();
        fs::create_dir(root.join("LICENSE")).unwrap();

        let errors =
            check_archive_contents(&root, "vogon", &default_archive_required_files()).unwrap_err();

        assert_eq!(
            errors,
            [
                "Packaged archive binary is not a regular file: vogon",
                "Packaged archive required file is not a regular file: README.md",
                "Packaged archive required file is not a regular file: LICENSE",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_unexpected_archive_entries() {
        let root = temp_root("archive-unexpected");
        write_archive_entry(&root, "vogon", "binary");
        write_archive_entry(&root, "README.md", "readme");
        write_archive_entry(&root, "LICENSE", "license");
        write_archive_entry(&root, ".env", "SECRET=value");
        fs::create_dir(root.join("docs")).unwrap();

        let errors =
            check_archive_contents(&root, "vogon", &default_archive_required_files()).unwrap_err();

        assert_eq!(
            errors,
            [
                "Packaged archive contains unexpected entry: .env",
                "Packaged archive contains unexpected entry: docs",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_expected_schema_files() {
        let root = temp_root("schema-accepts");
        write_schema_files(&root, None, None);
        write_schema_fixture_files(&root, None, None);

        assert_eq!(check_schema_files(&root), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_schema_files() {
        let root = temp_root("schema-missing");

        let errors = check_schema_files(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "schemas/workflow.schema.json: missing schema file",
                "schemas/replay.schema.json: missing schema file",
                "fixtures/workflows: missing workflow fixtures",
                "fixtures/replays: missing replay fixtures",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_invalid_schema_json() {
        let root = temp_root("schema-invalid-json");
        write_schema_files(&root, Some("{"), None);
        write_schema_fixture_files(&root, None, None);

        let errors = check_schema_files(&root).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("schemas/workflow.schema.json: invalid JSON:"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_weakened_schema_root_strictness() {
        let root = temp_root("schema-weakened-root");
        write_schema_files(
            &root,
            Some(&workflow_schema_text().replacen(
                "\"additionalProperties\": false",
                "\"additionalProperties\": true",
                1,
            )),
            None,
        );
        write_schema_fixture_files(&root, None, None);

        let errors = check_schema_files(&root).unwrap_err();

        assert_eq!(
            errors,
            ["schemas/workflow.schema.json: root additionalProperties must be false"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_workflow_fixture_outside_schema_shape() {
        let root = temp_root("schema-workflow-fixture");
        write_schema_files(&root, None, None);
        write_schema_fixture_files(&root, Some("name = \"support triage\"\n"), None);

        let errors = check_schema_files(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "fixtures/workflows/support-triage.toml: workflow name must use ASCII letters, digits, underscores, and hyphens",
                "fixtures/workflows/support-triage.toml: workflow steps must be a non-empty list",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_replay_fixture_outside_schema_shape() {
        let root = temp_root("schema-replay-fixture");
        write_schema_files(&root, None, None);
        write_schema_fixture_files(
            &root,
            None,
            Some(&replay_fixture_text().replacen(
                "\"schema_version\": 1",
                "\"schema_version\": 0",
                1,
            )),
        );

        let errors = check_schema_files(&root).unwrap_err();

        assert_eq!(
            errors,
            ["fixtures/replays/support-triage.replay.json: replay schema_version must be 1"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_expected_benchmark_metrics() {
        let output = [
            "Compiling benchmark harness",
            "iterations: 100",
            "elapsed_ms: 2.5",
            "iterations_per_second: 40",
        ]
        .join("\n");

        assert_eq!(check_benchmark_output(&output, 100), Ok(()));
    }

    #[test]
    fn reports_missing_benchmark_metrics() {
        let errors = check_benchmark_output("iterations: 100\n", 100).unwrap_err();

        assert_eq!(
            errors,
            [
                "missing benchmark metric: elapsed_ms",
                "missing benchmark metric: iterations_per_second",
            ]
        );
    }

    #[test]
    fn reports_benchmark_iteration_mismatch() {
        let output = [
            "iterations: 10",
            "elapsed_ms: 1",
            "iterations_per_second: 10",
        ]
        .join("\n");

        let errors = check_benchmark_output(&output, 100).unwrap_err();

        assert_eq!(
            errors,
            ["benchmark iterations mismatch: expected 100, got 10"]
        );
    }

    #[test]
    fn rejects_invalid_and_non_positive_benchmark_metrics() {
        let output = [
            "iterations: no",
            "elapsed_ms: 0",
            "iterations_per_second: nan",
        ]
        .join("\n");

        let errors = check_benchmark_output(&output, 100).unwrap_err();

        assert_eq!(
            errors,
            [
                "benchmark iterations must be an integer",
                "benchmark elapsed_ms must be greater than zero",
                "benchmark iterations_per_second must be finite",
            ]
        );
    }

    #[test]
    fn accepts_expected_cargo_metadata_json() {
        let output = valid_cargo_metadata_json();
        let expected_packages = vec!["vogon-core".to_owned(), "vogon-cli".to_owned()];

        assert_eq!(
            check_cargo_metadata_json(&output, &expected_packages),
            Ok(())
        );
    }

    #[test]
    fn accepts_cargo_metadata_json_file_path() {
        let root = temp_root("cargo-metadata-json-file");
        let metadata_file = root.join("metadata.json");
        fs::write(&metadata_file, valid_cargo_metadata_json()).unwrap();

        assert_eq!(
            check_cargo_metadata_json_file(&metadata_file, &["vogon-core".to_owned()]),
            Ok(())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_invalid_cargo_metadata_json() {
        let errors = check_cargo_metadata_json("{", &[]).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("Cargo metadata JSON is invalid:"));
    }

    #[test]
    fn reports_missing_cargo_metadata_package_fields() {
        let output = serde_json::json!({
            "packages": [{"id": "", "name": "vogon-core"}],
            "workspace_members": [
                "path+file:///repo#vogon-core@0.1.0",
                "path+file:///repo#vogon-cli@0.1.0"
            ],
            "resolve": {
                "nodes": [
                    {
                        "id": "path+file:///repo#vogon-core@0.1.0",
                        "deps": []
                    },
                    {
                        "id": "path+file:///repo#vogon-cli@0.1.0",
                        "deps": []
                    }
                ]
            }
        })
        .to_string();

        let errors = check_cargo_metadata_json(&output, &[]).unwrap_err();

        assert_eq!(
            errors,
            [
                "Cargo metadata package 1 id must be a non-empty string",
                "Cargo metadata package 1 version must be a non-empty string",
                "Cargo metadata package 1 manifest_path must be a non-empty string",
            ]
        );
    }

    #[test]
    fn reports_missing_expected_cargo_metadata_workspace_package() {
        let expected_packages = vec!["vogon-adapters".to_owned()];
        let errors = check_cargo_metadata_json(&valid_cargo_metadata_json(), &expected_packages)
            .unwrap_err();

        assert_eq!(
            errors,
            [
                "Cargo metadata workspace package missing: expected vogon-adapters, got [\"vogon-cli\",\"vogon-core\"]",
            ]
        );
    }

    #[test]
    fn reports_missing_cargo_metadata_resolve_nodes() {
        let output = serde_json::json!({
            "packages": [
                {
                    "id": "path+file:///repo#vogon-core@0.1.0",
                    "name": "vogon-core",
                    "version": "0.1.0",
                    "manifest_path": "/repo/crates/vogon-core/Cargo.toml"
                }
            ],
            "workspace_members": ["path+file:///repo#vogon-core@0.1.0"],
            "resolve": {
                "nodes": []
            }
        })
        .to_string();

        let errors = check_cargo_metadata_json(&output, &[]).unwrap_err();

        assert_eq!(
            errors,
            ["Cargo metadata JSON resolve.nodes must be a non-empty array"]
        );
    }

    #[test]
    fn accepts_expected_providers_json() {
        assert_eq!(check_providers_json(&provider_json_output()), Ok(()));
    }

    #[test]
    fn reports_invalid_providers_json() {
        let errors = check_providers_json("{").unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("providers JSON is invalid:"));
    }

    #[test]
    fn reports_missing_provider_json_entry() {
        let mut data: JsonValue = serde_json::from_str(&provider_json_output()).unwrap();
        let providers = data
            .get_mut("providers")
            .and_then(JsonValue::as_array_mut)
            .unwrap();
        providers.retain(|provider| {
            provider.get("name").and_then(JsonValue::as_str) != Some("openrouter")
        });

        let errors = check_providers_json(&data.to_string()).unwrap_err();

        assert_eq!(errors, ["providers must include openrouter"]);
    }

    #[test]
    fn reports_wrong_provider_json_default_count() {
        let mut data: JsonValue = serde_json::from_str(&provider_json_output()).unwrap();
        for provider in data
            .get_mut("providers")
            .and_then(JsonValue::as_array_mut)
            .unwrap()
        {
            provider["default"] = JsonValue::Bool(false);
        }

        let errors = check_providers_json(&data.to_string()).unwrap_err();

        assert_eq!(
            errors,
            [
                "provider deterministic default mismatch: expected true, got false",
                "exactly one provider must be default, found 0",
            ]
        );
    }

    #[test]
    fn reports_provider_json_metadata_mismatch() {
        let mut data: JsonValue = serde_json::from_str(&provider_json_output()).unwrap();
        let gemini = data
            .get_mut("providers")
            .and_then(JsonValue::as_array_mut)
            .unwrap()
            .iter_mut()
            .find(|provider| provider.get("name").and_then(JsonValue::as_str) == Some("gemini"))
            .unwrap();
        gemini["default_model"] = JsonValue::String("gemini-old".to_owned());

        let errors = check_providers_json(&data.to_string()).unwrap_err();

        assert_eq!(
            errors,
            [
                "provider gemini default_model mismatch: expected \"gemini-3.1-flash-lite\", got \"gemini-old\"",
            ]
        );
    }

    #[test]
    fn reports_non_boolean_provider_json_credential_status() {
        let mut data: JsonValue = serde_json::from_str(&provider_json_output()).unwrap();
        let groq = data
            .get_mut("providers")
            .and_then(JsonValue::as_array_mut)
            .unwrap()
            .iter_mut()
            .find(|provider| provider.get("name").and_then(JsonValue::as_str) == Some("groq"))
            .unwrap();
        groq["credential_configured"] = JsonValue::String("secret-groq-key".to_owned());

        let errors = check_providers_json(&data.to_string()).unwrap_err();

        assert_eq!(
            errors,
            [
                "provider groq credential_configured must be boolean or null, got \"secret-groq-key\"",
            ]
        );
    }

    #[test]
    fn accepts_expected_workflow_json() {
        let output = serde_json::json!({
            "workflow_name": "support-triage",
            "step_count": 2
        })
        .to_string();

        assert_eq!(
            check_workflow_json(&output, Some("support-triage"), Some(2)),
            Ok(())
        );
    }

    #[test]
    fn reports_invalid_workflow_json() {
        let errors = check_workflow_json("{", None, None).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("workflow check JSON is invalid:"));
    }

    #[test]
    fn reports_non_object_workflow_json_root() {
        let errors = check_workflow_json("[]", None, None).unwrap_err();

        assert_eq!(errors, ["workflow check JSON root must be an object"]);
    }

    #[test]
    fn reports_malformed_workflow_json_fields() {
        let output = serde_json::json!({
            "workflow_name": "",
            "step_count": 0
        })
        .to_string();

        let errors = check_workflow_json(&output, None, None).unwrap_err();

        assert_eq!(
            errors,
            [
                "workflow check JSON workflow_name must be a non-empty string",
                "workflow check JSON step_count must be a positive integer",
            ]
        );
    }

    #[test]
    fn reports_expected_workflow_json_value_mismatches() {
        let output = serde_json::json!({
            "workflow_name": "writing-pipeline",
            "step_count": 3
        })
        .to_string();

        let errors = check_workflow_json(&output, Some("support-triage"), Some(2)).unwrap_err();

        assert_eq!(
            errors,
            [
                "workflow check JSON workflow_name mismatch: expected support-triage, got \"writing-pipeline\"",
                "workflow check JSON step_count mismatch: expected 2, got 3",
            ]
        );
    }

    #[test]
    fn accepts_expected_doctor_json() {
        assert_eq!(check_doctor_json(&doctor_json_output()), Ok(()));
    }

    #[test]
    fn reports_invalid_doctor_json() {
        let errors = check_doctor_json("{").unwrap_err();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("doctor JSON is invalid:"));
    }

    #[test]
    fn reports_missing_doctor_runtime_check() {
        let mut data: JsonValue = serde_json::from_str(&doctor_json_output()).unwrap();
        data["checks"] = JsonValue::Array(Vec::new());

        let errors = check_doctor_json(&data.to_string()).unwrap_err();

        assert_eq!(
            errors,
            ["doctor checks must include ok deterministic_runtime"]
        );
    }

    #[test]
    fn reports_doctor_provider_usage_url_mismatch() {
        let mut data: JsonValue = serde_json::from_str(&doctor_json_output()).unwrap();
        let providers = data
            .get_mut("providers")
            .and_then(JsonValue::as_array_mut)
            .unwrap();
        let gemini = providers
            .iter_mut()
            .find(|provider| provider.get("name").and_then(JsonValue::as_str) == Some("gemini"))
            .unwrap();
        gemini["usage_url"] = JsonValue::String("https://example.com".to_owned());
        let openai_compatible = providers
            .iter_mut()
            .find(|provider| {
                provider.get("name").and_then(JsonValue::as_str) == Some("openai-compatible")
            })
            .unwrap();
        openai_compatible["usage_url"] = JsonValue::String("https://example.com/usage".to_owned());

        let errors = check_doctor_json(&data.to_string()).unwrap_err();

        assert_eq!(
            errors,
            [
                "doctor provider gemini usage_url mismatch: expected https://ai.google.dev/gemini-api/docs/pricing, got \"https://example.com\"",
                "doctor provider openai-compatible usage_url must be null, got \"https://example.com/usage\"",
            ]
        );
    }

    fn temp_root(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            env::temp_dir().join(format!("vogon-xtask-{name}-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).unwrap();
        path
    }

    fn default_archive_required_files() -> Vec<String> {
        DEFAULT_ARCHIVE_REQUIRED_FILES
            .iter()
            .map(|file| (*file).to_owned())
            .collect()
    }

    fn write_archive_entry(root: &Path, name: &str, contents: &str) {
        fs::write(root.join(name), contents).unwrap();
    }

    fn valid_cargo_metadata_json() -> String {
        serde_json::json!({
            "packages": [
                {
                    "id": "path+file:///repo#vogon-core@0.1.0",
                    "name": "vogon-core",
                    "version": "0.1.0",
                    "manifest_path": "/repo/crates/vogon-core/Cargo.toml"
                },
                {
                    "id": "path+file:///repo#vogon-cli@0.1.0",
                    "name": "vogon-cli",
                    "version": "0.1.0",
                    "manifest_path": "/repo/crates/vogon-cli/Cargo.toml"
                }
            ],
            "workspace_members": [
                "path+file:///repo#vogon-core@0.1.0",
                "path+file:///repo#vogon-cli@0.1.0"
            ],
            "resolve": {
                "nodes": [
                    {
                        "id": "path+file:///repo#vogon-core@0.1.0",
                        "deps": []
                    },
                    {
                        "id": "path+file:///repo#vogon-cli@0.1.0",
                        "deps": []
                    }
                ]
            }
        })
        .to_string()
    }

    fn provider_json_output() -> String {
        serde_json::json!({
            "providers": [
                {
                    "name": "deterministic",
                    "enabled": true,
                    "default": true,
                    "credential_env": null,
                    "credential_configured": null,
                    "default_base_url": null,
                    "default_model": null,
                    "documentation_url": "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#deterministic",
                    "usage_url": null
                },
                {
                    "name": "gemini",
                    "enabled": true,
                    "default": false,
                    "credential_env": "GEMINI_API_KEY",
                    "credential_configured": false,
                    "default_base_url": null,
                    "default_model": "gemini-3.1-flash-lite",
                    "documentation_url": "https://ai.google.dev/gemini-api/docs",
                    "usage_url": "https://ai.google.dev/gemini-api/docs/pricing"
                },
                {
                    "name": "groq",
                    "enabled": true,
                    "default": false,
                    "credential_env": "GROQ_API_KEY",
                    "credential_configured": true,
                    "default_base_url": "https://api.groq.com/openai/v1",
                    "default_model": "llama-3.1-8b-instant",
                    "documentation_url": "https://console.groq.com/docs/openai",
                    "usage_url": "https://console.groq.com/docs/rate-limits"
                },
                {
                    "name": "hugging-face",
                    "enabled": true,
                    "default": false,
                    "credential_env": "HF_TOKEN",
                    "credential_configured": true,
                    "default_base_url": "https://router.huggingface.co/v1",
                    "default_model": "openai/gpt-oss-120b:fastest",
                    "documentation_url": "https://huggingface.co/docs/inference-providers",
                    "usage_url": "https://huggingface.co/docs/inference-providers/pricing"
                },
                {
                    "name": "openrouter",
                    "enabled": true,
                    "default": false,
                    "credential_env": "OPENROUTER_API_KEY",
                    "credential_configured": false,
                    "default_base_url": "https://openrouter.ai/api/v1",
                    "default_model": "openrouter/free",
                    "documentation_url": "https://openrouter.ai/docs",
                    "usage_url": "https://openrouter.ai/pricing"
                },
                {
                    "name": "openai-compatible",
                    "enabled": true,
                    "default": false,
                    "credential_env": "OPENAI_COMPATIBLE_API_KEY",
                    "credential_configured": null,
                    "default_base_url": "https://router.huggingface.co/v1",
                    "default_model": "openai/gpt-oss-120b:fastest",
                    "documentation_url": "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#openai-compatible",
                    "usage_url": null
                }
            ]
        })
        .to_string()
    }

    fn doctor_json_output() -> String {
        serde_json::json!({
            "status": "ok",
            "version": "0.1.0",
            "checks": [
                {
                    "name": "deterministic_runtime",
                    "status": "ok",
                    "message": "deterministic runtime executed a one-step workflow"
                }
            ],
            "providers": [
                {
                    "name": "deterministic",
                    "usage_url": null
                },
                {
                    "name": "gemini",
                    "usage_url": "https://ai.google.dev/gemini-api/docs/pricing"
                },
                {
                    "name": "groq",
                    "usage_url": "https://console.groq.com/docs/rate-limits"
                },
                {
                    "name": "hugging-face",
                    "usage_url": "https://huggingface.co/docs/inference-providers/pricing"
                },
                {
                    "name": "openrouter",
                    "usage_url": "https://openrouter.ai/pricing"
                },
                {
                    "name": "openai-compatible",
                    "usage_url": null
                }
            ]
        })
        .to_string()
    }

    fn write_ci_workflow(root: &Path, text: &str) {
        let workflows = root.join(".github").join("workflows");
        fs::create_dir_all(&workflows).unwrap();
        fs::write(workflows.join("ci.yml"), text).unwrap();
    }

    fn ci_workflow_text() -> &'static str {
        r#"name: CI

on:
  pull_request:
  push:
    branches:
      - main

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_NET_RETRY: 10

jobs:
  rust:
    runs-on: ubuntu-24.04
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7
      - run: |
          cargo run -p vogon-xtask -- check-ci-workflow --root .
          cargo run -p vogon-xtask -- check-workflow-policies --root .
          cargo run -p vogon-xtask -- check-security-workflows --root .
          cargo run -p vogon-xtask -- check-container-policy --root .
          cargo run -p vogon-xtask -- check-secrets --root .
          python3 scripts/check_release_workflow.py --root .
          cargo run -p vogon-xtask -- check-changelog --root .
          cargo run -p vogon-xtask -- check-contributing-checklist --root .
          cargo run -p vogon-xtask -- check-deployment-checklist --root .
          cargo run -p vogon-xtask -- check-deployment-docs --root .
          cargo run -p vogon-xtask -- check-pr-template --root .
          cargo run -p vogon-xtask -- check-docs-links --root .
          cargo run -p vogon-xtask -- check-issue-templates --root .
          cargo run -p vogon-xtask -- check-release-checklist --root .
          cargo run -p vogon-xtask -- check-cargo-manifests --root .
          cargo run -p vogon-xtask -- check-env-example --root .
          cargo run -p vogon-xtask -- check-dependabot-config --root .
          cargo run -p vogon-xtask -- check-public-status-docs --root .
          cargo run -p vogon-xtask -- check-package-verification-docs --root .
          python3 scripts/check_live_workflows.py --root .
          cargo fmt --all -- --check
          cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
          cargo test --workspace --all-features --locked
          cargo check -p vogon-cli --no-default-features --locked
          cargo bench -p vogon-core --bench runtime --locked -- --iterations 100
          cargo run -p vogon-xtask -- check-benchmark-output --expected-iterations 100
          cargo build --release --workspace --all-features --locked
          ./target/release/vogon doctor --json
          cargo run -p vogon-xtask -- check-doctor-json
          ./target/release/vogon providers --json
          cargo run -p vogon-xtask -- check-providers-json
          ./target/release/vogon verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
          cargo run -p vogon-xtask -- check-workflow-json
          cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force
          cargo package -p vogon-core --allow-dirty --offline --locked
          cargo package --workspace --allow-dirty --no-verify --offline --locked
      - env:
          RUSTDOCFLAGS: -D warnings
        run: cargo doc --workspace --all-features --no-deps --locked

  msrv:
    runs-on: ubuntu-24.04
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@v7
      - run: cargo +1.85.0 test --workspace --all-features --locked

  container-smoke:
    runs-on: ubuntu-24.04
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7
      - run: |
          docker build --tag vogon-runtime:ci .
          docker run --rm --read-only vogon-runtime:ci --version

  windows-release-smoke:
    runs-on: windows-2025-vs2026
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7
      - run: |
          cargo build --release -p vogon-cli --locked
          .\target\release\vogon.exe verify fixtures\workflows\support-triage.toml fixtures\replays\support-triage.replay.json
"#
    }

    fn write_workflow_policy_file(root: &Path, filename: &str, lines: &[&str]) {
        let workflows = root.join(".github").join("workflows");
        fs::create_dir_all(&workflows).unwrap();
        fs::write(workflows.join(filename), lines.join("\n")).unwrap();
    }

    fn write_security_workflows(
        root: &Path,
        codeql: Option<String>,
        security_audit: Option<String>,
        dependency_review: Option<String>,
        dependency_review_config: Option<String>,
    ) {
        let workflows = root.join(".github").join("workflows");
        fs::create_dir_all(&workflows).unwrap();
        fs::write(
            workflows.join("codeql.yml"),
            codeql.unwrap_or_else(|| codeql_workflow_text().to_owned()),
        )
        .unwrap();
        fs::write(
            workflows.join("security-audit.yml"),
            security_audit.unwrap_or_else(|| security_audit_workflow_text().to_owned()),
        )
        .unwrap();
        fs::write(
            workflows.join("dependency-review.yml"),
            dependency_review.unwrap_or_else(|| dependency_review_workflow_text().to_owned()),
        )
        .unwrap();
        fs::write(
            root.join(".github").join("dependency-review-config.yml"),
            dependency_review_config.unwrap_or_else(|| dependency_review_config_text().to_owned()),
        )
        .unwrap();
    }

    fn codeql_workflow_text() -> &'static str {
        r#"name: CodeQL

on:
  pull_request:
  push:
    branches:
      - main
  schedule:
    - cron: "31 5 * * 2"
  workflow_dispatch:

permissions:
  contents: read
  security-events: write

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_NET_RETRY: 10

jobs:
  analyze:
    name: CodeQL Rust analysis
    runs-on: ubuntu-24.04
    timeout-minutes: 30

    steps:
      - name: Checkout
        uses: actions/checkout@v7

      - name: Initialize CodeQL
        uses: github/codeql-action/init@v4
        with:
          languages: rust
          build-mode: none
          queries: security-extended,security-and-quality

      - name: Perform CodeQL analysis
        uses: github/codeql-action/analyze@v4
"#
    }

    fn security_audit_workflow_text() -> &'static str {
        r#"name: Security Audit

on:
  pull_request:
    paths:
      - Cargo.lock
      - Cargo.toml
      - "crates/**/Cargo.toml"
      - .github/workflows/security-audit.yml
  push:
    branches:
      - main
    paths:
      - Cargo.lock
      - Cargo.toml
      - "crates/**/Cargo.toml"
      - .github/workflows/security-audit.yml
  schedule:
    - cron: "17 4 * * 1"
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  rustsec:
    name: RustSec advisory audit
    runs-on: ubuntu-24.04
    timeout-minutes: 10

    steps:
      - name: Checkout
        uses: actions/checkout@v7

      - name: Audit Cargo.lock
        uses: actions-rust-lang/audit@v1
        with:
          createIssues: false
"#
    }

    fn dependency_review_workflow_text() -> &'static str {
        r#"name: Dependency Review

on:
  pull_request:

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  dependency-review:
    name: Dependency review
    runs-on: ubuntu-24.04
    timeout-minutes: 10

    steps:
      - name: Checkout
        uses: actions/checkout@v7

      - name: Review dependency changes
        uses: actions/dependency-review-action@v5
        with:
          config-file: ./.github/dependency-review-config.yml
"#
    }

    fn dependency_review_config_text() -> &'static str {
        r#"fail-on-severity: high
license-check: true
vulnerability-check: true
allow-licenses:
  - Apache-2.0
  - BSD-3-Clause
  - CDLA-Permissive-2.0
  - ISC
  - MIT
  - Unicode-3.0
  - Unlicense
"#
    }

    fn write_schema_files(root: &Path, workflow_schema: Option<&str>, replay_schema: Option<&str>) {
        let schemas = root.join("schemas");
        fs::create_dir(&schemas).unwrap();
        fs::write(
            schemas.join("workflow.schema.json"),
            workflow_schema.unwrap_or(workflow_schema_text()),
        )
        .unwrap();
        fs::write(
            schemas.join("replay.schema.json"),
            replay_schema.unwrap_or(replay_schema_text()),
        )
        .unwrap();
    }

    fn write_schema_fixture_files(
        root: &Path,
        workflow_text: Option<&str>,
        replay_text: Option<&str>,
    ) {
        let workflows = root.join("fixtures").join("workflows");
        let replays = root.join("fixtures").join("replays");
        fs::create_dir_all(&workflows).unwrap();
        fs::create_dir_all(&replays).unwrap();
        fs::write(
            workflows.join("support-triage.toml"),
            workflow_text.unwrap_or(workflow_fixture_text()),
        )
        .unwrap();
        fs::write(
            replays.join("support-triage.replay.json"),
            replay_text.unwrap_or(replay_fixture_text()),
        )
        .unwrap();
    }

    fn workflow_schema_text() -> &'static str {
        r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Vogon Workflow",
  "type": "object",
  "additionalProperties": false,
  "required": ["name", "steps"],
  "properties": {
    "name": {},
    "steps": {}
  }
}
"#
    }

    fn replay_schema_text() -> &'static str {
        r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Vogon Replay",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema_version", "workflow_name", "runtime", "run_hash", "steps"],
  "properties": {
    "schema_version": {},
    "workflow_name": {},
    "runtime": {},
    "run_hash": {},
    "steps": {}
  }
}
"#
    }

    fn workflow_fixture_text() -> &'static str {
        r#"name = "support-triage"

[[steps]]
id = "classify"
prompt = "Classify this support request."
"#
    }

    fn replay_fixture_text() -> &'static str {
        r#"{
  "schema_version": 1,
  "workflow_name": "support-triage",
  "runtime": {
    "provider": "deterministic",
    "adapter": "deterministic-echo",
    "adapter_version": "0.1.0",
    "model": "deterministic-echo",
    "cache_identity": "vogon-adapters@0.1.0:deterministic-echo:v1",
    "parameters": {
      "mode": "offline"
    }
  },
  "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "steps": [
    {
      "step_id": "classify",
      "input_hash": "1111111111111111111111111111111111111111111111111111111111111111",
      "output_hash": "2222222222222222222222222222222222222222222222222222222222222222",
      "output": "done"
    }
  ]
}
"#
    }

    fn write_changelog(root: &Path, text: &str) {
        fs::write(root.join("CHANGELOG.md"), text).unwrap();
    }

    fn write_pr_template_docs(root: &Path, readme_commands: &[&str], template_commands: &[&str]) {
        fs::create_dir(root.join(".github")).unwrap();
        fs::write(
            root.join("README.md"),
            format!(
                "# README\n\nRun local checks:\n\n```sh\n{}\n```\n",
                readme_commands.join("\n")
            ),
        )
        .unwrap();
        fs::write(
            root.join(".github/pull_request_template.md"),
            format!(
                "## Verification\n\n{}\n",
                template_commands
                    .iter()
                    .map(|command| format!("- [ ] `{command}`"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        )
        .unwrap();
    }

    fn write_release_docs(root: &Path, readme_commands: &[&str], release_commands: &[&str]) {
        fs::create_dir(root.join("docs")).unwrap();
        fs::write(
            root.join("README.md"),
            format!(
                "# README\n\nRun local checks:\n\n```sh\n{}\n```\n",
                readme_commands.join("\n")
            ),
        )
        .unwrap();
        fs::write(
            root.join("docs").join("release.md"),
            format!(
                "# Release\n\nRun the full local verification set:\n\n```sh\n{}\n```\n",
                release_commands.join("\n")
            ),
        )
        .unwrap();
    }

    fn write_deployment_docs(
        root: &Path,
        deployment_commands: &[&str],
        readme_commands: &[&str],
        release_commands: &[&str],
    ) {
        fs::create_dir(root.join("docs")).unwrap();
        fs::write(
            root.join("README.md"),
            format!(
                "# README\n\nRun local checks:\n\n```sh\n{}\n```\n",
                readme_commands.join("\n")
            ),
        )
        .unwrap();
        fs::write(
            root.join("docs").join("release.md"),
            format!(
                "# Release\n\nRun the full local verification set:\n\n```sh\n{}\n```\n",
                release_commands.join("\n")
            ),
        )
        .unwrap();
        fs::write(
            root.join("docs").join("deployment.md"),
            format!(
                "# Deployment\n\nBefore publishing or deploying an image, run:\n\n```sh\n{}\n```\n",
                deployment_commands.join("\n")
            ),
        )
        .unwrap();
    }

    fn write_deployment_doc(root: &Path, body: &str) {
        fs::create_dir(root.join("docs")).unwrap();
        fs::write(
            root.join("docs").join("deployment.md"),
            format!("# Deployment\n\n{body}\n"),
        )
        .unwrap();
    }

    fn provider_credentials_section() -> String {
        let mut lines = vec!["## Provider Credentials".to_owned()];
        for (provider, env_var) in DEPLOYMENT_PROVIDER_EXAMPLES {
            lines.extend([
                String::new(),
                "```sh".to_owned(),
                "docker run --rm \\".to_owned(),
                format!("  -e {env_var} \\"),
                "  -v \"$PWD:/work\" \\".to_owned(),
                format!(
                    "  vogon-runtime:local run --provider {provider} fixtures/workflows/support-triage.toml"
                ),
                "```".to_owned(),
            ]);
        }
        lines.join("\n")
    }

    fn write_status_docs(root: &Path, readme: Option<&str>, security: Option<&str>) {
        fs::create_dir(root.join("docs")).unwrap();
        fs::write(
            root.join("README.md"),
            readme.unwrap_or(
                "# README\n\nVogon Runtime's latest public release is `v0.1.1`; `v0.1.0` was the first\npublic release. The project is still in the `0.x` series, so command and\nlibrary APIs may change as the runtime\nstabilizes.\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("SECURITY.md"),
            security.unwrap_or(
                "# Security\n\n`v0.1.1` is the latest public release of Vogon Runtime; `v0.1.0` was the first\npublic release. Security fixes are handled on the `main` branch and shipped in\nfollow-up patch or minor releases when they affect published artifacts.\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("SUPPORT.md"),
            "# Support\n\nVogon Runtime is released open-source software in the `0.x` series.\n",
        )
        .unwrap();
        fs::write(
            root.join("CHANGELOG.md"),
            "# Changelog\n\nand this project follows semantic versioning.\n\n## [0.1.1] - 2026-07-08\n\n## [0.1.0] - 2026-07-08\n",
        )
        .unwrap();
        fs::write(
            root.join("docs").join("release.md"),
            "# Release\n\nCrate publishing is manual while still in the `0.x` series.\n",
        )
        .unwrap();
    }

    fn write_package_verification_docs(
        root: &Path,
        package_command: &str,
        rationale: Option<&str>,
    ) {
        fs::create_dir(root.join("docs")).unwrap();
        let rationale = rationale.unwrap_or(
            "Cargo can fail offline verification while resolving unpublished internal workspace crates. The preceding build, test, docs, install, and smoke commands still verify compilation and CLI behavior.",
        );
        let text = format!("{package_command}\n\n{rationale}\n");
        fs::write(root.join("README.md"), &text).unwrap();
        fs::write(root.join("docs").join("release.md"), text).unwrap();
    }

    fn write_container_files(root: &Path, dockerignore: Option<&str>) {
        fs::write(
            root.join("Dockerfile"),
            [
                "# syntax=docker/dockerfile:1",
                "",
                "FROM rust:1.96.1-bookworm AS build",
                "",
                "WORKDIR /workspace",
                "",
                "ENV CARGO_INCREMENTAL=0",
                "ENV CARGO_NET_RETRY=10",
                "",
                "COPY Cargo.toml Cargo.lock rust-toolchain.toml ./",
                "COPY crates ./crates",
                "",
                "RUN cargo build --release --locked -p vogon-cli",
                "",
                "FROM debian:bookworm-slim AS runtime",
                "",
                "ARG VOGON_IMAGE_VERSION=dev",
                "ARG VOGON_IMAGE_REVISION=unknown",
                "",
                "LABEL org.opencontainers.image.title=\"Vogon Runtime\" \\",
                "    org.opencontainers.image.description=\"Deterministic, replayable AI workflow runtime CLI.\" \\",
                "    org.opencontainers.image.source=\"https://github.com/kaleab-kali/vogon-runtime\" \\",
                "    org.opencontainers.image.documentation=\"https://github.com/kaleab-kali/vogon-runtime#readme\" \\",
                "    org.opencontainers.image.licenses=\"MIT\" \\",
                "    org.opencontainers.image.version=\"${VOGON_IMAGE_VERSION}\" \\",
                "    org.opencontainers.image.revision=\"${VOGON_IMAGE_REVISION}\"",
                "",
                "RUN apt-get update \\",
                "    && apt-get install -y --no-install-recommends ca-certificates \\",
                "    && rm -rf /var/lib/apt/lists/* \\",
                "    && useradd --create-home --uid 10001 vogon",
                "",
                "COPY --from=build /workspace/target/release/vogon /usr/local/bin/vogon",
                "",
                "USER vogon",
                "WORKDIR /work",
                "ENTRYPOINT [\"vogon\"]",
            ]
            .join("\n"),
        )
        .unwrap();
        fs::write(
            root.join(".dockerignore"),
            dockerignore.unwrap_or(
                "/.git\n/.github\n/target\n.env\n.env.*\n!.env.example\n__pycache__/\n*.py[cod]\n*.cache.json\n",
            ),
        )
        .unwrap();
    }

    fn write_dependabot_config(root: &Path, text: &str) {
        let github = root.join(".github");
        fs::create_dir(&github).unwrap();
        fs::write(github.join("dependabot.yml"), text).unwrap();
    }

    fn dependabot_config_text() -> String {
        format!(
            "{}{}{}",
            "version: 2\n\
updates:\n\
  - package-ecosystem: cargo\n\
    directory: /\n\
    schedule:\n\
      interval: weekly\n\
    open-pull-requests-limit: 5\n",
            cargo_group_text(),
            "    commit-message:\n\
      prefix: deps\n\n\
  - package-ecosystem: github-actions\n\
    directory: /\n\
    schedule:\n\
      interval: weekly\n\
    open-pull-requests-limit: 5\n\
    groups:\n\
      github-actions-minor-patch:\n\
        patterns:\n\
          - \"*\"\n\
        update-types:\n\
          - minor\n\
          - patch\n\
    commit-message:\n\
      prefix: ci\n\n",
        ) + &docker_update_text()
    }

    fn cargo_group_text() -> String {
        "    groups:\n\
      cargo-minor-patch:\n\
        patterns:\n\
          - \"*\"\n\
        update-types:\n\
          - minor\n\
          - patch\n"
            .to_owned()
    }

    fn docker_update_text() -> String {
        "  - package-ecosystem: docker\n\
    directory: /\n\
    schedule:\n\
      interval: weekly\n\
    open-pull-requests-limit: 5\n\
    groups:\n\
      docker-minor-patch:\n\
        patterns:\n\
          - \"*\"\n\
        update-types:\n\
          - minor\n\
          - patch\n\
    commit-message:\n\
      prefix: deps\n"
            .to_owned()
    }

    fn write_issue_templates(root: &Path) {
        let template_dir = root.join(".github").join("ISSUE_TEMPLATE");
        fs::create_dir_all(&template_dir).unwrap();
        fs::write(
            template_dir.join("config.yml"),
            [
                "blank_issues_enabled: false",
                "contact_links:",
                "  - name: Security vulnerability",
                "    url: https://github.com/kaleab-kali/vogon-runtime/security/advisories/new",
            ]
            .join("\n")
                + "\n",
        )
        .unwrap();
        fs::write(
            template_dir.join("bug_report.yml"),
            valid_issue_form(
                "Bug report",
                "title: \"Bug: \"",
                "- bug",
                &[
                    "version",
                    "component",
                    "expected",
                    "actual",
                    "reproduce",
                    "environment",
                    "checks",
                ],
                true,
                None,
                "vogon 0.1.1",
            ),
        )
        .unwrap();
        fs::write(
            template_dir.join("feature_request.yml"),
            valid_issue_form(
                "Feature request",
                "title: \"Feature: \"",
                "- enhancement",
                &["problem", "proposal", "area", "checks"],
                true,
                None,
                "vogon 0.1.1",
            ),
        )
        .unwrap();
    }

    fn valid_issue_form(
        name: &str,
        title: &str,
        label: &str,
        fields: &[&str],
        include_secret_check: bool,
        options: Option<&[&str]>,
        version_placeholder: &str,
    ) -> String {
        let dropdown_options = options.unwrap_or(&[
            "CLI",
            "Runtime",
            "Replay verification",
            "Provider adapter",
            "Documentation",
            "Release artifact",
            "Other",
        ]);
        let mut lines = vec![
            format!("name: {name}"),
            "description: Example form.".to_owned(),
            title.to_owned(),
            "labels:".to_owned(),
            format!("  {label}"),
            "body:".to_owned(),
        ];

        for field in fields {
            if matches!(*field, "component" | "area") {
                lines.extend([
                    "  - type: dropdown".to_owned(),
                    format!("    id: {field}"),
                    "    attributes:".to_owned(),
                    "      label: Area".to_owned(),
                    "      options:".to_owned(),
                ]);
                lines.extend(
                    dropdown_options
                        .iter()
                        .map(|option| format!("        - {option}")),
                );
                lines.extend([
                    "    validations:".to_owned(),
                    "      required: true".to_owned(),
                ]);
            } else if *field == "checks" {
                lines.extend([
                    "  - type: checkboxes".to_owned(),
                    "    id: checks".to_owned(),
                    "    attributes:".to_owned(),
                    "      label: Before submitting".to_owned(),
                    "      options:".to_owned(),
                ]);
                if include_secret_check {
                    lines.extend([
                        "        - label: I have removed secrets, API keys, private prompts, and sensitive replay data.".to_owned(),
                        "          required: true".to_owned(),
                    ]);
                }
                lines.extend([
                    "        - label: I searched existing issues for a similar report.".to_owned(),
                    "          required: true".to_owned(),
                ]);
            } else if *field == "version" {
                lines.extend([
                    "  - type: input".to_owned(),
                    "    id: version".to_owned(),
                    "    attributes:".to_owned(),
                    "      label: version".to_owned(),
                    format!("      placeholder: \"{version_placeholder}\""),
                    "    validations:".to_owned(),
                    "      required: true".to_owned(),
                ]);
            } else {
                lines.extend([
                    "  - type: textarea".to_owned(),
                    format!("    id: {field}"),
                    "    attributes:".to_owned(),
                    format!("      label: {field}"),
                    "    validations:".to_owned(),
                    "      required: true".to_owned(),
                ]);
            }
        }

        lines.join("\n") + "\n"
    }

    fn write_contributing_docs(
        root: &Path,
        readme_commands: &[&str],
        contributing_commands: &[&str],
        live_guidance: &str,
    ) {
        fs::write(
            root.join("README.md"),
            format!(
                "# README\n\nRun local checks:\n\n```sh\n{}\n```\n",
                readme_commands.join("\n")
            ),
        )
        .unwrap();
        fs::write(
            root.join("CONTRIBUTING.md"),
            format!(
                "# Contributing\n\n## Development\n\n```sh\n{}\n```\n{}",
                contributing_commands.join("\n"),
                live_guidance
            ),
        )
        .unwrap();
    }

    fn live_guidance_text() -> &'static str {
        "\n- `Live Gemini Smoke` uses `GEMINI_API_KEY`.\n- `Live Groq Smoke` uses `GROQ_API_KEY`.\n- `Live Hugging Face Smoke` uses `HF_TOKEN`.\n- `Live OpenAI-Compatible Smoke` uses `OPENAI_COMPATIBLE_API_KEY`.\n- `Live OpenRouter Smoke` uses `OPENROUTER_API_KEY`.\n"
    }

    #[derive(Default)]
    struct WorkspaceOptions<'a> {
        workspace_package: Option<&'a str>,
        adapters_dependency_version: &'a str,
        release_profile: Option<&'a str>,
        workspace_lints: Option<&'a str>,
        crate_lints: Option<&'a str>,
    }

    fn write_workspace(root: &Path, mut options: WorkspaceOptions<'_>) {
        if options.adapters_dependency_version.is_empty() {
            options.adapters_dependency_version = "0.1.0";
        }

        fs::write(root.join("README.md"), "# Vogon Runtime\n").unwrap();
        fs::write(
            root.join("Cargo.toml"),
            format!(
                r#"[workspace]
resolver = "3"
members = [
    "crates/vogon-adapters",
    "crates/vogon-cli",
    "crates/vogon-core",
    "crates/vogon-xtask",
]

[workspace.package]
{}[workspace.dependencies]
vogon-adapters = {{ version = "{}", path = "crates/vogon-adapters" }}
vogon-core = {{ version = "0.1.0", path = "crates/vogon-core" }}
{}{}"#,
                options
                    .workspace_package
                    .unwrap_or(&workspace_package_text()),
                options.adapters_dependency_version,
                options.workspace_lints.unwrap_or(&workspace_lints_text()),
                options.release_profile.unwrap_or(&release_profile_text()),
            ),
        )
        .unwrap();
        write_crate_manifest(
            root,
            "vogon-core",
            "Core deterministic workflow runtime for Vogon Runtime.",
            &["ai", "workflow", "replay", "runtime"],
            &["development-tools"],
            options.crate_lints,
        );
        write_crate_manifest(
            root,
            "vogon-adapters",
            "Model adapters for Vogon Runtime.",
            &["ai", "model-adapters", "workflow", "runtime"],
            &["development-tools"],
            options.crate_lints,
        );
        write_crate_manifest(
            root,
            "vogon-cli",
            "Command-line interface for Vogon Runtime.",
            &["ai", "workflow", "replay", "cli"],
            &["command-line-utilities", "development-tools"],
            options.crate_lints,
        );
        write_crate_manifest(
            root,
            "vogon-xtask",
            "Repository maintenance tasks for Vogon Runtime.",
            &["workflow", "tooling", "ci", "maintenance"],
            &["development-tools"],
            options.crate_lints,
        );
    }

    fn workspace_package_text() -> String {
        r#"edition = "2024"
rust-version = "1.85"
license = "MIT"
repository = "https://github.com/kaleab-kali/vogon-runtime"
homepage = "https://github.com/kaleab-kali/vogon-runtime"
documentation = "https://github.com/kaleab-kali/vogon-runtime/tree/main/docs"
authors = ["Vogon Runtime Contributors"]
"#
        .to_owned()
    }

    fn release_profile_text() -> String {
        r#"
[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"
"#
        .to_owned()
    }

    fn workspace_lints_text() -> String {
        r#"
[workspace.lints.rust]
unsafe_code = "forbid"
"#
        .to_owned()
    }

    fn write_crate_manifest(
        root: &Path,
        name: &str,
        description: &str,
        keywords: &[&str],
        categories: &[&str],
        crate_lints: Option<&str>,
    ) {
        let crate_dir = root.join("crates").join(name);
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            format!(
                r#"[package]
name = "{name}"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
authors.workspace = true
description = "{description}"
readme = "../../README.md"
keywords = {}
categories = {}
{}"#,
                toml_string_array(keywords),
                toml_string_array(categories),
                crate_lints.unwrap_or(&crate_lints_text()),
            ),
        )
        .unwrap();
    }

    fn crate_lints_text() -> String {
        r#"
[lints]
workspace = true
"#
        .to_owned()
    }

    fn toml_string_array(values: &[&str]) -> String {
        let values = values
            .iter()
            .map(|value| format!("\"{value}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{values}]")
    }
}
