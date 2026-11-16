use std::fs;
use std::io::{self, Write};
use tracing::info;

use crate::errors::Result;
use crate::hasher::DuplicateGroup;

pub fn handle_duplicates(
    groups: Vec<DuplicateGroup>,
    dry_run: bool,
    interactive: bool,
    json: bool,
) -> Result<()> {
    if groups.is_empty() {
        println!("No duplicate files found.");
        return Ok(());
    }

    if json {
        let json_output = serde_json::to_string_pretty(&groups).unwrap_or_default();
        println!("{}", json_output);
        return Ok(());
    }

    let mut total_wasted_bytes: u64 = 0;

    for (index, group) in groups.iter().enumerate() {
        println!(
            "\n--- DUPLICATE SET #{} (Size: {} bytes, Hash: {}) ---",
            index + 1,
            group.file_size,
            group.file_hashes
        );

        // Treat the first path as the original, and the rest as duplicates
        let original = &group.paths[0];
        let duplicates = &group.paths[1..];

        println!("  [KEEP ORIGINAL]: {:?}", original);
        for dup in duplicates {
            println!("  [DUPLICATE]   : {:?}", dup);
            total_wasted_bytes += group.file_size;
        }

        if !dry_run && interactive {
            print!("\nDo you want to delete or symlink these duplicates? [d = delete, s = symlink, n = skip]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim().to_lowercase();

            match choice.as_str() {
                "d" => {
                    for dup in duplicates {
                        fs::remove_file(dup)?;
                        info!(path = ?dup, "Deleted duplicate file");
                        println!("Deleted: {:?}", dup);
                    }
                }
                "s" => {
                    for dup in duplicates {
                        fs::remove_file(dup)?;
                        
                        // Cross-platform symlink handling
                        #[cfg(unix)]
                        {
                            std::os::unix::fs::symlink(original, dup)?;
                        }
                        #[cfg(windows)]
                        {
                            std::os::windows::fs::symlink_file(original, dup)?;
                        }

                        info!(original = ?original, symlink = ?dup, "Replaced duplicate with symlink");
                        println!("Replaced with symlink: {:?}", dup);
                    }

                }

                "q" | "quit" | "abort" => {
                    println!("Operation aborted by user.");
                    return Err(crate::errors::DedupError::Aborted);
                }
                _ => {
                    println!("Skipped this set.");
                }
            }
        }
    }

    println!("\n========================================");
    println!(
        "Scan complete. Total potential space wasted by duplicates: {} MB",
        total_wasted_bytes / (1024 * 1024)
    );
    if dry_run {
        println!("Mode: Dry-run active. No files were modified.");
    }

    Ok(())
}