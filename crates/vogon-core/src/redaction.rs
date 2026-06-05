use serde::{Deserialize, Deserializer, Serialize, de};
use std::{cmp::Reverse, collections::BTreeSet};

use crate::{Result, VogonError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

impl<'de> Deserialize<'de> for RedactionRule {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RedactionRuleFields {
            label: String,
            literal: String,
        }

        let fields = RedactionRuleFields::deserialize(deserializer)?;
        RedactionRule::new(fields.label, fields.literal).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RedactionSet {
    rules: Vec<RedactionRule>,
}

impl RedactionSet {
    pub fn new(mut rules: Vec<RedactionRule>) -> Result<Self> {
        let mut labels = BTreeSet::new();
        for rule in &rules {
            if !labels.insert(rule.label.clone()) {
                return Err(VogonError::DuplicateRedactionLabel(rule.label.clone()));
            }
        }

        rules.sort_by_key(|rule| Reverse(rule.literal.len()));

        Ok(Self { rules })
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

impl<'de> Deserialize<'de> for RedactionSet {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RedactionSetFields {
            rules: Vec<RedactionRule>,
        }

        let fields = RedactionSetFields::deserialize(deserializer)?;
        RedactionSet::new(fields.rules).map_err(de::Error::custom)
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
    fn redaction_rule_deserialization_validates_input() {
        let result = serde_json::from_str::<RedactionRule>(
            r#"{
                "label": "api key",
                "literal": "sk-test-123"
            }"#,
        );

        assert_eq!(
            result.unwrap_err().to_string(),
            VogonError::InvalidRedactionLabel("api key".to_owned()).to_string()
        );
    }

    #[test]
    fn redaction_set_replaces_known_literals() {
        let redactions =
            RedactionSet::new(vec![RedactionRule::new("api_key", "sk-test-123").unwrap()]).unwrap();

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
        ])
        .unwrap();

        assert_eq!(
            redactions.redact("token=sk-test-123"),
            "token=[REDACTED:api_key]"
        );
    }

    #[test]
    fn redaction_set_rejects_duplicate_labels() {
        let result = RedactionSet::new(vec![
            RedactionRule::new("api_key", "sk-test-123").unwrap(),
            RedactionRule::new("api_key", "sk-live-456").unwrap(),
        ]);

        assert_eq!(
            result.unwrap_err(),
            VogonError::DuplicateRedactionLabel("api_key".to_owned())
        );
    }

    #[test]
    fn redaction_set_deserialization_rejects_duplicate_labels() {
        let result = serde_json::from_str::<RedactionSet>(
            r#"{
                "rules": [
                    { "label": "api_key", "literal": "sk-test-123" },
                    { "label": "api_key", "literal": "sk-live-456" }
                ]
            }"#,
        );

        assert_eq!(
            result.unwrap_err().to_string(),
            VogonError::DuplicateRedactionLabel("api_key".to_owned()).to_string()
        );
    }
}
