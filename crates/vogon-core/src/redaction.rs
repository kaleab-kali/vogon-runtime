use serde::{Deserialize, Serialize};
use std::cmp::Reverse;

use crate::{Result, VogonError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionRule {
    pub label: String,
    pub literal: String,
}

impl RedactionRule {
    pub fn new(label: impl Into<String>, literal: impl Into<String>) -> Result<Self> {
        let label = label.into();
        let literal = literal.into();
        let trimmed_label = label.trim();

        if trimmed_label.is_empty() {
            return Err(VogonError::EmptyRedactionLabel);
        }

        if label != trimmed_label {
            return Err(VogonError::InvalidRedactionLabel(label));
        }

        if !label
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        {
            return Err(VogonError::InvalidRedactionLabel(label));
        }

        if literal.is_empty() {
            return Err(VogonError::EmptyRedactionLiteral);
        }

        Ok(Self { label, literal })
    }

    pub fn replacement(&self) -> String {
        format!("[REDACTED:{}]", self.label)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionSet {
    rules: Vec<RedactionRule>,
}

impl RedactionSet {
    pub fn new(mut rules: Vec<RedactionRule>) -> Self {
        rules.sort_by_key(|rule| Reverse(rule.literal.len()));

        Self { rules }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn rules(&self) -> &[RedactionRule] {
        &self.rules
    }

    pub fn redact(&self, value: &str) -> String {
        self.rules.iter().fold(value.to_owned(), |redacted, rule| {
            redacted.replace(&rule.literal, &rule.replacement())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{RedactionRule, RedactionSet, VogonError};

    #[test]
    fn redaction_rule_rejects_empty_literals() {
        let result = RedactionRule::new("api_key", "");

        assert_eq!(result.unwrap_err(), VogonError::EmptyRedactionLiteral);
    }

    #[test]
    fn redaction_rule_rejects_whitespace_labels() {
        let result = RedactionRule::new(" api_key ", "sk-test-123");

        assert_eq!(
            result.unwrap_err(),
            VogonError::InvalidRedactionLabel(" api_key ".to_owned())
        );
    }

    #[test]
    fn redaction_set_replaces_known_literals() {
        let redactions =
            RedactionSet::new(vec![RedactionRule::new("api_key", "sk-test-123").unwrap()]);

        assert_eq!(
            redactions.redact("token=sk-test-123"),
            "token=[REDACTED:api_key]"
        );
    }

    #[test]
    fn redaction_set_prefers_longest_overlapping_literals() {
        let redactions = RedactionSet::new(vec![
            RedactionRule::new("prefix", "sk-test").unwrap(),
            RedactionRule::new("api_key", "sk-test-123").unwrap(),
        ]);

        assert_eq!(
            redactions.redact("token=sk-test-123"),
            "token=[REDACTED:api_key]"
        );
    }
}
