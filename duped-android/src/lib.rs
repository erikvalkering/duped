#[cfg(target_os = "android")]
mod activity_result;

#[cfg(target_os = "android")]
mod android_folder_picker;

#[cfg(target_os = "android")]
mod android_permissions;

slint::include_modules!();

use slint::ComponentHandle;

use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};

#[cfg(target_os = "android")]
static ANDROID_APP: OnceLock<slint::android::AndroidApp> = OnceLock::new();

#[cfg(target_os = "android")]
static MAIN_WINDOW: OnceLock<slint::Weak<MainWindow>> = OnceLock::new();

#[derive(Clone, Copy)]
enum ScanKind {
    Downloads,
    Documents,
    Root,
}

impl ScanKind {
    fn set_scanning(self, ui: &MainWindow, scanning: bool) {
        match self {
            Self::Downloads => {
                ui.set_scanning_downloads(scanning);
            }
            Self::Documents => {
                ui.set_scanning_documents(scanning);
            }
            Self::Root => {
                ui.set_scanning_root(scanning);
            }
        }
    }
}

fn string_model(values: Vec<String>) -> slint::ModelRc<slint::SharedString> {
    let values: Vec<slint::SharedString> =
        values.into_iter().map(slint::SharedString::from).collect();

    slint::ModelRc::new(slint::VecModel::from(values))
}

fn append_debug_entry(message: String) {
    let _ = slint::invoke_from_event_loop(move || {
        let Some(ui_weak) = MAIN_WINDOW.get() else {
            return;
        };

        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut entries: Vec<slint::SharedString> = Vec::new();
        let model = ui.get_debug_entries();

        for index in 0..model.row_count() {
            if let Some(value) = model.row_data(index) {
                entries.push(value);
            }
        }

        entries.push(slint::SharedString::from(message));

        ui.set_debug_entries(slint::ModelRc::new(slint::VecModel::from(entries)));
    });
}

fn start_scan(
    ui_weak: &slint::Weak<MainWindow>,
    cancel_flag: Arc<AtomicBool>,
    scan_running: Arc<AtomicBool>,
    kind: ScanKind,
    path: String,
) {
    // Only one scan may run at a time.
    if scan_running
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        return;
    }

    let Some(ui) = ui_weak.upgrade() else {
        scan_running.store(false, Ordering::Relaxed);
        return;
    };

    // A previous cancellation must not cancel the new scan.
    cancel_flag.store(false, Ordering::Relaxed);

    kind.set_scanning(&ui, true);
    ui.set_files_count(0);
    ui.set_files_list(string_model(Vec::new()));
    ui.set_debug_entries(string_model(Vec::new()));

    let ui_weak = ui_weak.clone();

    std::thread::spawn(move || {
        let mut files: Vec<String> = Vec::new();
        let mut debug_entries: Vec<String> = Vec::new();

        for entry in walkdir::WalkDir::new(&path) {
            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            let mut files_update = None;

            match entry {
                Ok(entry) => {
                    let entry_type = if entry.file_type().is_file() {
                        "[F]"
                    } else if entry.file_type().is_dir() {
                        "[D]"
                    } else {
                        "[?]"
                    };

                    debug_entries.push(format!("{} {}", entry_type, entry.path().display(),));

                    if entry.file_type().is_file() {
                        files.push(entry.path().display().to_string());
                        files_update = Some(files.clone());
                    }
                }

                Err(err) => {
                    debug_entries.push(format!("[ERR] {err}"));
                }
            }

            let debug_update = debug_entries.clone();
            let ui_weak_update = ui_weak.clone();

            // Construct Slint models on the UI thread.
            let result = slint::invoke_from_event_loop(move || {
                let Some(ui) = ui_weak_update.upgrade() else {
                    return;
                };

                ui.set_debug_entries(string_model(debug_update));

                if let Some(files) = files_update {
                    let count = files.len().min(i32::MAX as usize) as i32;

                    ui.set_files_count(count);
                    ui.set_files_list(string_model(files));
                }
            });

            // The event loop may have stopped during shutdown.
            if result.is_err() {
                break;
            }
        }

        let scan_running_done = scan_running.clone();

        let result = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                kind.set_scanning(&ui, false);
            }

            scan_running_done.store(false, Ordering::Relaxed);
        });

        if result.is_err() {
            scan_running.store(false, Ordering::Relaxed);
        }
    });
}

#[cfg(target_os = "android")]
fn check_all_files_access(app: &slint::android::AndroidApp) -> jni::errors::Result<bool> {
    // SAFETY: AndroidApp supplies a valid JVM pointer.
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) }?;

    let mut env = vm.attach_current_thread()?;

    env.with_local_frame(16, |env| android_permissions::has_all_files_access(env))
}

#[cfg(target_os = "android")]
fn open_all_files_access_settings(
    app: &slint::android::AndroidApp,
) -> Result<(), Box<dyn std::error::Error>> {
    // SAFETY: AndroidApp supplies a valid JVM pointer.
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) }?;

    let mut env = vm.attach_current_thread()?;

    env.with_local_frame(16, |env| -> Result<(), Box<dyn std::error::Error>> {
        // SAFETY: AndroidApp supplies an unowned global
        // reference. The app remains alive during this call.
        // Do not delete it or wrap it in AutoLocal.
        let activity = unsafe { jni::objects::JObject::from_raw(app.activity_as_ptr().cast()) };

        android_permissions::request_all_files_access(env, &activity)
    })
}

