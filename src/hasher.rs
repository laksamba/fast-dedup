use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use rayon::prelude::*;
use tracing::warn;

use crate::errors::Result;

/// Represents a validated duplicate group containing paths that share the exact same hash.
#[derive(Debug,serde::Serialize)]
pub struct DuplicateGroup {
    pub file_size: u64,
    pub file_hashes: String,
    pub paths: Vec<PathBuf>,
}

/// Runs the multi-tier hashing pipeline on the size-grouped candidates.
pub fn find_duplicates(size_map: std::collections::HashMap<u64, Vec<PathBuf>>) -> Result<Vec<DuplicateGroup>> {
    let mut confirmed_duplicates = Vec::new();

    for (size, paths) in size_map {
        // --- TIER 2: Partial Hashing (First 4KB) ---
        let mut partial_hash_map: std::collections::HashMap<[u8; 32], Vec<PathBuf>> = std::collections::HashMap::new();

        for path in paths {
            match partial_hash_file(&path, 4096) {
                Ok(hash) => {
                    partial_hash_map.entry(hash).or_default().push(path);
                }
                Err(e) => {
                    warn!(path = ?path, error = %e, "Failed to read file for partial hash, skipping");
                }
            }
        }

        // Prune partial hash groups with only 1 file
        partial_hash_map.retain(|_, group_paths| group_paths.len() > 1);

        // --- TIER 3: Full Parallel Hashing (BLAKE3 + Rayon) ---
        for (_, candidate_paths) in partial_hash_map {
            // Compute full file hashes in parallel across CPU cores using Rayon
            let hashed_pairs: Vec<(PathBuf, String)> = candidate_paths
                .into_par_iter()
                .filter_map(|path| {
                    match full_hash_file(&path) {
                        Ok(hash_str) => Some((path, hash_str)),
                        Err(e) => {
                            warn!(path = ?path, error = %e, "Failed to compute full hash, skipping");
                            None
                        }
                    }
                })
                .collect();

            // Group files by their final full cryptographic hash
            let mut full_hash_map: std::collections::HashMap<String, Vec<PathBuf>> = std::collections::HashMap::new();
            for (path, hash_str) in hashed_pairs {
                full_hash_map.entry(hash_str).or_default().push(path);
            }

            // Keep only sets that still have 2 or more files after full verification
            for (hash_str, final_paths) in full_hash_map {
                if final_paths.len() > 1 {
                    confirmed_duplicates.push(DuplicateGroup {
                        file_size: size,
                        file_hashes: hash_str,
                        paths: final_paths,
                    });
                }
            }
        }
    }

    Ok(confirmed_duplicates)
}

/// Computes a blake3 hash of the first `limit` bytes of a file.
fn partial_hash_file(path: &PathBuf, limit: usize) -> Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; limit];
    let n = file.read(&mut buffer)?;
    
    let mut hasher = blake3::Hasher::new();
    hasher.update(&buffer[..n]);
    Ok(*hasher.finalize().as_bytes())
}

/// Computes a full blake3 hash of an entire file using buffered chunking.
fn full_hash_file(path: &PathBuf) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65536]; // 64KB buffer for efficient disk I/O

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}