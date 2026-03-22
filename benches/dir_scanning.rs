use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use duped::{
    IgnoreStrategy, JwalkStrategy, ScanStrategy, StdFsRecursive, WalkDirStrategy,
    test_utils::TestStructure,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct BenchmarkBaseline {
    strategy_name: String,
    mean_ns: f64,
    hardware_ref_ns: f64,
    normalized_score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct BaselineData {
    baselines: Vec<BenchmarkBaseline>,
    timestamp: u64,
}

fn get_baseline_path() -> PathBuf {
    std::env::temp_dir().join("dir_scan_baseline.json")
}

fn hardware_reference_benchmark() -> Duration {
    let start = std::time::Instant::now();
    let mut sum: u64 = 0;
    for _ in 0..100 {
        for i in 0..10000u64 {
            sum = sum.wrapping_add(i.wrapping_mul(i));
        }
    }
    black_box(sum);
    start.elapsed()
}

fn save_baseline(strategy_name: &str, mean_ns: f64) {
    let hw_ref = hardware_reference_benchmark().as_nanos() as f64;
    let normalized = mean_ns / hw_ref;

    let baseline = BenchmarkBaseline {
        strategy_name: strategy_name.to_string(),
        mean_ns,
        hardware_ref_ns: hw_ref,
        normalized_score: normalized,
    };

    let path = get_baseline_path();
    let mut data = if path.exists() {
        let json = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str::<BaselineData>(&json).unwrap_or_else(|_| BaselineData {
            baselines: Vec::new(),
            timestamp: 0,
        })
    } else {
        BaselineData {
            baselines: Vec::new(),
            timestamp: 0,
        }
    };

    // Update or add baseline
    if let Some(existing) = data
        .baselines
        .iter_mut()
        .find(|b| b.strategy_name == strategy_name)
    {
        *existing = baseline;
    } else {
        data.baselines.push(baseline);
    }

    data.timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    fs::write(&path, serde_json::to_string_pretty(&data).unwrap()).ok();
}

fn check_regression(strategy_name: &str, current_mean_ns: f64) -> Option<f64> {
    let path = get_baseline_path();
    if !path.exists() {
        return None;
    }

    let json = fs::read_to_string(&path).ok()?;
    let data: BaselineData = serde_json::from_str(&json).ok()?;

    let baseline = data
        .baselines
        .iter()
        .find(|b| b.strategy_name == strategy_name)?;

    // Normalize current measurement
    let hw_ref = hardware_reference_benchmark().as_nanos() as f64;
    let current_normalized = current_mean_ns / hw_ref;

    // Calculate regression ratio
    let ratio = current_normalized / baseline.normalized_score;
    Some(ratio)
}

fn benchmark_strategies(c: &mut Criterion) {
    // Create test structure: 1000 files, 4 levels deep
    let test_structure =
        TestStructure::create(1000, 4, 50).expect("Failed to create test structure");
    let root = test_structure.root_path();

    println!(
        "📁 Test structure: {} files in {:?}",
        test_structure.files_created, root
    );

    let mut group = c.benchmark_group("directory_scanning");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function(BenchmarkId::new("std_fs", "recursive"), |b| {
        let strategy = StdFsRecursive;
        b.iter(|| {
            for p in strategy.scan(black_box(root)) {
                black_box(p);
            }
        });
    });

    group.bench_function(BenchmarkId::new("walkdir", "standard"), |b| {
        let strategy = WalkDirStrategy;
        b.iter(|| {
            for p in strategy.scan(black_box(root)) {
                black_box(p);
            }
        });
    });

    group.bench_function(BenchmarkId::new("ignore", "standard"), |b| {
        let strategy = IgnoreStrategy;
        b.iter(|| {
            for p in strategy.scan(black_box(root)) {
                black_box(p);
            }
        });
    });

    group.bench_function(BenchmarkId::new("jwalk", "standard"), |b| {
        let strategy = JwalkStrategy;
        b.iter(|| {
            for p in strategy.scan(black_box(root)) {
                black_box(p);
            }
        });
    });

    group.finish();
}

fn regression_check(c: &mut Criterion) {
    let test_structure = TestStructure::create(1000, 4, 50).unwrap();
    let root = test_structure.root_path();

    let strategy = IgnoreStrategy;
    let strategy_name = "ignore_regression";

    let mut group = c.benchmark_group("regression_check");

    println!("🔍 Running regression check for '{}'", strategy_name);

    group.bench_function(strategy_name, |b| {
        b.iter(|| {
            for p in strategy.scan(black_box(root)) {
                black_box(p);
            }
        });
    });

    group.finish();

    println!("✅ Criterion will compare against previous runs automatically");
    println!("   Baseline directory: target/criterion/");
}

criterion_group!(benches, benchmark_strategies, regression_check);
criterion_main!(benches);