#[cfg(target_os = "android")]
fn open_android_folder_picker(app: &slint::android::AndroidApp) -> jni::errors::Result<()> {
    // SAFETY: AndroidApp supplies a valid JVM pointer.
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) }?;

    let mut env = vm.attach_current_thread()?;

    env.with_local_frame(16, |env| {
        // SAFETY: AndroidApp supplies an unowned global
        // reference. The app remains alive during this call.
        // Do not delete it or wrap it in AutoLocal.
        let activity = unsafe { jni::objects::JObject::from_raw(app.activity_as_ptr().cast()) };

        android_folder_picker::open_folder_picker(env, &activity)
    })
}

#[cfg(target_os = "android")]
fn ui(app: slint::android::AndroidApp) -> MainWindow {
    let ui = MainWindow::new().unwrap();
    let ui_weak = ui.as_weak();

    let _ = MAIN_WINDOW.set(ui.as_weak());

    let cancel_flag = Arc::new(AtomicBool::new(false));
    let scan_running = Arc::new(AtomicBool::new(false));

    ui.on_scan_downloads({
        let ui_weak = ui_weak.clone();
        let cancel_flag = cancel_flag.clone();
        let scan_running = scan_running.clone();

        move || {
            start_scan(
                &ui_weak,
                cancel_flag.clone(),
                scan_running.clone(),
                ScanKind::Downloads,
                "/storage/emulated/0/Download".to_string(),
            );
        }
    });

    ui.on_scan_documents({
        let ui_weak = ui_weak.clone();
        let cancel_flag = cancel_flag.clone();
        let scan_running = scan_running.clone();

        move || {
            start_scan(
                &ui_weak,
                cancel_flag.clone(),
                scan_running.clone(),
                ScanKind::Documents,
                "/storage/emulated/0/Documents".to_string(),
            );
        }
    });

    ui.on_scan_root({
        let ui_weak = ui_weak.clone();
        let cancel_flag = cancel_flag.clone();
        let scan_running = scan_running.clone();

        move || {
            start_scan(
                &ui_weak,
                cancel_flag.clone(),
                scan_running.clone(),
                ScanKind::Root,
                "/".to_string(),
            );
        }
    });

    ui.on_cancel_scan({
        let cancel_flag = cancel_flag.clone();

        move || {
            cancel_flag.store(true, Ordering::Relaxed);
        }
    });

    match check_all_files_access(&app) {
        Ok(granted) => {
            ui.set_all_files_access(granted);
        }

        Err(err) => {
            eprintln!("Could not check all-files access: {err}");
            ui.set_all_files_access(false);
        }
    }

    ui.on_request_all_files_access({
        let app = app.clone();

        move || {
            if let Err(err) = open_all_files_access_settings(&app) {
                eprintln!("Could not open all-files settings: {err}");
            }
        }
    });

    ui.on_browse_folder({
        let app = app.clone();

        move |_dummy| {
            if let Err(err) = open_android_folder_picker(&app) {
                eprintln!("Could not open folder picker: {err}");
                append_debug_entry(format!("[PICKER ERROR] {err}"));
            }
        }
    });

    ui
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn duped_on_folder_picker_result(
    request_code: jni::sys::jint,
    result_code: jni::sys::jint,
    data: jni::sys::jobject,
) {
    const RESULT_OK: jni::sys::jint = -1;

    if request_code != android_folder_picker::FOLDER_PICKER_REQUEST_CODE {
        return;
    }

    if result_code != RESULT_OK {
        append_debug_entry("[PICKED] Folder selection cancelled".to_string());
        return;
    }

    if data.is_null() {
        append_debug_entry("[PICKED] No folder URI returned".to_string());
        return;
    }

    let Some(app) = ANDROID_APP.get() else {
        append_debug_entry("[PICKER ERROR] Android app not initialized".to_string());
        return;
    };

    // SAFETY: AndroidApp supplies a valid JVM pointer.
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) };

    let Ok(vm) = vm else {
        append_debug_entry("[PICKER ERROR] Could not obtain JavaVM".to_string());
        return;
    };

    let mut env = match vm.attach_current_thread() {
        Ok(env) => env,
        Err(err) => {
            append_debug_entry(format!("[PICKER ERROR] Could not attach JNI thread: {err}"));
            return;
        }
    };

    // SAFETY: `data` is a local JNI reference owned by this callback.
    let data = unsafe { jni::objects::JObject::from_raw(data) };

    let result = env.with_local_frame(16, |env| {
        activity_result::handle_folder_picker_result(env, data)
    });

    let message = match result {
        Ok(Some(uri)) => format!("[PICKED] {uri}"),
        Ok(None) => "[PICKED] No usable folder URI".to_string(),
        Err(err) => format!("[PICKER ERROR] {err}"),
    };

    append_debug_entry(message);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    let _ = ANDROID_APP.set(app.clone());

    slint::android::init(app.clone()).unwrap();

    let ui = ui(app.clone());

    // Retain the timer until the UI event loop exits.
    let permission_timer = slint::Timer::default();

    permission_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(1),
        {
            let ui_weak = ui.as_weak();

            move || {
                let Some(ui) = ui_weak.upgrade() else {
                    return;
                };

                match check_all_files_access(&app) {
                    Ok(granted) => {
                        ui.set_all_files_access(granted);
                    }

                    Err(err) => {
                        eprintln!("Could not refresh all-files access: {err}");
                    }
                }
            }
        },
    );

    ui.run().unwrap();
}
