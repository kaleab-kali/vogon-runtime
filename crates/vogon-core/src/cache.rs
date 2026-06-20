use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

/// Default maximum number of outputs retained by [`RunCache`].
pub const DEFAULT_RUN_CACHE_MAX_ENTRIES: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// In-memory cache keyed by stable step input hashes.
pub struct RunCache {
    outputs: BTreeMap<String, String>,
    #[serde(default)]
    insertion_order: VecDeque<String>,
    #[serde(default = "RunCache::default_max_entries")]
    max_entries: usize,
}

impl RunCache {
    /// Creates an empty run cache with the default entry limit.
    pub fn new() -> Self {
        Self::with_max_entries(DEFAULT_RUN_CACHE_MAX_ENTRIES)
    }

    /// Creates an empty run cache with an explicit entry limit.
    ///
    /// A limit of `0` disables storage while preserving lookup behavior.
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            outputs: BTreeMap::new(),
            insertion_order: VecDeque::new(),
            max_entries,
        }
    }

    /// Returns the cached output for an input hash.
    pub fn get_output(&self, input_hash: &str) -> Option<&str> {
        self.outputs.get(input_hash).map(String::as_str)
    }

    /// Stores an output for an input hash.
    pub fn insert_output(&mut self, input_hash: impl Into<String>, output: impl Into<String>) {
        if self.max_entries == 0 {
            return;
        }

        let input_hash = input_hash.into();
        self.ensure_order_integrity();
        self.forget_ordered_key(&input_hash);
        self.insertion_order.push_back(input_hash.clone());
        self.outputs.insert(input_hash, output.into());
        self.evict_excess_outputs();
    }

    /// Removes one cached output by input hash.
    pub fn remove_output(&mut self, input_hash: &str) -> Option<String> {
        self.ensure_order_integrity();
        self.forget_ordered_key(input_hash);
        self.outputs.remove(input_hash)
    }

    /// Clears all cached outputs.
    pub fn clear(&mut self) {
        self.outputs.clear();
        self.insertion_order.clear();
    }

    /// Returns the number of cached outputs.
    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    /// Returns true when the cache does not contain any outputs.
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    /// Returns the maximum number of outputs retained by this cache.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    fn evict_excess_outputs(&mut self) {
        while self.outputs.len() > self.max_entries {
            let Some(input_hash) = self.insertion_order.pop_front() else {
                break;
            };
            self.outputs.remove(&input_hash);
        }
    }

    fn forget_ordered_key(&mut self, input_hash: &str) {
        if let Some(index) = self
            .insertion_order
            .iter()
            .position(|key| key == input_hash)
        {
            self.insertion_order.remove(index);
        }
    }

    fn ensure_order_integrity(&mut self) {
        let order_matches_outputs = self.insertion_order.len() == self.outputs.len()
            && self
                .insertion_order
                .iter()
                .all(|input_hash| self.outputs.contains_key(input_hash));

        if !order_matches_outputs {
            self.insertion_order = self.outputs.keys().cloned().collect();
        }
    }

    fn default_max_entries() -> usize {
        DEFAULT_RUN_CACHE_MAX_ENTRIES
    }
}

impl Default for RunCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{DEFAULT_RUN_CACHE_MAX_ENTRIES, RunCache};

    #[test]
    fn cache_stores_outputs_by_input_hash() {
        let mut cache = RunCache::new();

        cache.insert_output("abc123", "cached output");

        assert_eq!(cache.get_output("abc123"), Some("cached output"));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cache_uses_default_entry_limit() {
        let cache = RunCache::new();

        assert_eq!(cache.max_entries(), DEFAULT_RUN_CACHE_MAX_ENTRIES);
    }

    #[test]
    fn cache_evicts_oldest_outputs_at_entry_limit() {
        let mut cache = RunCache::with_max_entries(2);

        cache.insert_output("first", "first output");
        cache.insert_output("second", "second output");
        cache.insert_output("third", "third output");

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get_output("first"), None);
        assert_eq!(cache.get_output("second"), Some("second output"));
        assert_eq!(cache.get_output("third"), Some("third output"));
    }

    #[test]
    fn cache_updates_existing_outputs_without_growing() {
        let mut cache = RunCache::with_max_entries(2);

        cache.insert_output("first", "old output");
        cache.insert_output("second", "second output");
        cache.insert_output("first", "new output");

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get_output("first"), Some("new output"));
    }

    #[test]
    fn cache_can_disable_storage() {
        let mut cache = RunCache::with_max_entries(0);

        cache.insert_output("first", "first output");

        assert!(cache.is_empty());
        assert_eq!(cache.get_output("first"), None);
    }

    #[test]
    fn cache_can_remove_outputs() {
        let mut cache = RunCache::new();
        cache.insert_output("first", "first output");

        assert_eq!(
            cache.remove_output("first"),
            Some("first output".to_owned())
        );
        assert!(cache.is_empty());
    }

    #[test]
    fn cache_can_clear_outputs() {
        let mut cache = RunCache::new();
        cache.insert_output("first", "first output");
        cache.insert_output("second", "second output");

        cache.clear();

        assert!(cache.is_empty());
    }

    #[test]
    fn cache_deserialization_defaults_missing_bounds() {
        let mut cache: RunCache =
            serde_json::from_str(r#"{"outputs":{"first":"first output"}}"#).unwrap();

        cache.insert_output("second", "second output");

        assert_eq!(cache.max_entries(), DEFAULT_RUN_CACHE_MAX_ENTRIES);
        assert_eq!(cache.get_output("first"), Some("first output"));
        assert_eq!(cache.get_output("second"), Some("second output"));
    }
}
