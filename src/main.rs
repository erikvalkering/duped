use clap::{Parser, ValueEnum};
use duped::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "duped")]
#[command(about = "Fast directory scanner with multiple strategies", long_about = None)]
struct Cli {
    /// Directory to scan
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Scanning strategy to use
    #[arg(short, long, value_enum, default_value_t = Strategy::Ignore)]
    strategy: Strategy,

    /// Show file metadata (size, modified time)
    #[arg(short, long)]
    metadata: bool,

    /// Show timing information
    #[arg(short, long)]
    time: bool,

    /// Suppress output (only show timing)
    #[arg(long)]
    silent: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Strategy {
    /// Standard library recursive (std::fs::read_dir)
    StdFs,
    /// Walkdir crate (most popular)
    Walkdir,
    /// Walkdir with optimized settings
    WalkdirParallel,
    /// Ignore crate (fastest, respects .gitignore)
    Ignore,

    // Super-duper fast strategy
    Jwalk,
}

fn main() {
    let cli = Cli::parse();

    if !cli.path.exists() {
        eprintln!("Error: Path '{}' does not exist", cli.path.display());
        std::process::exit(1);
    }

    let start = std::time::Instant::now();

    if cli.metadata {
        scan_with_metadata(&cli);
    } else {
        scan_files(&cli);
    }

    if cli.time {
        let elapsed = start.elapsed();
        eprintln!(
            "
⏱️  Completed in {:.2?}",
            elapsed
        );
    }
}

fn scan_files(cli: &Cli) {
    let strategy: Box<dyn ScanStrategy> = match cli.strategy {
        Strategy::StdFs => Box::new(StdFsRecursive),
        Strategy::Walkdir => Box::new(WalkDirStrategy),
        Strategy::WalkdirParallel => Box::new(WalkDirParallel),
        Strategy::Ignore => Box::new(IgnoreStrategy),
        Strategy::Jwalk => Box::new(JwalkStrategy),
    };

    println!("📁 Scanning: {}", cli.path.display());
    println!("🔧 Strategy: {:?}", cli.strategy);

    match strategy.scan(&cli.path) {
        Ok(files) => {
            println!("Found {} files:", files.len());
            if cli.silent {
                return;
            }
            for file in &files {
                println!("  {}", file.display());
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn scan_with_metadata(cli: &Cli) {
    println!("📁 Scanning: {}", cli.path.display());
    println!("🔧 Strategy: with-metadata");

    let strategy = WithMetadata;

    match strategy.scan(&cli.path) {
        Ok(mut files) => {
            println!("Found {} files:", files.len());

            // Sort by size (largest first)
            files.sort_by(|a, b| b.size.cmp(&a.size));

            for file_info in &files {
                let size_kb = file_info.size as f64 / 1024.0;
                println!("  {:>10.2} KB  {}", size_kb, file_info.path.display());
            }

            let total_size: u64 = files.iter().map(|f| f.size).sum();
            let total_mb = total_size as f64 / (1024.0 * 1024.0);
            println!("📊 Total size: {:.2} MB", total_mb);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
