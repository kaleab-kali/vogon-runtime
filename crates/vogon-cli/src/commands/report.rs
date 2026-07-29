use std::{fmt::Write as _, fs, io, path::Path};

use vogon_core::{DecisionOutcome, RedactionSet, RunReport};

use crate::commands::{
    file_io, redaction::parse_redactions, redaction_markers::validate_redaction_markers,
};

const MAX_REPORT_BYTES: usize = 8 * 1024 * 1024;

pub fn run(
    replay_file: &Path,
    output: &Path,
    redaction_values: &[String],
    redaction_environment_values: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    reject_source_as_output(replay_file, output)?;
    let replay_text = file_io::read_to_string(replay_file, "replay file")?;
    let replay: RunReport = serde_json::from_str(&replay_text).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "failed to parse replay file `{}`: {error}",
                replay_file.display()
            ),
        )
    })?;
    validate_redaction_markers(&replay)?;
    replay.validate_integrity().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "replay integrity check failed for `{}`: {error}",
                replay_file.display()
            ),
        )
    })?;
    let redactions = parse_redactions(redaction_values, redaction_environment_values)?;
    let html = render_html(&replay, &redactions);
    if html.len() > MAX_REPORT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "HTML report `{}` is {} bytes, exceeding the 8 MiB limit",
                output.display(),
                html.len()
            ),
        )
        .into());
    }

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!(
                        "failed to create report directory `{}`: {error}",
                        parent.display()
                    ),
                )
            })?;
        }
    }
    fs::write(output, html).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "failed to write HTML report `{}`: {error}",
                output.display()
            ),
        )
    })?;
    println!("HTML evidence report written: {}", output.display());
    Ok(())
}

fn reject_source_as_output(replay_file: &Path, output: &Path) -> io::Result<()> {
    let paths_match = replay_file == output
        || fs::canonicalize(replay_file)
            .ok()
            .zip(fs::canonicalize(output).ok())
            .is_some_and(|(replay, report)| replay == report);
    if paths_match {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "HTML report output `{}` must differ from the source replay",
                output.display()
            ),
        ));
    }
    Ok(())
}

