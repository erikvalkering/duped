#[cfg(test)]
mod tests {
    use duped::test_utils::TestStructure;
    use duped::*;
    use std::time::Instant;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_duplicate_group_with_size() {
        // Test that DuplicateGroup correctly stores and computes sizes
        let paths = vec![
            std::path::PathBuf::from("/tmp/file1.txt"),
            std::path::PathBuf::from("/tmp/file2.txt"),
            std::path::PathBuf::from("/tmp/file3.txt"),
        ];
        let group = DuplicateGroup::new(paths, 1024);
        
        assert_eq!(group.file_count(), 3);
        assert_eq!(group.size_bytes, 1024);
        assert_eq!(group.total_size(), 3072); // 3 * 1024
    }

    #[test]
    fn test_wasted_space_calculation() {
        // Test that wasted space is correctly calculated (count - 1) * size
        let paths = vec![
            std::path::PathBuf::from("/tmp/file1.txt"),
            std::path::PathBuf::from("/tmp/file2.txt"),
        ];
        let group = DuplicateGroup::new(paths, 100);
        
        // Wasted space = (2 - 1) * 100 = 100
        let wasted = (group.file_count() as u64 - 1) * group.size_bytes;
        assert_eq!(wasted, 100);
    }


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

        // Find duplicates
        let dedupe = FullContentStrategy;
        let groups = dedupe
            .find_duplicates(files, None)
            .expect("Failed to find duplicates");

        // Should have 2 groups: one with 3 files (content1) and one with 2 files (content2)
        assert_eq!(groups.len(), 2, "Should have 2 duplicate groups");

        // Check sizes
        let mut sizes: Vec<_> = groups.iter().map(|g| g.file_count()).collect();
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

        // Find duplicates
        let dedupe = FullContentStrategy;
        let groups = dedupe
            .find_duplicates(files, None)
            .expect("Failed to find duplicates");

