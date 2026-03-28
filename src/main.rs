use clap::{Parser, ValueEnum};
use duped::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "duped")]
#[command(about = "Fast directory scanner with duplicate detection", long_about = None)]
struct Cli {
    /// Directory to scan
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Scanning strategy to use
    #[arg(short, long, value_enum, default_value_t = Strategy::Jwalk)]
    strategy: Strategy,

    /// Deduplication strategy to use
    #[arg(short, long, value_enum, default_value_t = DedupeStrategy::FullContent)]
    dedupe: DedupeStrategy,

    /// Show timing information
    #[arg(short, long)]
    time: bool,

    /// Suppress output (only show timing)
    #[arg(long)]
    silent: bool,

    /// Minimum file size in bytes (filters to show only duplicates of files >= this size)
    #[arg(long)]
    min_size: Option<u64>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Strategy {
    /// Standard library recursive (std::fs::read_dir)
    StdFs,
    /// Walkdir crate (most popular)
    Walkdir,
    /// Ignore crate (fastest, respects .gitignore)
    Ignore,
    /// Jwalk crate (concurrent)
    Jwalk,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum DedupeStrategy {
    /// Compare full file contents
    FullContent,
}

fn main() {
    let cli = Cli::parse();

    if !cli.path.exists() {
        eprintln!("Error: Path '{}' does not exist", cli.path.display());
        std::process::exit(1);
    }

    let start = std::time::Instant::now();

    if let Err(e) = find_duplicates(&cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    if cli.time {
        let elapsed = start.elapsed();
        eprintln!("⏱️  Completed in {:.2?}", elapsed);
    }
}

fn find_duplicates(cli: &Cli) -> std::io::Result<()> {
    let strategy: Box<dyn ScanStrategy> = match cli.strategy {
        Strategy::StdFs => Box::new(StdFsStrategy),
        Strategy::Walkdir => Box::new(WalkDirStrategy),
        Strategy::Ignore => Box::new(IgnoreStrategy),
        Strategy::Jwalk => Box::new(JwalkStrategy),
    };

    let dedupe_strategy: Box<dyn DeduplicateStrategy> = match cli.dedupe {
        DedupeStrategy::FullContent => Box::new(FullContentStrategy),
    };

    if !cli.silent {
        eprintln!("📁 Scanning: {}", cli.path.display());
        eprintln!("🔧 Scan strategy: {:?}", cli.strategy);
        eprintln!("🔍 Dedupe strategy: {:?}", cli.dedupe);
    }

    // Collect files from the scanner - we need this due to iterator lifetime constraints
    let files: Vec<_> = strategy.scan(&cli.path).collect();
    
    // Find duplicates - pass min_size for early filtering
    let duplicate_groups = dedupe_strategy.find_duplicates(files, cli.min_size)?;

    if !cli.silent && !duplicate_groups.is_empty() {
        let total_files: usize = duplicate_groups.iter().map(|g| g.file_count()).sum();
        eprintln!("Found {} files in {} duplicate groups", total_files, duplicate_groups.len());
    }

    // Calculate statistics
    let total_duplicate_files: usize = duplicate_groups.iter().map(|g| g.file_count()).sum();
    let total_wasted_bytes: u64 = duplicate_groups.iter().map(|g| {
        // Wasted space = (count - 1) * size_bytes (we keep one original)
        (g.file_count() as u64 - 1) * g.size_bytes
    }).sum();

    // Output as JSON
    let json = serde_json::json!({
        "duplicate_groups": duplicate_groups,
        "statistics": {
            "total_groups": duplicate_groups.len(),
            "total_duplicate_files": total_duplicate_files,
            "total_wasted_bytes": total_wasted_bytes,
        }
    });

    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}
