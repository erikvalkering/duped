#[cfg(test)]
mod tests {
    use duped::test_utils::TestStructure;
    use duped::*;
    use std::time::Instant;

    #[test]
    fn test_relative_performance_ranking() {
        // Create test structure
        let test_structure =
            TestStructure::create(500, 3, 30).expect("Failed to create test structure");
        let root = test_structure.root_path();

        println!("🔬 Testing {} files", test_structure.files_created);

        // Benchmark each strategy
        let strategies: Vec<(&str, Box<dyn ScanStrategy>)> = vec![
            ("std::fs", Box::new(StdFsRecursive)),
            ("walkdir", Box::new(WalkDirStrategy)),
            ("ignore", Box::new(IgnoreStrategy)),
            ("jwalk", Box::new(JwalkStrategy)),
        ];

        let mut results = Vec::new();

        for (name, strategy) in strategies {
            let start = Instant::now();
            let files = strategy.scan(root).expect("Scan failed");
            let duration = start.elapsed();

            println!("{:15} {:>8.2?}", name, duration);
            results.push((name, duration, files.len()));
        }

        // Assert: All strategies find the same number of files
        let expected_count = results[0].2;
        for (name, _, count) in &results {
            assert_eq!(
                *count, expected_count,
                "{} found {} files, expected {}",
                name, count, expected_count
            );
        }

        // Assert: ignore crate should be fastest or very close
        let ignore_time = results
            .iter()
            .find(|(name, _, _)| *name == "ignore")
            .map(|(_, time, _)| time)
            .unwrap();

        let fastest_time = results.iter().map(|(_, time, _)| time).min().unwrap();

        let ratio = ignore_time.as_secs_f64() / fastest_time.as_secs_f64();
        assert!(
            ratio < 2.0,
            "ignore crate should be within 2x of fastest (ratio: {:.2})",
            ratio
        );

        println!("✅ All assertions passed");
    }
}
