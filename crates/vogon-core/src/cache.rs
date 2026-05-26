use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCache {
    outputs: BTreeMap<String, String>,
}

impl RunCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_output(&self, input_hash: &str) -> Option<&str> {
        self.outputs.get(input_hash).map(String::as_str)
    }

    pub fn insert_output(&mut self, input_hash: impl Into<String>, output: impl Into<String>) {
        self.outputs.insert(input_hash.into(), output.into());
    }

    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::RunCache;

    #[test]
    fn cache_stores_outputs_by_input_hash() {
        let mut cache = RunCache::new();

        cache.insert_output("abc123", "cached output");

        assert_eq!(cache.get_output("abc123"), Some("cached output"));
        assert_eq!(cache.len(), 1);
    }
}
