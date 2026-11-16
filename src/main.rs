mod errors;
mod scanner;
mod hasher;
mod actions;

use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use errors::Result;

/// A high-performance file deduplicator CLI built in Rust
#[derive(Parser, Debug)]
#[command(name = "fast-dedup", author, version, about, long_about = None)]
pub struct Cli {
    /// Target directory to scan for duplicates
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Preview duplicates without deleting or modifying any files
    #[arg(short, long)]
    pub dry_run: bool,

    /// Prompt interactively before deleting or replacing duplicate files
    #[arg(short, long)]
    pub interactive: bool,

    /// Output duplicate scan results in JSON format
    #[arg(short, long)]
    pub json: bool,
}

fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Cli::parse();

    info!(target_dir = ?args.path, "Starting fast-dedup scan workflow");

    println!("Scanning directory: {:?}\n", args.path);

    // Step 1: Walk directory tree and group files by size
    let size_map = scanner::group_files_by_size(&args.path)?;
    println!("Found {} file size groups with potential duplicates.", size_map.len());

    // Step 2: Run the partial + parallel full hashing pipeline
    let duplicate_groups = hasher::find_duplicates(size_map)?;

    // Step 3: Execute actions (report, dry-run, interactive delete/symlink)
    actions::handle_duplicates(duplicate_groups, args.dry_run, args.interactive, args.json)?;

    Ok(())
}