use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use duped::{
    IgnoreStrategy, JwalkStrategy, ScanStrategy, StdFsStrategy, WalkDirStrategy,
    test_utils::TestStructure,
};
use std::hint::black_box;
use std::time::Duration;

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

    let strategies: Vec<Box<dyn ScanStrategy>> = vec![
        Box::new(StdFsStrategy),
        Box::new(WalkDirStrategy),
        Box::new(IgnoreStrategy),
        Box::new(JwalkStrategy),
    ];

    for strategy in strategies.iter() {
        let id = BenchmarkId::new(strategy.name(), "standard");
        group.bench_function(id, |b| {
            let s = strategy.as_ref();
            b.iter(|| {
                for p in s.scan(black_box(root)) {
                    black_box(p);
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_strategies);
criterion_main!(benches);
