# fast-dedup 🚀

A lightning-fast, production-grade command-line utility written in Rust to scan directory trees, detect exact duplicate files using a multi-tier filtering pipeline, and safely clean them up via deletion or symbolic links.

---

## 🛠️ Engineering Architecture & Performance

`fast-dedup` avoids naive, resource-heavy file comparisons. Instead, it utilizes a strict **3-tier filtering pipeline** to ensure maximum CPU and disk efficiency:

1. **Tier 1 - Size Grouping (Metadata Only):** Walks directory trees while respecting `.gitignore`, grouping files by exact byte size. Uniquely sized files are instantly discarded without reading data from disk.
2. **Tier 2 - Partial Hashing (4KB Quick Peek):** Reads only the first 4KB of candidate files sharing identical sizes and performs a fast header check to eliminate false matches early.
3. **Tier 3 - Parallel Full Hashing (BLAKE3 + Rayon):** Distributes cryptographic full-file `blake3` hashing across all logical CPU cores in parallel, utilizing fixed 64KB chunks to keep memory usage completely flat regardless of file size.

---

## 📦 Tech Stack

- **[Rust](https://www.rust-lang.org/)** — Memory safety with zero garbage collection overhead.
- **[Clap](https://crates.io/crates/clap)** — Robust, type-safe CLI argument parsing.
- **[Ignore](https://crates.io/crates/ignore)** — Blazing-fast directory traversal with automatic `.gitignore` support.
- **[Blake3](https://crates.io/crates/blake3)** — State-of-the-art cryptographic hashing utilizing SIMD instructions.
- **[Rayon](https://crates.io/crates/rayon)** — Data-parallelism library for multi-threaded worker pools.
- **[Thiserror](https://crates.io/crates/thiserror)** — Clean, strongly-typed error handling.
- **[Tracing](https://crates.io/crates/tracing)** — Structured diagnostic logging.

---

## 🚀 Getting Started

### Prerequisites

Make sure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed (Edition 2021+).

### Installation & Build

Clone the repository and build an optimized release binary:

```bash
git clone [https://github.com/your-username/fast-dedup.git](https://github.com/laksamba/fast-dedup.git)
cd fast-dedup
cargo build --release
The compiled binary will be located at ./target/release/fast-dedup (or ./target/release/fast-dedup.exe on Windows).📖 Usage & Examples1. Run a Dry-Run Scan (Safest Option)Scans the target directory and previews potential duplicate sets and wasted space without modifying or deleting any files.Bash./target/release/fast-dedup --path "C:\Users\YourName\Documents" --dry-run
2. Run Interactive CleanupPrompts you interactively for each duplicate set, allowing you to delete duplicates (d), replace them with cross-platform symbolic links (s) to save 100% of the disk space, or skip (n).Bash./target/release/fast-dedup --path ./my_folder --interactive
3. Output Results as JSONExports structured duplicate data for automation or pipeline ingestion.Bash./target/release/fast-dedup --path ./my_folder --json
4. Enable Verbose Debug LoggingPowered by tracing, you can control log output levels via environment variables:Bash$env:RUST_LOG="debug"; ./target/release/fast-dedup --path ./my_folder --dry-run
⚙️ CLI Options ReferenceFlag / OptionShortDescriptionDefault--path <PATH>-pTarget directory path to scan for duplicates. (Current Directory)--dry-run-dPreview duplicates without modifying or deleting filesfalse--interactive-iPrompt interactively before deleting or symlinking duplicatesfalse--json-jOutput scan results in formatted JSONfalse--help-hPrints help documentation—--version-VPrints version information—
```
🛡️ License
This project is open-source and available under the MIT License.