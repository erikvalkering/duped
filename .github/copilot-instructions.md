# Copilot instructions for duped

This repository is a small Rust CLI tool for fast duplicate-file discovery and directory scanning using multiple strategies (std::fs recursive, walkdir, ignore, jwalk). These instructions help Copilot sessions and assistants work effectively here.

Build, test, bench, and lint commands

- Build: `cargo build` (use `--release` for optimized builds)
- Run binary locally: `cargo run -- <args>` or `cargo run --release -- <args>`
- Run all tests: `cargo test`
- Run a single test by name/pattern: `cargo test <pattern>` (e.g., `cargo test test_relative_performance_ranking`)
- Run benches (Criterion): `cargo bench`
- Run a single bench by filtering Criterion group/name via environment or cargo options; generally use `cargo bench --bench dir_scanning`
- Format check: `cargo fmt -- --check` (if rustfmt installed)
- Lint with clippy: `cargo clippy -- -D warnings` (if clippy installed)

High-level architecture (big picture)

- Core crate: src/lib.rs
  - Exposes a ScanStrategy trait and multiple implementations:
    - StdFsRecursive: recursive traversal using std::fs::read_dir
    - WalkDirStrategy: uses the walkdir crate
    - IgnoreStrategy: uses ignore crate (respects ignore rules when configured)
    - JwalkStrategy: uses jwalk for concurrent scanning
  - Strategies return Vec<PathBuf> of files found under a root path.
  - Test utilities (src/test_utils.rs) provide TestStructure for creating on-disk test fixtures (temp dirs with many files/subdirs).

- CLI and runner: src/main.rs
  - Uses clap to parse options (path, strategy, timing, silent)
  - Selects a scanning strategy and prints results/timing

- Benchmarks: benches/dir_scanning.rs
  - Uses Criterion to measure scanning performance across strategies
  - Persists baseline data in a temp file for regression checks

- Tests: tests/integration_test.rs
  - Exercises all strategies on generated test fixtures and asserts file counts and relative performance expectations

Key conventions and repository-specific patterns

- Strategy pattern for scanning: Keep new scanning approaches as types implementing ScanStrategy. Export them from lib so benches/tests can construct them directly.
- Test fixtures: Use test_utils::TestStructure to create reproducible directory trees for tests and benches; prefer this instead of handwritten temp directories.
- Silent/timing flags in CLI: `--silent` suppresses file listing (useful for timing-only runs); `--time` prints elapsed time.
- Benchmark baselines: Criterion bench harness persists baseline JSON to the system temp dir (`dir_scan_baseline.json`) to detect regressions; be careful when running benches on CI vs developer machines.
- Error handling: Directory scanning implementations print and continue on non-fatal errors (e.g., permission denied), but return Err on unrecoverable IO errors.

Files to consult for implementation details

- src/lib.rs — core scanning trait and strategies
- src/main.rs — CLI glue and strategy selection
- src/test_utils.rs — test/benchmark fixture generator
- benches/dir_scanning.rs — Criterion benchmarks and regression helpers
- tests/integration_test.rs — integration assertions and behavioral expectations

AI / Copilot guidance specific to this repo

- When suggesting new scanning strategies, follow the ScanStrategy trait signature exactly and ensure the implementation does not follow symlinks by default (current strategies skip symlinks).
- Prefer using the existing TestStructure to validate behavior in tests and benches.
- For performance changes, update or re-run Criterion baselines and include notes about hardware variability; don't assume bench numbers are stable across machines.

If .github/copilot-instructions.md already exists, merge additions that reference benches and test utilities, and avoid removing any CI-related instructions present there.

---

Would you like to configure any MCP servers (e.g., Playwright) for this project? If so, specify which server(s) to add.