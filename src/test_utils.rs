use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestStructure {
    pub temp_dir: TempDir,
    pub files_created: usize,
}

impl TestStructure {
    /// Create a test directory with n files distributed across m levels
    pub fn create(n_files: usize, depth: usize, files_per_dir: usize) -> std::io::Result<Self> {
        let temp_dir = TempDir::new()?;
        let files_created = Self::create_structure(temp_dir.path(), n_files, depth, files_per_dir)?;

        Ok(Self {
            temp_dir,
            files_created,
        })
    }

    pub fn root_path(&self) -> &Path {
        self.temp_dir.path()
    }

    fn create_structure(
        root: &Path,
        n_files: usize,
        depth: usize,
        files_per_dir: usize,
    ) -> std::io::Result<usize> {
        let mut files_created = 0;
        let mut queue = vec![(root.to_path_buf(), 0)];

        while files_created < n_files && !queue.is_empty() {
            let (current_dir, level) = queue.remove(0);

            // Create files in current directory
            let num_files = std::cmp::min(files_per_dir, n_files - files_created);
            for i in 0..num_files {
                let file_path = current_dir.join(format!("file_{}.txt", files_created));
                let mut file = File::create(&file_path)?;
                writeln!(file, "Content {}", files_created)?;
                files_created += 1;
            }

            // Create subdirectories if not at max depth
            if level < depth - 1 {
                let num_subdirs = std::cmp::max(2, files_per_dir / 10);
                for i in 0..num_subdirs {
                    let subdir = current_dir.join(format!("L{}_D{}", level, i));
                    fs::create_dir_all(&subdir)?;
                    queue.push((subdir, level + 1));
                }
            }
        }

        Ok(files_created)
    }
}
