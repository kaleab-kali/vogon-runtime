use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
/// In-memory cache keyed by stable step input hashes.
pub struct RunCache {
    outputs: BTreeMap<String, String>,
}

impl RunCache {
    /// Creates an empty run cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached output for an input hash.
    pub fn get_output(&self, input_hash: &str) -> Option<&str> {
        self.outputs.get(input_hash).map(String::as_str)
    }

    /// Stores an output for an input hash.
    pub fn insert_output(&mut self, input_hash: impl Into<String>, output: impl Into<String>) {
        self.outputs.insert(input_hash.into(), output.into());
    }

    /// Returns the number of cached outputs.
    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    /// Returns true when the cache does not contain any outputs.
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
