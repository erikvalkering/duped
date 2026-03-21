use std::fs;
use std::path::{Path, PathBuf};

// #[cfg(test)]
pub mod test_utils;

/// Iterator-based scan strategy: returns a boxed iterator producing PathBufs.
/// Implementations should strive to avoid per-file heap allocations where possible.
pub trait ScanStrategy {
    fn name(&self) -> &'static str;

    fn scan<'a>(&'a self, root: &'a Path) -> Box<dyn Iterator<Item = PathBuf> + 'a>;
}

pub struct StdFsRecursive;

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

impl ScanStrategy for StdFsRecursive {
    fn name(&self) -> &'static str {
        "std::fs recursive"
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
