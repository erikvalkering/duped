slint::slint! {
    export component HelloWorld inherits Window {
        width: 360px;
        height: 640px;
        title: "Duped Hello";

        VerticalLayout {
            alignment: center;
            spacing: 20px;
            padding: 24px;

            Text {
                text: "Hello from Duped!";
                font-size: 24px;
                horizontal-alignment: center;
            }

            Text {
                text: "Slint hello-world inside your existing Rust project.";
                font-size: 14px;
                horizontal-alignment: center;
            }
        }
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: slint::platform::android::AndroidApp) {
    use slint::platform::android::AndroidPlatform;

    AndroidPlatform::new_with_app(app).run_app(|| {
        let ui = HelloWorld::new().expect("Failed to create MainWindow");
        ui.run().expect("Failed to run app");
    });
}
