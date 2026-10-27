slint::include_modules!();

fn ui() -> MainWindow {
    let ui = MainWindow::new().unwrap();
    let ui_weak = ui.as_weak();
    ui.on_scan_downloads({
        let ui_weak = ui_weak.clone();
        move || {
            let ui_weak2 = ui_weak.clone();
            if let Some(ui) = ui_weak2.upgrade() {
                ui.set_scanning_downloads(true);
                ui.set_files_list(slint::ModelRc::from(slint::SharedVector::from_slice(&[])));
            }
            let ui_weak2 = ui_weak.clone();
            std::thread::spawn(move || {
                let mut count = 0;
                let mut files = Vec::new();
                let downloads =
                    std::env::var("HOME").unwrap_or("/storage/emulated/0/Download".to_string());
                for entry in walkdir::WalkDir::new(downloads) {
                    if let Ok(entry) = entry {
                        count += 1;
                        let path = entry.path().display().to_string();
                        files.push(path.clone());
                        let files_clone = files.clone();
                        let ui_weak3 = ui_weak2.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak3.upgrade() {
                                ui.set_files_count(count);
                                ui.set_files_list(slint::ModelRc::from(
                                    slint::SharedVector::from_slice(
                                        &files_clone
                                            .iter()
                                            .map(|s| slint::SharedString::from(s.as_str()))
                                            .collect::<Vec<_>>(),
                                    ),
                                ));
                            }
                        })
                        .unwrap();
                    }
                }
                let ui_weak4 = ui_weak2.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak4.upgrade() {
                        ui.set_scanning_downloads(false);
                    }
                })
                .unwrap();
            });
        }
    });

    ui.on_scan_parent({
        let ui_weak = ui_weak.clone();
        move || {
            let ui_weak2 = ui_weak.clone();
            if let Some(ui) = ui_weak2.upgrade() {
                ui.set_scanning_parent(true);
                ui.set_files_list(slint::ModelRc::from(slint::SharedVector::from_slice(&[])));
            }
            let ui_weak2 = ui_weak.clone();
            std::thread::spawn(move || {
                let mut count = 0;
                let mut files = Vec::new();
                let parent = "/storage/emulated/0";
                for entry in walkdir::WalkDir::new(parent) {
                    if let Ok(entry) = entry {
                        count += 1;
                        let path = entry.path().display().to_string();
                        files.push(path.clone());
                        let files_clone = files.clone();
                        let ui_weak3 = ui_weak2.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak3.upgrade() {
                                ui.set_files_count(count);
                                ui.set_files_list(slint::ModelRc::from(
                                    slint::SharedVector::from_slice(
                                        &files_clone
                                            .iter()
                                            .map(|s| slint::SharedString::from(s.as_str()))
                                            .collect::<Vec<_>>(),
                                    ),
                                ));
                            }
                        })
                        .unwrap();
                    }
                }
                let ui_weak4 = ui_weak2.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak4.upgrade() {
                        ui.set_scanning_parent(false);
                    }
                })
                .unwrap();
            });
        }
    });

    ui.on_scan_root({
        let ui_weak = ui_weak.clone();
        move || {
            let ui_weak2 = ui_weak.clone();
            if let Some(ui) = ui_weak2.upgrade() {
                ui.set_scanning_root(true);
                ui.set_files_list(slint::ModelRc::from(slint::SharedVector::from_slice(&[])));
            }
            let ui_weak2 = ui_weak.clone();
            std::thread::spawn(move || {
                let mut count = 0;
                let mut files = Vec::new();
                let root = "/";
                for entry in walkdir::WalkDir::new(root) {
                    if let Ok(entry) = entry {
                        count += 1;
                        let path = entry.path().display().to_string();
                        files.push(path.clone());
                        let files_clone = files.clone();
                        let ui_weak3 = ui_weak2.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak3.upgrade() {
                                ui.set_files_count(count);
                                ui.set_files_list(slint::ModelRc::from(slint::SharedVector::from(
                                    files_clone
                                        .iter()
                                        .map(|s| slint::SharedString::from(s.as_str()))
                                        .collect::<Vec<_>>(),
                                )));
                            }
                        })
                        .unwrap();
                    }
                }
                let ui_weak4 = ui_weak2.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak4.upgrade() {
                        ui.set_scanning_root(false);
                    }
                })
                .unwrap();
            });
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
