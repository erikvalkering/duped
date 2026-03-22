use duped::test_utils::TestStructure;
use std::fs;

// Use walkdir in tests
use walkdir::WalkDir;

#[test]
fn create_exact_number_of_files() {
    let t = TestStructure::create(100, 3, 10).expect("create failed");
    let root = t.root_path();

    // Count files on disk
    let mut count = 0usize;
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            count += 1;
        }
    }

    assert_eq!(count, 100, "expected 100 files, found {}", count);
}

#[test]
fn respects_files_per_dir_limit() {
    let max_per_dir = 5;
    let total = 23;
    let t = TestStructure::create(total, 2, max_per_dir).expect("create failed");
    let root = t.root_path();

    // For each directory check no more than max_per_dir files
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            let mut files_here = 0usize;
            for child in fs::read_dir(entry.path()).unwrap().filter_map(|e| e.ok()) {
                if child.metadata().map(|m| m.is_file()).unwrap_or(false) {
                    files_here += 1;
                }
            }
            assert!(files_here <= max_per_dir, "dir {} has {} files > {}", entry.path().display(), files_here, max_per_dir);
        }
    }
}
