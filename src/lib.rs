use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// #[cfg(test)]
pub mod test_utils; // Available in tests and benches

/// Scanning strategy trait
pub trait ScanStrategy {
    fn scan(&self, root: &Path) -> io::Result<Vec<PathBuf>>;
    fn name(&self) -> &'static str;
}

/// Strategy 1: std::fs::read_dir with manual recursion
pub struct StdFsRecursive;

impl ScanStrategy for StdFsRecursive {
    fn name(&self) -> &'static str {
        "std::fs recursive"
    }

    fn scan(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.scan_recursive(root, &mut files)?;
        Ok(files)
    }
}

impl StdFsRecursive {
    fn scan_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                self.scan_recursive(&path, files)?;
            }
        }
        Ok(())
    }
}

/// Strategy 2: walkdir crate (most popular)
pub struct WalkDirStrategy;

impl ScanStrategy for WalkDirStrategy {
    fn name(&self) -> &'static str {
        "walkdir"
    }

    fn scan(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect())
    }
}

/// Strategy 3: walkdir with parallel iteration
pub struct WalkDirParallel;

impl ScanStrategy for WalkDirParallel {
    fn name(&self) -> &'static str {
        "walkdir_parallel"
    }

    fn scan(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(walkdir::WalkDir::new(root)
            .min_depth(1)
            .max_depth(100)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect())
    }
}

/// Strategy 4: ignore crate (fastest, respects .gitignore)
pub struct IgnoreStrategy;

impl ScanStrategy for IgnoreStrategy {
    fn name(&self) -> &'static str {
        "ignore"
    }

    fn scan(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(ignore::WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(false)
            .build()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
            .map(|e| e.path().to_path_buf())
            .collect())
    }
}

/// Strategy 5: With metadata collection
pub struct WithMetadata;

#[derive(Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub modified: std::time::SystemTime,
}

impl WithMetadata {
    pub fn scan(&self, root: &Path) -> io::Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        self.scan_recursive(root, &mut files)?;
        Ok(files)
    }

    fn scan_recursive(&self, dir: &Path, files: &mut Vec<FileInfo>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                files.push(FileInfo {
                    path,
                    size: metadata.len(),
                    modified: metadata.modified()?,
                });
            } else if metadata.is_dir() {
                self.scan_recursive(&path, files)?;
            }
        }
        Ok(())
    }
}

/// Strategy 6: jwalk crate (super-duper fast)
pub struct JwalkStrategy;

impl ScanStrategy for JwalkStrategy {
    fn name(&self) -> &'static str {
        "jwalk"
    }

    fn scan(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(jwalk::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect())
    }
}
