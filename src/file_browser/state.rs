//! File browser tree state management utilities
//!
//! This module provides utilities for managing the flattened tree structure
//! and efficient binary search operations on expanded directory IDs.

use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Placeholder for `ProjectEntryId` until project crate is integrated
/// In actual implementation, this will be `project::ProjectEntryId`
pub type ProjectEntryId = u64;

/// Placeholder entry struct for tree building
/// In actual implementation, this will be `project::GitEntry`
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: ProjectEntryId,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
}

/// Build a flattened tree from the local filesystem
///
/// This function traverses the directory tree depth-first and returns all entries
/// (both files and directories) with stable IDs derived from their relative paths.
///
/// # Arguments
/// * `root` - Absolute path of the workspace root
///
/// # Returns
/// Vec of entries in depth-first order
#[allow(clippy::missing_const_for_fn)] // Returns Vec, which is not const-compatible
pub fn build_entries_from_fs(root: &Path) -> Vec<Entry> {
    let mut entries = Vec::new();
    walk_dir(root, Path::new(""), 0, &mut entries);
    entries
}

fn walk_dir(root: &Path, rel_path: &Path, depth: usize, entries: &mut Vec<Entry>) {
    let abs_path = if rel_path.as_os_str().is_empty() {
        root.to_path_buf()
    } else {
        root.join(rel_path)
    };

    let read_dir = match std::fs::read_dir(&abs_path) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            tracing::warn!(
                "Failed to read directory {}: {}",
                abs_path.display(),
                error
            );
            return;
        }
    };

    let mut children: Vec<(PathBuf, bool)> = Vec::new();
    for entry in read_dir {
        match entry {
            Ok(entry) => {
                let file_type = entry.file_type().ok();
                let is_dir = file_type.is_some_and(|file_type| file_type.is_dir());
                let name = PathBuf::from(entry.file_name());
                children.push((name, is_dir));
            }
            Err(error) => {
                tracing::warn!(
                    "Failed to read directory entry under {}: {}",
                    abs_path.display(),
                    error
                );
            }
        }
    }

    children.sort_by(|a, b| {
        match (a.1, b.1) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.0.to_string_lossy().cmp(&b.0.to_string_lossy()),
        }
    });

    for (child_name, is_dir) in children {
        let child_rel_path = if rel_path.as_os_str().is_empty() {
            child_name
        } else {
            rel_path.join(child_name)
        };

        entries.push(Entry {
            id: entry_id_for_path(&child_rel_path),
            path: child_rel_path.clone(),
            is_dir,
            depth,
        });

        if is_dir {
            walk_dir(root, &child_rel_path, depth + 1, entries);
        }
    }
}