        // Should have 0 groups (no duplicates)
        assert_eq!(groups.len(), 0, "Should have no duplicate groups");
    }

    #[test]
    fn test_size_filter_on_duplicates() {
        // Create files with different sizes
        let root = tempfile::tempdir().expect("Failed to create temp dir");
        let root_path = root.path();

        // Small duplicate group: 5 bytes each
        let mut f1 = fs::File::create(root_path.join("small1.txt"))
            .expect("Failed to create small1");
        f1.write_all(b"hello").expect("Failed to write");

        let mut f2 = fs::File::create(root_path.join("small2.txt"))
            .expect("Failed to create small2");
        f2.write_all(b"hello").expect("Failed to write");

        // Large duplicate group: 100 bytes each
        let large_content = "x".repeat(100);
        let mut f3 = fs::File::create(root_path.join("large1.txt"))
            .expect("Failed to create large1");
        f3.write_all(large_content.as_bytes()).expect("Failed to write");

        let mut f4 = fs::File::create(root_path.join("large2.txt"))
            .expect("Failed to create large2");
        f4.write_all(large_content.as_bytes()).expect("Failed to write");

        // Scan and find duplicates
        let scanner = JwalkStrategy;
        let files: Vec<_> = scanner.scan(root_path).collect();

        let dedupe = FullContentStrategy;
        let all_groups = dedupe
            .find_duplicates(files, None)
            .expect("Failed to find duplicates");

        // Should find 2 groups before filtering
        assert_eq!(all_groups.len(), 2);

        // Test filter: only groups >= 50 bytes
        let filtered: Vec<_> = all_groups
            .into_iter()
            .filter(|g| g.size_bytes >= 50)
            .collect();

        // Should only have the large group (100 bytes)
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].size_bytes, 100);
    }

    #[test]
    fn test_early_size_filtering() {
        // Create files with different sizes to test early filtering
        let root = tempfile::tempdir().expect("Failed to create temp dir");
        let root_path = root.path();

        // Small duplicates: 10 bytes each
        let mut small1 = fs::File::create(root_path.join("small1.txt"))
            .expect("Failed to create small1");
        small1.write_all(b"small12345").expect("Failed to write");

        let mut small2 = fs::File::create(root_path.join("small2.txt"))
            .expect("Failed to create small2");
        small2.write_all(b"small12345").expect("Failed to write");

        // Large duplicates: 1000 bytes each
        let large_content = "x".repeat(1000);
        let mut large1 = fs::File::create(root_path.join("large1.txt"))
            .expect("Failed to create large1");
        large1.write_all(large_content.as_bytes()).expect("Failed to write");

        let mut large2 = fs::File::create(root_path.join("large2.txt"))
            .expect("Failed to create large2");
        large2.write_all(large_content.as_bytes()).expect("Failed to write");

        // Scan files
        let scanner = JwalkStrategy;
        let files: Vec<_> = scanner.scan(root_path).collect();
        assert_eq!(files.len(), 4);

        // Find duplicates with min_size = 100 (filters out small files before reading)
        let dedupe = FullContentStrategy;
        let groups = dedupe
            .find_duplicates(files, Some(100))
            .expect("Failed to find duplicates");

        // Should only find 1 group (the large duplicates)
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].size_bytes, 1000);
        assert_eq!(groups[0].file_count(), 2);
    }

    #[test]
    fn test_size_only_strategy() {
        // Create a temporary directory with files of various sizes
        let root = tempfile::tempdir().expect("Failed to create temp dir");
        let root_path = root.path();

        // Group 1: 3 files of 100 bytes (different contents, but same size)
        for i in 1..=3 {
            let mut f = fs::File::create(root_path.join(format!("file100_{}.txt", i)))
                .expect("Failed to create file");
            f.write_all(format!("content{}", i).as_bytes())
                .expect("Failed to write");
            // Pad to 100 bytes
            f.write_all(&vec![b' '; 100 - format!("content{}", i).len()])
                .expect("Failed to write");
        }

        // Group 2: 2 files of 200 bytes
        for i in 1..=2 {
            let mut f = fs::File::create(root_path.join(format!("file200_{}.txt", i)))
                .expect("Failed to create file");
            f.write_all(&vec![b'x'; 200]).expect("Failed to write");
        }

        // Unique file: 50 bytes
        let mut f = fs::File::create(root_path.join("unique.txt"))
            .expect("Failed to create unique");
        f.write_all(&vec![b'y'; 50]).expect("Failed to write");

        // Scan files
        let scanner = JwalkStrategy;
        let files: Vec<_> = scanner.scan(root_path).collect();
        assert_eq!(files.len(), 6);

        // Find groups by size only (no content comparison)
        let dedupe = SizeOnlyStrategy;
        let groups = dedupe
            .find_duplicates(files, None)
            .expect("Failed to find duplicates");

        // Should find 2 groups: one with 3 files (100 bytes) and one with 2 files (200 bytes)
        assert_eq!(groups.len(), 2);

        // Verify group sizes
        let mut file_counts: Vec<_> = groups.iter().map(|g| g.file_count()).collect();
        file_counts.sort();
        assert_eq!(file_counts, vec![2, 3]);
    }

    #[test]
    fn test_size_only_with_min_size_filter() {
        let root = tempfile::tempdir().expect("Failed to create temp dir");
        let root_path = root.path();

        // Small duplicates: 10 bytes each
        for i in 1..=2 {
            let mut f = fs::File::create(root_path.join(format!("small_{}.txt", i)))
                .expect("Failed to create small");
            f.write_all(b"small12345").expect("Failed to write");
        }

        // Large duplicates: 500 bytes each
        for i in 1..=2 {
            let mut f = fs::File::create(root_path.join(format!("large_{}.txt", i)))
                .expect("Failed to create large");
            f.write_all(&vec![b'x'; 500]).expect("Failed to write");
        }

        // Scan files
        let scanner = JwalkStrategy;
        let files: Vec<_> = scanner.scan(root_path).collect();
        assert_eq!(files.len(), 4);

        // Find groups with min_size = 100 (should skip small files)
        let dedupe = SizeOnlyStrategy;
        let groups = dedupe
            .find_duplicates(files, Some(100))
            .expect("Failed to find duplicates");

        // Should only find 1 group (the 500-byte files)
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].size_bytes, 500);
        assert_eq!(groups[0].file_count(), 2);
    }

    #[test]
    fn test_partial_content_first_bytes() {
        // Create a temporary directory with files that differ only at the end
        let root = tempfile::tempdir().expect("Failed to create temp dir");
        let root_path = root.path();

        // Files with same first 10 bytes but different endings
        let mut f1 = fs::File::create(root_path.join("f1.txt"))
            .expect("Failed to create f1");
        f1.write_all(b"0123456789AAAA").expect("Failed to write");

        let mut f2 = fs::File::create(root_path.join("f2.txt"))
            .expect("Failed to create f2");
        f2.write_all(b"0123456789BBBB").expect("Failed to write");

        let mut f3 = fs::File::create(root_path.join("f3.txt"))
            .expect("Failed to create f3");
        f3.write_all(b"9876543210CCCC").expect("Failed to write");

        // Scan files
        let scanner = JwalkStrategy;
        let files: Vec<_> = scanner.scan(root_path).collect();
        assert_eq!(files.len(), 3);

        // Group by first 10 bytes
        let dedupe = PartialContentStrategy::new(10);
        let groups = dedupe
            .find_duplicates(files, None)
            .expect("Failed to find duplicates");

        // Should find 1 group (f1 and f2 have same first 10 bytes)
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].file_count(), 2);
    }

    #[test]
    fn test_partial_content_last_bytes() {
        // Create files that differ only at the beginning
        let root = tempfile::tempdir().expect("Failed to create temp dir");
        let root_path = root.path();

        // Files with same last 10 bytes but different starts
        let mut f1 = fs::File::create(root_path.join("f1.txt"))
            .expect("Failed to create f1");
        f1.write_all(b"AAAA0123456789").expect("Failed to write");

        let mut f2 = fs::File::create(root_path.join("f2.txt"))
            .expect("Failed to create f2");
        f2.write_all(b"BBBB0123456789").expect("Failed to write");

        let mut f3 = fs::File::create(root_path.join("f3.txt"))
            .expect("Failed to create f3");
        f3.write_all(b"CCCC9876543210").expect("Failed to write");

        // Scan files
        let scanner = JwalkStrategy;
        let files: Vec<_> = scanner.scan(root_path).collect();
        assert_eq!(files.len(), 3);

        // Group by last 10 bytes (negative = from end)
        let dedupe = PartialContentStrategy::new(-10);
        let groups = dedupe
            .find_duplicates(files, None)
            .expect("Failed to find duplicates");

        // Should find 1 group (f1 and f2 have same last 10 bytes)
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].file_count(), 2);
    }

    #[test]
    fn test_partial_content_with_min_size() {
        // Create files with different sizes
        let root = tempfile::tempdir().expect("Failed to create temp dir");
        let root_path = root.path();

        // Small files: 5 bytes
        let mut small1 = fs::File::create(root_path.join("small1.txt"))
            .expect("Failed to create small1");
        small1.write_all(b"hello").expect("Failed to write");

        let mut small2 = fs::File::create(root_path.join("small2.txt"))
            .expect("Failed to create small2");
        small2.write_all(b"hello").expect("Failed to write");

        // Large files: 100 bytes with same first 50
        let large_content = "x".repeat(100);
        let mut large1 = fs::File::create(root_path.join("large1.txt"))
            .expect("Failed to create large1");
        large1.write_all(large_content.as_bytes()).expect("Failed to write");

        let mut large2 = fs::File::create(root_path.join("large2.txt"))
            .expect("Failed to create large2");
        large2.write_all(large_content.as_bytes()).expect("Failed to write");

        // Scan files
        let scanner = JwalkStrategy;
        let files: Vec<_> = scanner.scan(root_path).collect();
        assert_eq!(files.len(), 4);

        // Group by first 50 bytes with min_size filter
        let dedupe = PartialContentStrategy::new(50);
        let groups = dedupe
            .find_duplicates(files, Some(50))
            .expect("Failed to find duplicates");

        // Should only find the large files group (small files filtered out)
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].file_count(), 2);
        assert_eq!(groups[0].size_bytes, 100);
    }
}
