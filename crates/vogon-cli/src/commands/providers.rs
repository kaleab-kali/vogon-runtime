use serde::Serialize;

use crate::commands::run::{
    DEFAULT_GROQ_BASE_URL, DEFAULT_GROQ_MODEL, DEFAULT_HUGGING_FACE_BASE_URL,
    DEFAULT_HUGGING_FACE_MODEL, DEFAULT_OPENAI_COMPATIBLE_BASE_URL,
    DEFAULT_OPENAI_COMPATIBLE_MODEL, DEFAULT_OPENROUTER_BASE_URL, DEFAULT_OPENROUTER_MODEL,
};

pub fn run(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let report = ProviderDiagnostics {
        providers: provider_statuses(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct ProviderDiagnostics {
    providers: Vec<ProviderStatus>,
}

#[derive(Debug, Serialize)]
struct ProviderStatus {
    name: &'static str,
    enabled: bool,
    default: bool,
    credential_env: Option<&'static str>,
    credential_configured: Option<bool>,
    default_base_url: Option<&'static str>,
    default_model: Option<&'static str>,
}

fn provider_statuses() -> Vec<ProviderStatus> {
    vec![
        ProviderStatus {
            name: "deterministic",
            enabled: true,
            default: true,
            credential_env: None,
            credential_configured: None,
            default_base_url: None,
            default_model: None,
        },
        gemini_status(),
        groq_status(),
        hugging_face_status(),
        openrouter_status(),
        openai_compatible_status(),
    ]
}

#[cfg(feature = "gemini")]
fn gemini_status() -> ProviderStatus {
    ProviderStatus {
        name: "gemini",
        enabled: true,
        default: false,
        credential_env: Some("GEMINI_API_KEY"),
        credential_configured: Some(env_is_configured("GEMINI_API_KEY")),
        default_base_url: None,
        default_model: Some("gemini-3.1-flash-lite"),
    }
}

#[cfg(not(feature = "gemini"))]
fn gemini_status() -> ProviderStatus {
    ProviderStatus {
        name: "gemini",
        enabled: false,
        default: false,
        credential_env: Some("GEMINI_API_KEY"),
        credential_configured: None,
        default_base_url: None,
        default_model: Some("gemini-3.1-flash-lite"),
    }
}

#[cfg(feature = "openai-compatible")]
fn groq_status() -> ProviderStatus {
    ProviderStatus {
        name: "groq",
        enabled: true,
        default: false,
        credential_env: Some("GROQ_API_KEY"),
        credential_configured: Some(env_is_configured("GROQ_API_KEY")),
        default_base_url: Some(DEFAULT_GROQ_BASE_URL),
        default_model: Some(DEFAULT_GROQ_MODEL),
    }
}

#[cfg(not(feature = "openai-compatible"))]
fn groq_status() -> ProviderStatus {
    ProviderStatus {
        name: "groq",
        enabled: false,
        default: false,
        credential_env: Some("GROQ_API_KEY"),
        credential_configured: None,
        default_base_url: Some(DEFAULT_GROQ_BASE_URL),
        default_model: Some(DEFAULT_GROQ_MODEL),
    }
}

#[cfg(feature = "openai-compatible")]
fn hugging_face_status() -> ProviderStatus {
    ProviderStatus {
        name: "hugging-face",
        enabled: true,
        default: false,
        credential_env: Some("HF_TOKEN"),
        credential_configured: Some(env_is_configured("HF_TOKEN")),
        default_base_url: Some(DEFAULT_HUGGING_FACE_BASE_URL),
        default_model: Some(DEFAULT_HUGGING_FACE_MODEL),
    }
}

#[cfg(not(feature = "openai-compatible"))]
fn hugging_face_status() -> ProviderStatus {
    ProviderStatus {
        name: "hugging-face",
        enabled: false,
        default: false,
        credential_env: Some("HF_TOKEN"),
        credential_configured: None,
        default_base_url: Some(DEFAULT_HUGGING_FACE_BASE_URL),
        default_model: Some(DEFAULT_HUGGING_FACE_MODEL),
    }
}

#[cfg(feature = "openai-compatible")]
fn openrouter_status() -> ProviderStatus {
    ProviderStatus {
        name: "openrouter",
        enabled: true,
        default: false,
        credential_env: Some("OPENROUTER_API_KEY"),
        credential_configured: Some(env_is_configured("OPENROUTER_API_KEY")),
        default_base_url: Some(DEFAULT_OPENROUTER_BASE_URL),
        default_model: Some(DEFAULT_OPENROUTER_MODEL),
    }
}

#[cfg(not(feature = "openai-compatible"))]
fn openrouter_status() -> ProviderStatus {
    ProviderStatus {
        name: "openrouter",
        enabled: false,
        default: false,
        credential_env: Some("OPENROUTER_API_KEY"),
        credential_configured: None,
        default_base_url: Some(DEFAULT_OPENROUTER_BASE_URL),
        default_model: Some(DEFAULT_OPENROUTER_MODEL),
    }
}

#[cfg(feature = "openai-compatible")]
fn openai_compatible_status() -> ProviderStatus {
    ProviderStatus {
        name: "openai-compatible",
        enabled: true,
        default: false,
        credential_env: Some("OPENAI_COMPATIBLE_API_KEY"),
        credential_configured: Some(env_is_configured("OPENAI_COMPATIBLE_API_KEY")),
        default_base_url: Some(DEFAULT_OPENAI_COMPATIBLE_BASE_URL),
        default_model: Some(DEFAULT_OPENAI_COMPATIBLE_MODEL),
    }
}

#[cfg(not(feature = "openai-compatible"))]
fn openai_compatible_status() -> ProviderStatus {
    ProviderStatus {
        name: "openai-compatible",
        enabled: false,
        default: false,
        credential_env: Some("OPENAI_COMPATIBLE_API_KEY"),
        credential_configured: None,
        default_base_url: Some(DEFAULT_OPENAI_COMPATIBLE_BASE_URL),
        default_model: Some(DEFAULT_OPENAI_COMPATIBLE_MODEL),
    }
}

#[cfg(any(feature = "gemini", feature = "openai-compatible"))]
fn env_is_configured(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

fn print_human(report: &ProviderDiagnostics) {
    println!("Providers:");
    for provider in &report.providers {
        let enabled = if provider.enabled {
            "enabled"
        } else {
            "disabled"
        };
        let default = if provider.default { " default" } else { "" };
        println!("- {}: {enabled}{default}", provider.name);

        if let Some(env_name) = provider.credential_env {
            let configured = match provider.credential_configured {
                Some(true) => "configured",
                Some(false) => "missing",
                None => "not checked because provider support is disabled",
            };
            println!("  credential: {env_name} ({configured})");
        } else {
            println!("  credential: not required");
        }

        if let Some(base_url) = provider.default_base_url {
            println!("  default base URL: {base_url}");
        }

        if let Some(model) = provider.default_model {
            println!("  default model: {model}");
        }
    }
}
