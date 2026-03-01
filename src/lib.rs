use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// #[cfg(test)]
pub mod test_utils;

pub trait ScanStrategy {
    fn scan(&self, root: &Path) -> io::Result<Vec<PathBuf>>;
    fn name(&self) -> &'static str;
}

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

pub struct IgnoreStrategy;

impl ScanStrategy for IgnoreStrategy {
    fn name(&self) -> &'static str {
        "ignore"
    }

    fn scan(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(ignore::WalkBuilder::new(root)
            .hidden(true)
            .git_ignore(false)
            .build()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
            .map(|e| e.path().to_path_buf())
            .collect())
    }
}

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