fn render_html(replay: &RunReport, redactions: &RedactionSet) -> String {
    let mut html = String::with_capacity(16 * 1024);
    html.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    html.push_str(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src 'none'; font-src 'none'; connect-src 'none'; base-uri 'none'; form-action 'none'\">",
    );
    write!(
        html,
        "<title>{} | Vogon evidence</title>",
        escape_html(&replay.workflow_name)
    )
    .unwrap();
    html.push_str("<style>");
    html.push_str(STYLES);
    html.push_str("</style></head><body>");
    html.push_str(
        "<header class=\"topbar\"><span class=\"brand\">VOGON RUNTIME</span><span class=\"artifact\">Replay evidence</span></header>",
    );
    html.push_str("<main class=\"page\">");

    let (status_class, status_label, status_value, status_detail) = match &replay.decision {
        Some(decision) if decision.outcome == DecisionOutcome::Deny => (
            "deny",
            "RELEASE BLOCKED",
            redactions.redact(&decision.value),
            "The recorded workflow decision denied this change.",
        ),
        Some(decision) => (
            "allow",
            "GATE PASSED",
            redactions.redact(&decision.value),
            "The recorded workflow decision allowed this change.",
        ),
        None => (
            "recorded",
            "RUN RECORDED",
            "NO DECISION POLICY".to_owned(),
            "This replay records execution evidence without a gate decision.",
        ),
    };

    html.push_str("<section class=\"intro\" aria-labelledby=\"report-title\"><div>");
    html.push_str("<p class=\"eyebrow\">Workflow evidence</p>");
    write!(
        html,
        "<h1 id=\"report-title\">{}</h1>",
        escape_html(&replay.workflow_name)
    )
    .unwrap();
    html.push_str("</div>");
    write!(
        html,
        "<div class=\"status {}\"><span class=\"status-label\">{}</span><strong>{}</strong><span>{}</span></div>",
        status_class,
        status_label,
        escape_html(&status_value),
        status_detail
    )
    .unwrap();
    html.push_str("</section>");

    html.push_str("<section class=\"facts\" aria-label=\"Run summary\">");
    fact(&mut html, "Provider", &replay.runtime.provider);
    fact(
        &mut html,
        "Model",
        replay.runtime.model.as_deref().unwrap_or("Not reported"),
    );
    fact(&mut html, "Steps", &replay.steps.len().to_string());
    fact(
        &mut html,
        "Replay schema",
        &replay.schema_version.to_string(),
    );
    html.push_str("</section>");

    if let Some(decision) = &replay.decision {
        html.push_str("<section class=\"section decision\" aria-labelledby=\"decision-title\">");
        html.push_str("<div class=\"section-heading\"><p class=\"eyebrow\">Decision evidence</p><h2 id=\"decision-title\">Release assessment</h2></div>");
        html.push_str("<div class=\"decision-meta\">");
        fact(&mut html, "Decision step", decision.step_id.as_str());
        fact(&mut html, "JSON pointer", &decision.pointer);
        fact(
            &mut html,
            "Outcome",
            match decision.outcome {
                DecisionOutcome::Allow => "allow",
                DecisionOutcome::Deny => "deny",
            },
        );
        html.push_str("</div>");

        if let Some(step) = replay
            .steps
            .iter()
            .find(|step| step.step_id == decision.step_id)
        {
            let output = redactions.redact(&step.output);
            let details = decision_details(&output);
            if !details.reasons.is_empty() || !details.required_actions.is_empty() {
                html.push_str("<div class=\"decision-columns\">");
                decision_list(&mut html, "Grounded reasons", &details.reasons);
                decision_list(&mut html, "Required actions", &details.required_actions);
                html.push_str("</div>");
            }
        }
        html.push_str("</section>");
    }

    html.push_str("<section class=\"section\" aria-labelledby=\"steps-title\">");
    html.push_str("<div class=\"section-heading\"><p class=\"eyebrow\">Execution evidence</p><h2 id=\"steps-title\">Recorded steps</h2></div>");
    html.push_str("<div class=\"steps\">");
    for (index, step) in replay.steps.iter().enumerate() {
        write!(
            html,
            "<article class=\"step\"><div class=\"step-heading\"><span class=\"step-index\">{:02}</span><h3>{}</h3></div>",
            index + 1,
            escape_html(step.step_id.as_str())
        )
        .unwrap();
        write!(
            html,
            "<pre>{}</pre>",
            escape_html(&redactions.redact(&step.output))
        )
        .unwrap();
        html.push_str("<dl class=\"hashes\">");
        if let Some(prompt_hash) = step.prompt_hash.as_deref() {
            hash(&mut html, "Prompt hash", prompt_hash);
        }
        hash(&mut html, "Input hash", &step.input_hash);
        hash(&mut html, "Output hash", &step.output_hash);
        html.push_str("</dl></article>");
    }
    html.push_str("</div></section>");

    html.push_str("<section class=\"section integrity\" aria-labelledby=\"integrity-title\">");
    html.push_str("<div class=\"section-heading\"><p class=\"eyebrow\">Integrity</p><h2 id=\"integrity-title\">Self-consistency check passed</h2></div>");
    html.push_str("<p>Vogon recomputed every output hash and the aggregate run hash before creating this page.</p>");
    html.push_str("<dl class=\"hashes primary-hashes\">");
    hash(&mut html, "Run hash", &replay.run_hash);
    if let Some(decision) = &replay.decision {
        hash(&mut html, "Decision policy hash", &decision.policy_hash);
    }
    if let Some(policy_hash) = replay.execution_policy_hash.as_deref() {
        hash(&mut html, "Execution policy hash", policy_hash);
    }
    html.push_str("</dl>");
    html.push_str("<p class=\"caveat\">Hash consistency is not a digital signature, provider attestation, or proof that the model decision is correct. Retain the original replay, deterministic checks, and required human review.</p>");
    html.push_str("</section>");

    write!(
        html,
        "<footer>Generated locally by Vogon Runtime {}. No external assets or network requests.</footer>",
        env!("CARGO_PKG_VERSION")
    )
    .unwrap();
    html.push_str("</main></body></html>");
    html
}

