use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{Result, VogonError};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
/// Validated workflow step identifier.
pub struct StepId(String);

impl StepId {
    /// Creates a step identifier from an ASCII slug.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(VogonError::EmptyStepId);
        }

        if value != trimmed {
            return Err(VogonError::InvalidStepId(value));
        }

        if !trimmed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        {
            return Err(VogonError::InvalidStepId(trimmed.to_owned()));
        }

        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the step identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for StepId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        StepId::new(value).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::StepId;

    #[test]
    fn step_id_deserialization_validates_input() {
        let result = serde_json::from_str::<StepId>(r#""bad id""#);

        assert!(result.is_err());
    }

    #[test]
    fn step_id_rejects_surrounding_whitespace() {
        let result = StepId::new(" classify ");

        assert!(result.is_err());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One prompt in an ordered workflow.
pub struct Step {
    /// Stable identifier for this step.
    pub id: StepId,
    /// Prompt sent to the model adapter for this step.
    pub prompt: String,
}

impl Step {
    /// Creates a step from a validated identifier and prompt text.
    pub fn new(id: StepId, prompt: impl Into<String>) -> Self {
        Self {
            id,
            prompt: prompt.into(),
        }
    }

    /// Returns this step's identifier.
    pub fn id(&self) -> &StepId {
        &self.id
    }

    /// Returns this step's prompt text.
    pub fn prompt(&self) -> &str {
        &self.prompt
    }
}
