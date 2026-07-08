use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

type TomlTable = toml::Table;

const EXPECTED_ENV_VARS: &[&str] = &[
    "GEMINI_API_KEY",
    "GROQ_API_KEY",
    "HF_TOKEN",
    "OPENAI_COMPATIBLE_API_KEY",
    "OPENROUTER_API_KEY",
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
const REQUIRED_README_COMMANDS: &[&str] = &[
    "python -m unittest scripts.test_check_sha256_file",
    "python -m unittest scripts.test_check_archive_contents",
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

#[derive(Clone, Copy)]
enum ExpectedValue {
    Integer(i64),
    String(&'static str),
    StringList(&'static [&'static str]),
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage_and_exit();
    };

    let result = match command.as_str() {
        "check-env-example" => {
            let root = parse_root(args.collect());
            check_env_example(&root)
        }
        "check-cargo-manifests" => {
            let root = parse_root(args.collect());
            check_cargo_manifests(&root)
        }
        "check-changelog" => {
            let root = parse_root(args.collect());
            check_changelog(&root)
        }
        "check-contributing-checklist" => {
            let root = parse_root(args.collect());
            check_contributing_checklist(&root)
        }
        "check-deployment-checklist" => {
            let root = parse_root(args.collect());
            check_deployment_checklist(&root)
        }
        "check-package-verification-docs" => {
            let root = parse_root(args.collect());
            check_package_verification_docs(&root)
        }
        "check-pr-template" => {
            let root = parse_root(args.collect());
            check_pr_template(&root)
        }
        "check-public-status-docs" => {
            let root = parse_root(args.collect());
            check_public_status_docs(&root)
        }
        "check-release-checklist" => {
            let root = parse_root(args.collect());
            check_release_checklist(&root)
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

fn print_usage_and_exit() -> ! {
    eprintln!(
        "usage: cargo run -p vogon-xtask -- <check-cargo-manifests|check-changelog|check-contributing-checklist|check-deployment-checklist|check-env-example|check-package-verification-docs|check-pr-template|check-public-status-docs|check-release-checklist> [--root PATH]"
    );
    std::process::exit(2);
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
            &[
                "cargo test",
                "python -m unittest scripts.test_check_sha256_file",
                "python -m unittest scripts.test_check_archive_contents",
                "python scripts/check_docs_links.py --root .",
            ],
            &[
                "cargo test",
                "python -m unittest scripts.test_check_sha256_file",
                "python -m unittest scripts.test_check_archive_contents",
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
            &[
                "cargo test",
                "cargo clippy -- -D warnings",
                "python -m unittest scripts.test_check_sha256_file",
                "python -m unittest scripts.test_check_archive_contents",
            ],
            &[
                "cargo test",
                "python -m unittest scripts.test_check_sha256_file",
                "python -m unittest scripts.test_check_archive_contents",
            ],
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
    fn reports_missing_required_readme_release_validator_tests() {
        let root = temp_root("contributing-missing-required-readme");
        write_contributing_docs(
            &root,
            &["cargo test"],
            &["cargo test"],
            live_guidance_text(),
        );

        let errors = check_contributing_checklist(&root).unwrap_err();

        assert_eq!(
            errors,
            [
                "README.md: missing required local check `python -m unittest scripts.test_check_sha256_file`",
                "README.md: missing required local check `python -m unittest scripts.test_check_archive_contents`",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_live_workflow_guidance() {
        let root = temp_root("contributing-missing-live-guidance");
        write_contributing_docs(
            &root,
            &[
                "cargo test",
                "python -m unittest scripts.test_check_sha256_file",
                "python -m unittest scripts.test_check_archive_contents",
            ],
            &[
                "cargo test",
                "python -m unittest scripts.test_check_sha256_file",
                "python -m unittest scripts.test_check_archive_contents",
            ],
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
                "README.md: missing required local check `python -m unittest scripts.test_check_sha256_file`",
                "README.md: missing required local check `python -m unittest scripts.test_check_archive_contents`",
            ]
        );
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