fn fact(html: &mut String, label: &str, value: &str) {
    write!(
        html,
        "<div class=\"fact\"><span>{}</span><strong>{}</strong></div>",
        escape_html(label),
        escape_html(value)
    )
    .unwrap();
}

fn hash(html: &mut String, label: &str, value: &str) {
    write!(
        html,
        "<div><dt>{}</dt><dd><code>{}</code></dd></div>",
        escape_html(label),
        escape_html(value)
    )
    .unwrap();
}

fn decision_list(html: &mut String, title: &str, values: &[String]) {
    write!(html, "<div><h3>{}</h3><ul>", escape_html(title)).unwrap();
    if values.is_empty() {
        html.push_str("<li>None recorded in the conventional decision field.</li>");
    } else {
        for value in values {
            write!(html, "<li>{}</li>", escape_html(value)).unwrap();
        }
    }
    html.push_str("</ul></div>");
}

#[derive(Default)]
struct DecisionDetails {
    reasons: Vec<String>,
    required_actions: Vec<String>,
}

fn decision_details(output: &str) -> DecisionDetails {
    let Ok(document) = serde_json::from_str::<serde_json::Value>(output) else {
        return DecisionDetails::default();
    };
    DecisionDetails {
        reasons: string_array(&document, "reasons"),
        required_actions: string_array(&document, "required_actions"),
    }
}

