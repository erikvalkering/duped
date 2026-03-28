use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

// #[cfg(test)]
pub mod test_utils;

/// Iterator-based scan strategy: returns a boxed iterator producing PathBufs.
/// Implementations should strive to avoid per-file heap allocations where possible.
pub trait ScanStrategy {
    fn name(&self) -> &'static str;

    fn scan<'a>(&'a self, root: &'a Path) -> Box<dyn Iterator<Item = PathBuf> + 'a>;
}

pub struct StdFsStrategy;

struct StdFsIter {
    stack: Vec<PathBuf>,
}

impl StdFsIter {
    fn new(root: &Path) -> Self {
        StdFsIter {
            stack: vec![root.to_path_buf()],
        }
    }
}

impl Iterator for StdFsIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.stack.pop() {
            match std::fs::metadata(&path) {
                Ok(meta) => {
                    if meta.is_file() {
                        return Some(path);
                    } else if meta.is_dir() {
                        // Read entries and push them onto the stack
                        match fs::read_dir(&path) {
                            Ok(entries) => {
                                // Collect entries into a small vector then push to stack
                                let mut v = Vec::new();
                                for e in entries.filter_map(|e| e.ok()) {
                                    v.push(e.path());
                                }
                                // push in reverse so the iterator is depth-first left-to-right
                                for p in v.into_iter().rev() {
                                    self.stack.push(p);
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading directory {}: {}", path.display(), e);
                                continue;
                            }
                        }
                    }
                    // symlinks and others are skipped
                }
                Err(e) => {
                    eprintln!("Error reading metadata for {}: {}", path.display(), e);
                    continue;
                }
            }
        }
        None
    }
}

impl ScanStrategy for StdFsStrategy {
    fn name(&self) -> &'static str {
        "std::fs"
    }

    fn scan<'a>(&'a self, root: &'a Path) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        Box::new(StdFsIter::new(root))
    }
}

pub struct WalkDirStrategy;

impl ScanStrategy for WalkDirStrategy {
    fn name(&self) -> &'static str {
        "walkdir"
    }

    fn scan<'a>(&'a self, root: &'a Path) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        Box::new(
            walkdir::WalkDir::new(root)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf()),
        )
    }
}

pub struct IgnoreStrategy;

impl ScanStrategy for IgnoreStrategy {
    fn name(&self) -> &'static str {
        "ignore"
    }

    fn scan<'a>(&'a self, root: &'a Path) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        Box::new(
            ignore::WalkBuilder::new(root)
                .hidden(false)
                .git_ignore(false)
                .build()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
                .map(|e| e.path().to_path_buf()),
        )
    }
}

pub struct JwalkStrategy;

impl ScanStrategy for JwalkStrategy {
    fn name(&self) -> &'static str {
        "jwalk"
    }

    fn scan<'a>(&'a self, root: &'a Path) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        Box::new(jwalk::WalkDir::new(root)
            .skip_hidden(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf()))
    }
}

/// Represents a group of duplicate files (all with the same content).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateGroup {
    pub files: Vec<PathBuf>,
    /// Size in bytes of each file (all duplicates have same size)
    pub size_bytes: u64,
}

impl DuplicateGroup {
    pub fn new(files: Vec<PathBuf>, size_bytes: u64) -> Self {
        DuplicateGroup { files, size_bytes }
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Total size of all duplicates in this group (size_bytes * count)
    pub fn total_size(&self) -> u64 {
        self.size_bytes * (self.file_count() as u64)
    }
}

/// Strategy for detecting duplicate files.
pub trait DeduplicateStrategy {
    fn name(&self) -> &'static str;

    /// Takes a vector of file paths and returns groups of duplicates.
    /// Only returns groups with 2+ files (excludes unique files).
    fn find_duplicates(&self, files: Vec<PathBuf>) -> std::io::Result<Vec<DuplicateGroup>>;
}

/// Reference implementation: compares full file contents.
pub struct FullContentStrategy;

impl DeduplicateStrategy for FullContentStrategy {
    fn name(&self) -> &'static str {
        "full-content"
    }

    fn find_duplicates(&self, files: Vec<PathBuf>) -> std::io::Result<Vec<DuplicateGroup>> {
        // Map: file contents -> list of (path, size) pairs with that content
        let mut content_to_paths: HashMap<Vec<u8>, Vec<(PathBuf, u64)>> = HashMap::new();

        for path in files {
            match fs::read(&path) {
                Ok(contents) => {
                    // Get file size
                    let size = match fs::metadata(&path) {
                        Ok(metadata) => metadata.len(),
                        Err(e) => {
                            eprintln!("Warning: Could not get metadata for file {}: {}", path.display(), e);
                            continue;
                        }
                    };
                    content_to_paths
                        .entry(contents)
                        .or_insert_with(Vec::new)
                        .push((path, size));
                }
                Err(e) => {
                    eprintln!("Warning: Could not read file {}: {}", path.display(), e);
                    continue;
                }
            }
        }

        // Filter to only groups with 2+ files (duplicates) and convert to DuplicateGroup
        let duplicate_groups: Vec<DuplicateGroup> = content_to_paths
            .into_values()
            .filter(|group| group.len() >= 2)
            .map(|group| {
                // All files in the group have the same size (same content)
                let size_bytes = group.get(0).map(|(_, size)| *size).unwrap_or(0);
                let paths: Vec<PathBuf> = group.into_iter().map(|(path, _)| path).collect();
                DuplicateGroup::new(paths, size_bytes)
            })
            .collect();

        Ok(duplicate_groups)
    }
}
