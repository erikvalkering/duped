use clap::{Parser, ValueEnum, Subcommand};
use duped::*;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "duped")]
#[command(about = "Fast directory scanner with duplicate detection", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Directory to scan (used with default find command)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Scanning strategy to use (used with default find command)
    #[arg(short, long, value_enum, default_value_t = Strategy::Jwalk)]
    strategy: Strategy,

    /// Deduplication strategy to use (used with default find command)
    #[arg(short, long, value_enum, default_value_t = DedupeStrategy::FullContent)]
    dedupe: DedupeStrategy,

    /// Show timing information (used with default find command)
    #[arg(short, long)]
    time: bool,

    /// Suppress output (only show timing) (used with default find command)
    #[arg(long)]
    silent: bool,

    /// Minimum file size in bytes (filters to show only duplicates of files >= this size)
    #[arg(long)]
    min_size: Option<u64>,

    /// Number of bytes to compare for partial strategy (positive: first n bytes, negative: last n bytes)
    #[arg(long, default_value_t = 512)]
    partial_bytes: i64,
}

#[derive(Subcommand)]
enum Command {
    /// Benchmark scan performance
    Bench {
        /// Directory to scan
        path: PathBuf,

        /// Scanning strategy to benchmark
        #[arg(short, long, value_enum)]
        strategy: Strategy,

        /// Number of iterations to run
        #[arg(short, long, default_value_t = 5)]
        iterations: u32,

        /// Show detailed timing for each iteration
        #[arg(long)]
        verbose: bool,
    },
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

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
enum DedupeStrategy {
    /// Group by size only (fast, no content comparison)
    SizeOnly,
    /// Compare first/last n bytes of files
    /// Positive: first n bytes, Negative: last |n| bytes
    Partial,
    /// Compare full file contents (accurate, slower)
    FullContent,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Bench {
            path,
            strategy,
            iterations,
            verbose,
        }) => {
            if !path.exists() {
                eprintln!("Error: Path '{}' does not exist", path.display());
                std::process::exit(1);
            }

            if let Err(e) = benchmark_scan(&path, strategy, iterations, verbose) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            // Default: find duplicates
            if !cli.path.exists() {
                eprintln!("Error: Path '{}' does not exist", cli.path.display());
                std::process::exit(1);
            }

            let start = Instant::now();

            if let Err(e) = find_duplicates(
                &cli.path,
                cli.strategy,
                cli.dedupe,
                cli.time,
                cli.silent,
                cli.min_size,
                cli.partial_bytes,
            ) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }

            if cli.time {
                let elapsed = start.elapsed();
                eprintln!("⏱️  Completed in {:.2?}", elapsed);
            }
        }
    }
}

fn find_duplicates(
    path: &PathBuf,
    strategy: Strategy,
    dedupe: DedupeStrategy,
    _time: bool,
    silent: bool,
    min_size: Option<u64>,
    partial_bytes: i64,
) -> std::io::Result<()> {
    let scan_strategy: Box<dyn ScanStrategy> = match strategy {
        Strategy::StdFs => Box::new(StdFsStrategy),
        Strategy::Walkdir => Box::new(WalkDirStrategy),
        Strategy::Ignore => Box::new(IgnoreStrategy),
        Strategy::Jwalk => Box::new(JwalkStrategy),
    };

    let dedupe_strategy: Box<dyn DeduplicateStrategy> = match dedupe {
        DedupeStrategy::SizeOnly => Box::new(SizeOnlyStrategy),
        DedupeStrategy::Partial => Box::new(PartialContentStrategy::new(partial_bytes)),
        DedupeStrategy::FullContent => Box::new(FullContentStrategy),
    };

    if !silent {
        eprintln!("📁 Scanning: {}", path.display());
        eprintln!("🔧 Scan strategy: {:?}", strategy);
        eprintln!("🔍 Dedupe strategy: {:?}", dedupe);
    }

    // Collect files from the scanner - we need this due to iterator lifetime constraints
    let files: Vec<_> = scan_strategy.scan(&path).collect();
    
    // Find duplicates - pass min_size for early filtering
    let duplicate_groups = dedupe_strategy.find_duplicates(files, min_size)?;

    if !silent && !duplicate_groups.is_empty() {
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

fn benchmark_scan(
    path: &PathBuf,
    strategy: Strategy,
    iterations: u32,
    verbose: bool,
) -> std::io::Result<()> {
    let scan_strategy: Box<dyn ScanStrategy> = match strategy {
        Strategy::StdFs => Box::new(StdFsStrategy),
        Strategy::Walkdir => Box::new(WalkDirStrategy),
        Strategy::Ignore => Box::new(IgnoreStrategy),
        Strategy::Jwalk => Box::new(JwalkStrategy),
    };

    eprintln!("🏃 Benchmarking scan performance");
    eprintln!("📁 Directory: {}", path.display());
    eprintln!("🔧 Strategy: {:?}", strategy);
    eprintln!("🔁 Iterations: {}", iterations);
    eprintln!();

    let mut times = Vec::new();

    for i in 1..=iterations {
        let start = Instant::now();
        let file_count: usize = scan_strategy.scan(path).count();
        let elapsed = start.elapsed();
        
        times.push(elapsed);

        if verbose {
            eprintln!("  Iteration {}: {:?} ({} files)", i, elapsed, file_count);
        }
    }

    let total_duration: std::time::Duration = times.iter().sum();
    let min_duration = times.iter().min().copied().unwrap();
    let max_duration = times.iter().max().copied().unwrap();
    let avg_duration = total_duration / iterations;

    eprintln!("📊 Results:");
    eprintln!("  Min:     {:.2?}", min_duration);
    eprintln!("  Max:     {:.2?}", max_duration);
    eprintln!("  Average: {:.2?}", avg_duration);
    eprintln!("  Total:   {:.2?}", total_duration);

    Ok(())
}
