use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
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
        // Treat files_per_dir as the max files per leaf directory.
        if files_per_dir == 0 || n_files == 0 {
            return Ok(0);
        }

        // Number of leaf directories required to hold all files
        let num_leaf_dirs = (n_files + files_per_dir - 1) / files_per_dir;

        // If everything fits in root, just create files there
        if num_leaf_dirs <= 1 {
            let mut files_created = 0usize;
            for _ in 0..std::cmp::min(files_per_dir, n_files) {
                let file_path = root.join(format!("file_{}.txt", files_created));
                let mut file = File::create(&file_path)?;
                writeln!(file, "Content {}", files_created)?;
                files_created += 1;
            }
            return Ok(files_created);
        }

        // Determine branching factor: choose a value based on files_per_dir similar to the
        // previous heuristic, ensuring at least 2 branches per node.
        let branching = std::cmp::max(2, files_per_dir / 10);

        // Determine depth to use by picking the minimal depth such that the tree capacity
        // (total dirs * files_per_dir) can accommodate n_files. This ignores the caller's
        // depth request and ensures the requested number of files will be created.
        let mut depth_to_use = 1usize;
        loop {
            // total directories in a full b-ary tree of depth d is (b^d - 1) / (b - 1)
            let total_dirs = if branching == 1 {
                depth_to_use
            } else {
                let pow = branching.saturating_pow(depth_to_use as u32);
                (pow.saturating_sub(1)).saturating_div(branching - 1)
            };
            let capacity = total_dirs.saturating_mul(files_per_dir);
            if capacity >= n_files {
                break;
            }
            depth_to_use += 1;
        }

        // Now build directories and place files using a BFS approach similar to the original
        // implementation, but using the computed branching and depth_to_use. This interleaves
        // file creation with subdirectory creation and tends to distribute files across
        // directories in BFS order.
        let mut files_created = 0usize;
        let mut queue: Vec<(std::path::PathBuf, usize)> = vec![(root.to_path_buf(), 0)];

        while files_created < n_files && !queue.is_empty() {
            let (current_dir, level) = queue.remove(0);

            // Create files in current directory (up to files_per_dir)
            let num_files_here = std::cmp::min(files_per_dir, n_files - files_created);
            for _ in 0..num_files_here {
                let file_path = current_dir.join(format!("file_{}.txt", files_created));
                let mut file = File::create(&file_path)?;
                writeln!(file, "Content {}", files_created)?;
                files_created += 1;
            }

            // Enqueue subdirectories if not at max depth
            if level < depth_to_use - 1 {
                for i in 0..branching {
                    let subdir = current_dir.join(format!("L{}_D{}", level, i));
                    fs::create_dir_all(&subdir)?;
                    queue.push((subdir, level + 1));
                }
            }
        }

        Ok(files_created)
    }
}
