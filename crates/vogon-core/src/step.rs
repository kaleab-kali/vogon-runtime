use serde::{Deserialize, Serialize};

use crate::{Result, VogonError};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StepId(String);

impl StepId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(VogonError::EmptyStepId);
        }

        if !trimmed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        {
            return Err(VogonError::InvalidStepId(trimmed.to_owned()));
        }

        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub id: StepId,
    pub prompt: String,
}

impl Step {
    pub fn new(id: StepId, prompt: impl Into<String>) -> Self {
        Self {
            id,
            prompt: prompt.into(),
        }
    }

    pub fn id(&self) -> &StepId {
        &self.id
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }
}
