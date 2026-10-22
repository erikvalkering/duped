slint::include_modules!();

fn ui() -> MainWindow {
    MainWindow::new().unwrap()
    // TODO: add logic for scan button
}

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();

    let ui = ui();
    ui.run().unwrap();
}
