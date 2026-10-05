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
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).expect("Failed to init Slint Android backend");
    let ui = HelloWorld::new().expect("Failed to create main window");
    ui.run().expect("Failed to run Slint event loop");
}
