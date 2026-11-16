use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

use crate::errors::Result;

/// Scans the target directory recursively, respects .gitignore,
/// and groups files by their exact byte size.
pub fn group_files_by_size(root_path: &Path) -> Result<HashMap<u64, Vec<PathBuf>>> {
    if !root_path.exists() {
        return Err(crate::errors::DedupError::InvalidPath(format!(
            "Path does not exist: {:?}",
            root_path
        )));
    }

    let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    // WalkBuilder configures directory traversal
    let walker = WalkBuilder::new(root_path)
        .hidden(false) // Include hidden files
        .git_ignore(true) // Automatically respect .gitignore files
        .build();

    for entry in walker {
        match entry {
            Ok(ent) => {
                // We only care about standard files, skip directories and symlinks
                if ent.file_type().map_or(false, |ft| ft.is_file()) {
                    let path = ent.path().to_path_buf();

                    match std::fs::metadata(&path) {
                        Ok(metadata) => {
                            let size = metadata.len();
                            // Skip empty files (0 bytes) as deduplicating them provides no value
                            if size > 0 {
                                size_map.entry(size).or_default().push(path);
                            }
                        }
                        Err(e) => {
                            // Production resilience: log warning instead of crashing on permission denied
                            warn!(path = ?path, error = %e, "Failed to read file metadata, skipping");
                        }
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "Walkdir entry error encountered, skipping");
            }
        }
    }

    // Optimization: Immediately prune any size group with fewer than 2 files.
    // If a file size is unique across the entire scan tree, it has no duplicates.
    size_map.retain(|size, paths| {
        if paths.len() < 2 {
            debug!(file_size = size, "Discarding unique file size group");
            false
        } else {
            true
        }
    });

    Ok(size_map)
}