fn entry_id_for_path(path: &Path) -> ProjectEntryId {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

/// Check if an entry ID is in the sorted `expanded_dir_ids` vector
///
/// Uses binary search for O(log n) performance.
///
/// # Arguments
/// * `entry_id` - The entry ID to check
/// * `expanded_dir_ids` - Sorted slice of expanded directory IDs
///
/// # Returns
/// `true` if the entry is expanded, `false` otherwise
#[must_use]
pub fn is_expanded(entry_id: u64, expanded_dir_ids: &[u64]) -> bool {
    expanded_dir_ids.binary_search(&entry_id).is_ok()
}

/// Insert `entry_id` into sorted Vec, maintaining sort order
///
/// Uses binary search to find the correct insertion position. If the entry
/// is already present, this is a no-op.
///
/// # Arguments
/// * `entry_id` - The entry ID to insert
/// * `expanded_dir_ids` - Mutable sorted Vec of expanded directory IDs
pub fn expand_dir(entry_id: u64, expanded_dir_ids: &mut Vec<u64>) {
    if let Err(ix) = expanded_dir_ids.binary_search(&entry_id) {
        expanded_dir_ids.insert(ix, entry_id);
    }
}

/// Remove `entry_id` from sorted Vec
///
/// Uses binary search to find the entry. If the entry is not found,
/// this is a no-op.
///
/// # Arguments
/// * `entry_id` - The entry ID to remove
/// * `expanded_dir_ids` - Mutable sorted Vec of expanded directory IDs
pub fn collapse_dir(entry_id: u64, expanded_dir_ids: &mut Vec<u64>) {
    if let Ok(ix) = expanded_dir_ids.binary_search(&entry_id) {
        expanded_dir_ids.remove(ix);
    }
}

/// Calculate the depth of an entry for rendering indentation
///
/// Depth is calculated based on the number of path components relative
/// to the workspace root.
///
/// # Arguments
/// * `path` - The entry's path relative to workspace root
///
/// # Returns
/// Depth as usize (0 for root-level entries)
#[must_use]
pub fn calculate_depth(path: &Path) -> usize {
    path.components().count().saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_expanded_present() {
        let expanded = vec![1, 3, 5, 7, 9];
        assert!(is_expanded(1, &expanded));
        assert!(is_expanded(5, &expanded));
        assert!(is_expanded(9, &expanded));
    }

    #[test]
    fn is_expanded_absent() {
        let expanded = vec![1, 3, 5, 7, 9];
        assert!(!is_expanded(0, &expanded));
        assert!(!is_expanded(2, &expanded));
        assert!(!is_expanded(4, &expanded));
        assert!(!is_expanded(6, &expanded));
        assert!(!is_expanded(8, &expanded));
        assert!(!is_expanded(10, &expanded));
    }

    #[test]
    fn is_expanded_empty() {
        let expanded: Vec<u64> = vec![];
        assert!(!is_expanded(1, &expanded));
    }

    #[test]
    fn expand_dir_insert_middle() {
        let mut expanded = vec![1, 3, 5, 7];
        expand_dir(4, &mut expanded);
        assert_eq!(expanded, vec![1, 3, 4, 5, 7]);
    }

    #[test]
    fn expand_dir_insert_beginning() {
        let mut expanded = vec![3, 5, 7];
        expand_dir(1, &mut expanded);
        assert_eq!(expanded, vec![1, 3, 5, 7]);
    }

    #[test]
    fn expand_dir_insert_end() {
        let mut expanded = vec![1, 3, 5];
        expand_dir(7, &mut expanded);
        assert_eq!(expanded, vec![1, 3, 5, 7]);
    }

    #[test]
    fn expand_dir_duplicate() {
        let mut expanded = vec![1, 3, 5];
        expand_dir(3, &mut expanded);
        assert_eq!(expanded, vec![1, 3, 5]);
    }

    #[test]
    fn expand_dir_empty() {
        let mut expanded: Vec<u64> = vec![];
        expand_dir(1, &mut expanded);
        assert_eq!(expanded, vec![1]);
    }

    #[test]
    fn collapse_dir_remove_middle() {
        let mut expanded = vec![1, 3, 5, 7];
        collapse_dir(5, &mut expanded);
        assert_eq!(expanded, vec![1, 3, 7]);
    }

    #[test]
    fn collapse_dir_remove_beginning() {
        let mut expanded = vec![1, 3, 5, 7];
        collapse_dir(1, &mut expanded);
        assert_eq!(expanded, vec![3, 5, 7]);
    }

    #[test]
    fn collapse_dir_remove_end() {
        let mut expanded = vec![1, 3, 5, 7];
        collapse_dir(7, &mut expanded);
        assert_eq!(expanded, vec![1, 3, 5]);
    }

    #[test]
    fn collapse_dir_not_found() {
        let mut expanded = vec![1, 3, 5, 7];
        collapse_dir(4, &mut expanded);
        assert_eq!(expanded, vec![1, 3, 5, 7]);
    }

    #[test]
    fn collapse_dir_empty() {
        let mut expanded: Vec<u64> = vec![];
        collapse_dir(1, &mut expanded);
        assert!(expanded.is_empty());
    }

    #[test]
    fn expand_collapse_roundtrip() {
        let mut expanded = vec![1, 5, 9];

        expand_dir(3, &mut expanded);
        expand_dir(7, &mut expanded);
        assert_eq!(expanded, vec![1, 3, 5, 7, 9]);

        collapse_dir(3, &mut expanded);
        collapse_dir(7, &mut expanded);
        assert_eq!(expanded, vec![1, 5, 9]);
    }

    #[test]
    fn expand_maintains_sort_order() {
        let mut expanded = vec![10, 30, 50];

        expand_dir(40, &mut expanded);
        expand_dir(20, &mut expanded);
        expand_dir(60, &mut expanded);
        expand_dir(5, &mut expanded);

        assert_eq!(expanded, vec![5, 10, 20, 30, 40, 50, 60]);
    }

    #[test]
    fn calculate_depth_root_level() {
        let path = PathBuf::from("README.md");
        assert_eq!(calculate_depth(&path), 0);
    }

    #[test]
    fn calculate_depth_one_level() {
        let path = PathBuf::from("src/main.rs");
        assert_eq!(calculate_depth(&path), 1);
    }

    #[test]
    fn calculate_depth_two_levels() {
        let path = PathBuf::from("src/file_browser/state.rs");
        assert_eq!(calculate_depth(&path), 2);
    }

    #[test]
    fn calculate_depth_deep_nesting() {
        let path = PathBuf::from("a/b/c/d/e/f/file.txt");
        assert_eq!(calculate_depth(&path), 6);
    }

    #[test]
    fn build_entries_from_fs_orders_depth_first() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let root = temp_dir.path();

        std::fs::create_dir(root.join("src")).expect("create src dir");
        std::fs::write(root.join("src/main.rs"), "fn main() {}").expect("write main");
        std::fs::write(root.join("b.txt"), "b").expect("write b");

        let entries = build_entries_from_fs(root);
        let paths: Vec<String> = entries
            .iter()
            .map(|entry| entry.path.to_string_lossy().to_string())
            .collect();

        assert_eq!(paths, vec!["src", "src/main.rs", "b.txt"]);
        assert_eq!(entries[0].depth, 0);
        assert_eq!(entries[1].depth, 1);
        assert_eq!(entries[2].depth, 0);
        assert!(entries[0].is_dir);
        assert!(!entries[2].is_dir);
    }

    #[test]
    fn binary_search_performance() {
        let mut expanded: Vec<u64> = (0..10000).step_by(2).collect();

        assert!(is_expanded(5000, &expanded));
        assert!(!is_expanded(5001, &expanded));

        expand_dir(5001, &mut expanded);
        assert!(is_expanded(5001, &expanded));

        collapse_dir(5001, &mut expanded);
        assert!(!is_expanded(5001, &expanded));
    }
}
