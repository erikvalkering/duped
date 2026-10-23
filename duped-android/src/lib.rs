slint::include_modules!();

fn ui() -> MainWindow {
    let ui = MainWindow::new().unwrap();
    let ui_weak = ui.as_weak();
    ui.on_scan_downloads(move || {
        let ui_weak2 = ui_weak.clone();
        if let Some(ui) = ui_weak2.upgrade() {
            ui.set_scanning_downloads(true);
        }
        let ui_weak2 = ui_weak.clone();
        std::thread::spawn(move || {
            let mut count = 0;
            let downloads = std::env::var("HOME").unwrap_or("/storage/emulated/0/Download".to_string());
            for entry in walkdir::WalkDir::new(downloads) {
                if entry.is_ok() {
                    count += 1;
                    let ui_weak3 = ui_weak2.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak3.upgrade() {
                            ui.set_downloads_count(count);
                        }
                    }).unwrap();
                }
            }
            let ui_weak4 = ui_weak2.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak4.upgrade() {
                    ui.set_scanning_downloads(false);
                }
            }).unwrap();
        });
    });
    ui
}

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();

    let ui = ui();
    ui.run().unwrap();
}
