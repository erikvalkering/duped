slint::include_modules!();

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

fn start_scan<FSetScan, FSetFiles, FSetCount, FSetDebug>(
    ui_weak: &slint::Weak<MainWindow>,
    cancel_flag: Arc<AtomicBool>,
    path: String,
    set_scanning: FSetScan,
    set_files: FSetFiles,
    set_count: FSetCount,
    set_debug: FSetDebug,
) where
    FSetScan: Fn(&MainWindow, bool) + Send + Sync + 'static,
    FSetFiles: Fn(&MainWindow, slint::ModelRc<slint::SharedString>) + Send + Sync + 'static,
    FSetCount: Fn(&MainWindow, i32) + Send + Sync + 'static,
    FSetDebug: Fn(&MainWindow, slint::ModelRc<slint::SharedString>) + Send + Sync + 'static,
{
    let ui_weak = ui_weak.clone();
    let set_scanning = Arc::new(set_scanning);
    let set_files = Arc::new(set_files);
    let set_count = Arc::new(set_count);
    let set_debug = Arc::new(set_debug);
    std::thread::spawn({
        let set_scanning = set_scanning.clone();
        let set_files = set_files.clone();
        let set_count = set_count.clone();
        let set_debug = set_debug.clone();
        move || {
            let mut count = 0;
            let mut files = Vec::new();
            let mut debug_entries = Vec::new();
            for entry in walkdir::WalkDir::new(path) {
                if cancel_flag.load(Ordering::Relaxed) {
                    break;
                }
                match entry {
                    Ok(entry) => {
                        let entry_type = if entry.file_type().is_file() {
                            "[F]"
                        } else if entry.file_type().is_dir() {
                            "[D]"
                        } else {
                            "[?]"
                        };
                        let entry_str = format!("{} {}", entry_type, entry.path().display());
                        debug_entries.push(entry_str.clone());
                        let debug_entries_clone = debug_entries.clone();
                        let ui_weak_dbg = ui_weak.clone();
                        let set_debug = set_debug.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_dbg.upgrade() {
                                let dbg_vec: Vec<slint::SharedString> = debug_entries_clone
                                    .iter()
                                    .map(|s| slint::SharedString::from(s.as_str()))
                                    .collect();
                                set_debug(&ui, slint::ModelRc::from(dbg_vec.as_slice()));
                            }
                        }).unwrap();
                        if entry.file_type().is_file() {
                            count += 1;
                            let path = entry.path().display().to_string();
                            files.push(path.clone());
                            let files_clone = files.clone();
                            let ui_weak3 = ui_weak.clone();
                            let set_files = set_files.clone();
                            let set_count = set_count.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak3.upgrade() {
                                    set_count(&ui, count);
                                    let files_vec: Vec<slint::SharedString> = files_clone
                                        .iter()
                                        .map(|s| slint::SharedString::from(s.as_str()))
                                        .collect();
                                    set_files(&ui, slint::ModelRc::from(files_vec.as_slice()));
                                }
                            })
                            .unwrap();
                        }
                    }
                    Err(e) => {
                        let entry_str = format!("[ERR] {}", e);
                        debug_entries.push(entry_str.clone());
                        let debug_entries_clone = debug_entries.clone();
                        let ui_weak_dbg = ui_weak.clone();
                        let set_debug = set_debug.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_dbg.upgrade() {
                                let dbg_vec: Vec<slint::SharedString> = debug_entries_clone
                                    .iter()
                                    .map(|s| slint::SharedString::from(s.as_str()))
                                    .collect();
                                set_debug(&ui, slint::ModelRc::from(dbg_vec.as_slice()));
                            }
                        }).unwrap();
                    }
                }
            }
            let ui_weak4 = ui_weak.clone();
            let set_scanning = set_scanning.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak4.upgrade() {
                    set_scanning(&ui, false);
                }
            })
            .unwrap();
        }
    });
}

fn ui() -> MainWindow {
    let ui = MainWindow::new().unwrap();
    let ui_weak = ui.as_weak();
    let cancel_flag = Arc::new(AtomicBool::new(false));

    ui.on_scan_downloads({
        let ui_weak = ui_weak.clone();
        let cancel_flag = cancel_flag.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_scanning_downloads(true);
                ui.set_files_list(slint::ModelRc::from(&[] as &[slint::SharedString]));
                ui.set_debug_entries(slint::ModelRc::from(&[] as &[slint::SharedString]));
            }
            start_scan(
                &ui_weak,
                cancel_flag.clone(),
                std::env::var("HOME").unwrap_or("/storage/emulated/0/Download".to_string()),
                |ui, val| ui.set_scanning_downloads(val),
                |ui, files| ui.set_files_list(files),
                |ui, count| ui.set_files_count(count),
                |ui, dbg| ui.set_debug_entries(dbg),
            );
        }
    });

    ui.on_scan_documents({
        let ui_weak = ui_weak.clone();
        let cancel_flag = cancel_flag.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_scanning_documents(true);
                ui.set_files_list(slint::ModelRc::from(&[] as &[slint::SharedString]));
                ui.set_debug_entries(slint::ModelRc::from(&[] as &[slint::SharedString]));
            }
            start_scan(
                &ui_weak,
                cancel_flag.clone(),
                "/storage/emulated/0/Documents".to_string(),
                |ui, val| ui.set_scanning_documents(val),
                |ui, files| ui.set_files_list(files),
                |ui, count| ui.set_files_count(count),
                |ui, dbg| ui.set_debug_entries(dbg),
            );
        }
    });

    ui.on_scan_root({
        let ui_weak = ui_weak.clone();
        let cancel_flag = cancel_flag.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_scanning_root(true);
                ui.set_files_list(slint::ModelRc::from(&[] as &[slint::SharedString]));
                ui.set_debug_entries(slint::ModelRc::from(&[] as &[slint::SharedString]));
            }
            start_scan(
                &ui_weak,
                cancel_flag.clone(),
                "/".to_string(),
                |ui, val| ui.set_scanning_root(val),
                |ui, files| ui.set_files_list(files),
                |ui, count| ui.set_files_count(count),
                |ui, dbg| ui.set_debug_entries(dbg),
            );
        }
    });
    ui.on_cancel_scan({
        let cancel_flag = cancel_flag.clone();
        move || {
            cancel_flag.store(true, Ordering::Relaxed);
        }
    });
    ui
}

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();

    let ui = ui();
    ui.run().unwrap();
}
