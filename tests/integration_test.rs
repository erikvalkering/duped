#[cfg(test)]
mod tests {
    use duped::test_utils::TestStructure;
    use duped::*;
    use std::time::Instant;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_relative_performance_ranking() {
        // Create test structure
        let test_structure =
            TestStructure::create(500, 3, 30).expect("Failed to create test structure");
        let root = test_structure.root_path();

        println!("🔬 Testing {} files", test_structure.files_created);

        // Benchmark each strategy
        let strategies: Vec<Box<dyn ScanStrategy>> = vec![
            Box::new(StdFsStrategy),
            Box::new(WalkDirStrategy),
            Box::new(IgnoreStrategy),
            Box::new(JwalkStrategy),
        ];

        let mut results = Vec::new();

        for strategy in strategies {
            let start = Instant::now();
            let files: Vec<_> = strategy.scan(root).collect();
            let duration = start.elapsed();

            println!("{:15} {:>8.2?}", strategy.name(), duration);
            results.push((strategy.name(), duration, files.len()));
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

    #[test]
    fn test_full_content_duplicate_detection() {
        // Create a temporary directory with known duplicates
        let root = tempfile::tempdir().expect("Failed to create temp dir");
        let root_path = root.path();

        // Write test files
        let mut f1 = fs::File::create(root_path.join("file1.txt"))
            .expect("Failed to create file1");
        f1.write_all(b"content1").expect("Failed to write file1");

        let mut f2 = fs::File::create(root_path.join("file2.txt"))
            .expect("Failed to create file2");
        f2.write_all(b"content1").expect("Failed to write file2");

        let mut f3 = fs::File::create(root_path.join("file3.txt"))
            .expect("Failed to create file3");
        f3.write_all(b"content1").expect("Failed to write file3");

        let mut f4 = fs::File::create(root_path.join("file4.txt"))
            .expect("Failed to create file4");
        f4.write_all(b"content2").expect("Failed to write file4");

        let mut f5 = fs::File::create(root_path.join("file5.txt"))
            .expect("Failed to create file5");
        f5.write_all(b"content2").expect("Failed to write file5");

        let mut f6 = fs::File::create(root_path.join("file6.txt"))
            .expect("Failed to create file6");
        f6.write_all(b"content3").expect("Failed to write file6");

        // Scan files
        let scanner = JwalkStrategy;
        let files: Vec<_> = scanner.scan(root_path).collect();
        assert_eq!(files.len(), 6, "Should find 6 files");

        // Find duplicates
        let dedupe = FullContentStrategy;
        let groups = dedupe
            .find_duplicates(files)
            .expect("Failed to find duplicates");

        // Should have 2 groups: one with 3 files (content1) and one with 2 files (content2)
        assert_eq!(groups.len(), 2, "Should have 2 duplicate groups");

        // Check sizes
        let mut sizes: Vec<_> = groups.iter().map(|g| g.size()).collect();
        sizes.sort();
        assert_eq!(sizes, vec![2, 3], "Group sizes should be [2, 3]");
    }

    #[test]
    fn test_full_content_no_duplicates() {
        // Create a temporary directory with unique files
        let root = tempfile::tempdir().expect("Failed to create temp dir");
        let root_path = root.path();

        // Write unique files
        for i in 0..5 {
            let mut f = fs::File::create(root_path.join(format!("file{}.txt", i)))
                .expect("Failed to create file");
            f.write_all(format!("content{}", i).as_bytes())
                .expect("Failed to write file");
        }

        // Scan files
        let scanner = JwalkStrategy;
        let files: Vec<_> = scanner.scan(root_path).collect();
        assert_eq!(files.len(), 5, "Should find 5 files");

        // Find duplicates
        let dedupe = FullContentStrategy;
        let groups = dedupe
            .find_duplicates(files)
            .expect("Failed to find duplicates");

        // Should have 0 groups (no duplicates)
        assert_eq!(groups.len(), 0, "Should have no duplicate groups");
    }
}
