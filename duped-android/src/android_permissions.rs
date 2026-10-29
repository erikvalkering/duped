use jni::objects::{JObject, JString};
use jni::JNIEnv;
use jni::sys::jboolean;

/// Checks if MANAGE_EXTERNAL_STORAGE is granted
pub fn has_all_files_access(env: &JNIEnv, context: JObject) -> bool {
    let activity = context;
    let package_manager = env
        .call_method(activity, "getPackageManager", "()Landroid/content/pm/PackageManager;", &[])
        .unwrap()
        .l()
        .unwrap();
    let package_name = env
        .call_method(activity, "getPackageName", "()Ljava/lang/String;", &[])
        .unwrap()
        .l()
        .unwrap();
    let permission = env.new_string("android.permission.MANAGE_EXTERNAL_STORAGE").unwrap();
    let result = env
        .call_method(
            package_manager,
            "checkPermission",
            "(Ljava/lang/String;Ljava/lang/String;)I",
            &[permission.into(), package_name.into()],
        )
        .unwrap()
        .i()
        .unwrap();
    // PERMISSION_GRANTED = 0
    if result == 0 {
        // For Android 11+, must also check Environment.isExternalStorageManager()
        let env_class = env.find_class("android/os/Environment").unwrap();
        let is_mgr = env
            .call_static_method(env_class, "isExternalStorageManager", "()Z", &[])
            .unwrap()
            .z()
            .unwrap();
        is_mgr
    } else {
        false
    }
}

/// Opens the settings page for MANAGE_EXTERNAL_STORAGE
pub fn request_all_files_access(env: &JNIEnv, context: JObject) {
    let intent_action = env.new_string("android.settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION").unwrap();
    let uri_class = env.find_class("android/net/Uri").unwrap();
    let package_name = env.call_method(context, "getPackageName", "()Ljava/lang/String;", &[]).unwrap().l().unwrap();
    let uri = env.call_static_method(
        uri_class,
        "fromParts",
        "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Landroid/net/Uri;",
        &[
            JString::from(env.new_string("package").unwrap()).into(),
            package_name.into(),
            JObject::null().into(),
        ],
    ).unwrap().l().unwrap();
    let intent_class = env.find_class("android/content/Intent").unwrap();
    let intent = env.new_object(
        intent_class,
        "(Ljava/lang/String;)V",
        &[JString::from(intent_action).into()],
    ).unwrap();
    env.call_method(
        intent,
        "setData",
        "(Landroid/net/Uri;)Landroid/content/Intent;",
        &[uri.into()],
    ).unwrap();
    env.call_method(
        context,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[intent.into()],
    ).unwrap();
}
