//! Word-by-word diff highlighting using `similar` crate
//!
//! Provides diff computation with color highlighting support:
//! - Green: Added words
//! - Red: Removed words
//! - Cached results to avoid recomputation

use similar::{ChangeTag, TextDiff};

/// Represents a single change in the diff
#[derive(Debug, Clone, PartialEq)]
pub enum DiffChange {
    /// Text that was removed (shown in red)
    Delete(String),
    /// Text that was added (shown in green)
    Insert(String),
    /// Text that remained the same
    Equal(String),
}

pub fn compute_diff(original: &str, corrected: &str) -> Vec<DiffChange> {
    let diff = TextDiff::from_words(original, corrected);
    let mut changes = Vec::new();

    for change in diff.iter_all_changes() {
        let text = change.value().to_string();

        match change.tag() {
            ChangeTag::Delete => changes.push(DiffChange::Delete(text)),
            ChangeTag::Insert => changes.push(DiffChange::Insert(text)),
            ChangeTag::Equal => changes.push(DiffChange::Equal(text)),
        }
    }

    changes
}

/// Cached diff result to avoid recomputation
#[derive(Debug, Clone)]
pub struct CachedDiff {
    original: String,
    corrected: String,
    changes: Vec<DiffChange>,
}

impl CachedDiff {
    /// Creates a new cached diff
    pub fn new(original: String, corrected: String) -> Self {
        let changes = compute_diff(&original, &corrected);
        Self {
            original,
            corrected,
            changes,
        }
    }

    /// Returns the cached changes, recomputing only if text changed
    pub fn get_or_update(&mut self, original: &str, corrected: &str) -> &[DiffChange] {
        if self.original != original || self.corrected != corrected {
            self.original = original.to_string();
            self.corrected = corrected.to_string();
            self.changes = compute_diff(original, corrected);
        }
        &self.changes
    }

    /// Returns the cached changes without updating
    pub fn changes(&self) -> &[DiffChange] {
        &self.changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_diff_no_changes() {
        let original = "Hello world";
        let corrected = "Hello world";
        let changes = compute_diff(original, corrected);

        assert!(!changes.is_empty());
        assert!(changes.iter().all(|c| matches!(c, DiffChange::Equal(_))));
    }

    #[test]
    fn test_compute_diff_insertion() {
        let original = "Hello world";
        let corrected = "Hello beautiful world";
        let changes = compute_diff(original, corrected);

        // Should have: Equal("Hello "), Insert("beautiful "), Equal("world")
        assert!(changes.iter().any(|c| matches!(c, DiffChange::Insert(_))));
    }

    #[test]
    fn test_compute_diff_deletion() {
        let original = "Hello beautiful world";
        let corrected = "Hello world";
        let changes = compute_diff(original, corrected);

        // Should have deletion
        assert!(changes.iter().any(|c| matches!(c, DiffChange::Delete(_))));
    }

    #[test]
    fn test_compute_diff_replacement() {
        let original = "Hello world";
        let corrected = "Hello universe";
        let changes = compute_diff(original, corrected);

        // Should have both delete and insert
        assert!(changes.iter().any(|c| matches!(c, DiffChange::Delete(_))));
        assert!(changes.iter().any(|c| matches!(c, DiffChange::Insert(_))));
    }

    #[test]
    fn test_cached_diff_new() {
        let original = "Hello world".to_string();
        let corrected = "Hello universe".to_string();
        let cached = CachedDiff::new(original.clone(), corrected.clone());

        assert_eq!(cached.original, original);
        assert_eq!(cached.corrected, corrected);
        assert!(!cached.changes.is_empty());
    }

    #[test]
    fn test_cached_diff_no_update() {
        let original = "Hello world".to_string();
        let corrected = "Hello universe".to_string();
        let mut cached = CachedDiff::new(original.clone(), corrected.clone());

        let changes1_len = cached.get_or_update(&original, &corrected).len();
        let changes2_len = cached.get_or_update(&original, &corrected).len();

        assert_eq!(changes1_len, changes2_len);
    }

    #[test]
    fn test_cached_diff_update_on_change() {
        let original1 = "Hello world".to_string();
        let corrected1 = "Hello universe".to_string();
        let mut cached = CachedDiff::new(original1.clone(), corrected1.clone());

        assert_eq!(cached.original, "Hello world");
        assert_eq!(cached.corrected, "Hello universe");

        let original2 = "Completely different text";
        let corrected2 = "Another different text";
        cached.get_or_update(original2, corrected2);

        assert_eq!(cached.original, original2);
        assert_eq!(cached.corrected, corrected2);
    }

    #[test]
    fn test_word_diff_polish_text() {
        let original = "Witam serdecznie wszystkich";
        let corrected = "Witam bardzo serdecznie wszystkich";
        let changes = compute_diff(original, corrected);

        // Should detect "bardzo" as insertion
        assert!(changes.iter().any(|c| matches!(c, DiffChange::Insert(_))));
    }

    #[test]
    fn test_diff_change_equality() {
        let change1 = DiffChange::Insert("test".to_string());
        let change2 = DiffChange::Insert("test".to_string());
        let change3 = DiffChange::Delete("test".to_string());

        assert_eq!(change1, change2);
        assert_ne!(change1, change3);
    }

    #[test]
    fn test_diff_demonstration() {
        let original = "Witam serdecznie wszystkich";
        let corrected = "Witam bardzo serdecznie wszystkich uzytkownikow";
        let changes = compute_diff(original, corrected);

        println!("\n=== DIFF DEMONSTRATION ===");
        println!("Original:  {}", original);
        println!("Corrected: {}", corrected);
        println!("\nChanges:");
        for (i, change) in changes.iter().enumerate() {
            match change {
                DiffChange::Delete(text) => println!("  [{}] DELETE: {:?}", i, text),
                DiffChange::Insert(text) => println!("  [{}] INSERT: {:?}", i, text),
                DiffChange::Equal(text) => println!("  [{}] EQUAL:  {:?}", i, text),
            }
        }

        assert!(changes.iter().any(|c| matches!(c, DiffChange::Insert(_))));
    }
}
