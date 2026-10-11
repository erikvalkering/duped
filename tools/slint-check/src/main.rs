use slint_interpreter::Compiler;
use std::{env, path::PathBuf};

fn main() {
    let mut args = env::args_os().skip(1);
    let path = args.next().unwrap_or_else(|| {
        eprintln!("Usage: cargo slint-check <file.slint>");
        std::process::exit(2);
    });

    let path = PathBuf::from(path);

    let project_root = env::current_dir()
        .expect("Could not determine current directory")
        .parent()
        .expect("Could not determine repository root")
        .to_path_buf();

    let mut compiler = Compiler::default();
    compiler.set_library_paths(std::collections::HashMap::from([(
        "material".to_string(),
        project_root
            .join("duped-android")
            .join("material-1.1.0")
            .join("material.slint"),
    )]));

    let result = spin_on::spin_on(compiler.build_from_path(&path));

    let mut has_error = false;
    for diagnostic in result.diagnostics() {
        eprintln!("{diagnostic:?}");
        // A failed compilation has no component for the requested source.
        // Report the diagnostics above and use component presence as failure.
    }

    if result.component_names().next().is_none() {
        has_error = true;
    }

    if has_error {
        std::process::exit(1);
    }
}
