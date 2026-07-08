use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_ENV_VARS: &[&str] = &[
    "GEMINI_API_KEY",
    "GROQ_API_KEY",
    "HF_TOKEN",
    "OPENAI_COMPATIBLE_API_KEY",
    "OPENROUTER_API_KEY",
];

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
    eprintln!("usage: cargo run -p vogon-xtask -- check-env-example [--root PATH]");
    std::process::exit(2);
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
}
