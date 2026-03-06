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
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading directory {}: {}", dir.display(), e);
                return Err(e);
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Error reading entry in {}: {}", dir.display(), e);
                    continue;
                }
            };

            let path = entry.path();
            let meta = match entry.metadata() {
                // lstat — no symlink following
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error reading metadata for {}: {}", path.display(), e);
                    continue;
                }
            };

            if meta.is_file() {
                files.push(path);
            } else if meta.is_dir() {
                // only real dirs, not symlinks to dirs
                match self.scan_recursive(&path, files) {
                    Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                        eprintln!("Skipping {}: {}", path.display(), e);
                    }
                    result => result?,
                }
            }
            // symlinks are just skipped (meta.is_symlink() == true, neither branch runs)
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
            .hidden(false)
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
            .skip_hidden(false) // ← include hidden files
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect())
    }
}