fn string_array(document: &serde_json::Value, key: &str) -> Vec<String> {
    document
        .get(key)
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

const STYLES: &str = r#"
:root {
  color-scheme: light;
  --ink: #17202a;
  --muted: #64717d;
  --line: #d9dee3;
  --surface: #f6f8f9;
  --white: #ffffff;
  --red: #b42318;
  --red-bg: #fff1f0;
  --green: #18794e;
  --green-bg: #edf8f2;
  --blue: #175cd3;
  --blue-bg: #eef4ff;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  background: var(--white);
  color: var(--ink);
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 15px;
  line-height: 1.55;
}
.topbar {
  min-height: 58px;
  padding: 0 32px;
  border-bottom: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #111820;
  color: var(--white);
}
.brand { font-size: 13px; font-weight: 800; letter-spacing: 0; }
.artifact { color: #b9c3cc; font-size: 13px; }
.page { width: min(1120px, calc(100% - 40px)); margin: 0 auto; padding: 54px 0 36px; }
.intro { display: grid; grid-template-columns: minmax(0, 1fr) 360px; gap: 36px; align-items: center; }
.eyebrow { margin: 0 0 8px; color: var(--blue); font-size: 12px; font-weight: 800; text-transform: uppercase; }
h1, h2, h3 { margin: 0; letter-spacing: 0; line-height: 1.2; }
h1 { font-size: 32px; overflow-wrap: anywhere; }
h2 { font-size: 22px; }
h3 { font-size: 15px; }
.status {
  min-height: 112px;
  border-left: 5px solid;
  padding: 18px 20px;
  display: grid;
  align-content: center;
  gap: 3px;
}
.status-label { font-size: 11px; font-weight: 800; }
.status strong { font-size: 25px; overflow-wrap: anywhere; }
.status > span:last-child { color: var(--muted); font-size: 13px; }
.status.deny { border-color: var(--red); background: var(--red-bg); }
.status.deny .status-label, .status.deny strong { color: var(--red); }
.status.allow { border-color: var(--green); background: var(--green-bg); }
.status.allow .status-label, .status.allow strong { color: var(--green); }
.status.recorded { border-color: var(--blue); background: var(--blue-bg); }
.status.recorded .status-label, .status.recorded strong { color: var(--blue); }
.facts, .decision-meta {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  margin-top: 36px;
  border-top: 1px solid var(--line);
  border-bottom: 1px solid var(--line);
}
.fact { min-width: 0; padding: 17px 20px 17px 0; }
.fact + .fact { border-left: 1px solid var(--line); padding-left: 20px; }
.fact span { display: block; color: var(--muted); font-size: 12px; }
.fact strong { display: block; margin-top: 3px; overflow-wrap: anywhere; }
.section { margin-top: 54px; }
.section-heading { display: flex; justify-content: space-between; align-items: end; gap: 20px; padding-bottom: 14px; border-bottom: 2px solid var(--ink); }
.section-heading .eyebrow { margin: 0; }
.decision-meta { grid-template-columns: repeat(3, minmax(0, 1fr)); margin-top: 0; border-top: 0; }
.decision-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 36px; margin-top: 26px; }
.decision-columns > div { border-left: 4px solid var(--line); padding-left: 18px; }
.decision-columns ul { margin: 12px 0 0; padding-left: 20px; }
.decision-columns li + li { margin-top: 8px; }
.steps { display: grid; gap: 18px; margin-top: 22px; }
.step { border: 1px solid var(--line); border-radius: 6px; overflow: hidden; }
.step-heading { min-height: 48px; padding: 0 16px; display: flex; align-items: center; gap: 12px; background: var(--surface); border-bottom: 1px solid var(--line); }
.step-index { color: var(--blue); font: 700 12px ui-monospace, SFMono-Regular, Consolas, monospace; }
pre {
  margin: 0;
  padding: 18px;
  overflow: auto;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  background: var(--white);
  color: #202b35;
  font: 13px/1.6 ui-monospace, SFMono-Regular, Consolas, monospace;
}
.hashes { margin: 0; border-top: 1px solid var(--line); background: var(--surface); }
.hashes > div { display: grid; grid-template-columns: 150px minmax(0, 1fr); padding: 9px 16px; gap: 16px; }
.hashes > div + div { border-top: 1px solid var(--line); }
.hashes dt { color: var(--muted); font-size: 12px; }
.hashes dd { min-width: 0; margin: 0; }
code { font: 11px/1.5 ui-monospace, SFMono-Regular, Consolas, monospace; overflow-wrap: anywhere; }
.integrity > p { max-width: 760px; }
.primary-hashes { margin-top: 18px; border: 1px solid var(--line); }
.caveat { padding: 14px 16px; border-left: 4px solid #d4a72c; background: #fff9e6; color: #514515; }
footer { margin-top: 58px; padding-top: 18px; border-top: 1px solid var(--line); color: var(--muted); font-size: 12px; }
@media (max-width: 760px) {
  .topbar { padding: 0 20px; }
  .page { width: min(100% - 28px, 1120px); padding-top: 32px; }
  .intro { grid-template-columns: 1fr; gap: 24px; }
  h1 { font-size: 27px; }
  .facts, .decision-meta { grid-template-columns: 1fr 1fr; }
  .fact + .fact { border-left: 0; padding-left: 0; }
  .fact:nth-child(even) { border-left: 1px solid var(--line); padding-left: 16px; }
  .decision-columns { grid-template-columns: 1fr; gap: 22px; }
  .section-heading { display: block; }
  .section-heading h2 { margin-top: 6px; }
  .hashes > div { grid-template-columns: 1fr; gap: 3px; }
}
@media print {
  .topbar { background: var(--white); color: var(--ink); }
  .artifact { color: var(--muted); }
  .page { width: 100%; padding: 24px 0; }
  .step { break-inside: avoid; }
}
"#;

#[cfg(test)]
mod tests {
    use super::{decision_details, escape_html};

    #[test]
    fn escapes_active_html_content() {
        assert_eq!(
            escape_html("<script title=\"x\">'&'"),
            "&lt;script title=&quot;x&quot;&gt;&#39;&amp;&#39;"
        );
    }

    #[test]
    fn extracts_conventional_decision_details() {
        let details = decision_details(
            r#"{"decision":"NO_GO","reasons":["unsafe"],"required_actions":["restore"]}"#,
        );
        assert_eq!(details.reasons, ["unsafe"]);
        assert_eq!(details.required_actions, ["restore"]);
    }
}
