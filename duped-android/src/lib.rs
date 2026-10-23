slint::include_modules!();

fn ui() -> MainWindow {
    let ui = MainWindow::new().unwrap();
    let ui_weak = ui.as_weak();
    ui.on_scan_downloads(move || {
        let ui = ui_weak.unwrap();
        ui.set_scanning_downloads(true);
        std::thread::spawn(move || {
            let mut count = 0;
            let downloads = std::env::var("HOME").unwrap_or("/storage/emulated/0/Download".to_string());
            for entry in walkdir::WalkDir::new(downloads) {
                if entry.is_ok() {
                    count += 1;
                    ui.set_downloads_count(count);
                }
            }
            ui.set_scanning_downloads(false);
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
