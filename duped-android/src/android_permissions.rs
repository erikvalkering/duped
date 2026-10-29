use jni::{
    JNIEnv,
    objects::{JObject, JValue},
};

fn sdk_version(env: &mut JNIEnv<'_>) -> jni::errors::Result<i32> {
    env.get_static_field("android/os/Build$VERSION", "SDK_INT", "I")?
        .i()
}

/// Whether Android 11+ all-files access is currently enabled.
pub fn has_all_files_access(env: &mut JNIEnv<'_>) -> jni::errors::Result<bool> {
    if sdk_version(env)? < 30 {
        return Ok(false);
    }

    env.call_static_method(
        "android/os/Environment",
        "isExternalStorageManager",
        "()Z",
        &[],
    )?
    .z()
}

/// Opens the app-specific all-files-access settings page.
pub fn request_all_files_access(
    env: &mut JNIEnv<'_>,
    activity: &JObject<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    if sdk_version(env)? < 30 {
        return Err("All-files access requires Android 11 (API 30) or newer".into());
    }

    let package_name = env
        .call_method(activity, "getPackageName", "()Ljava/lang/String;", &[])?
        .l()?;

    let scheme = env.new_string("package")?;
    let null = JObject::null();

    let uri = env
        .call_static_method(
            "android/net/Uri",
            "fromParts",
            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Landroid/net/Uri;",
            &[
                JValue::Object(scheme.as_ref()),
                JValue::Object(&package_name),
                JValue::Object(&null),
            ],
        )?
        .l()?;

    let action = env.new_string("android.settings.MANAGE_APP_ALL_FILES_ACCESS_PERMISSION")?;

    let intent = env.new_object(
        "android/content/Intent",
        "(Ljava/lang/String;)V",
        &[JValue::Object(action.as_ref())],
    )?;

    env.call_method(
        &intent,
        "setData",
        "(Landroid/net/Uri;)Landroid/content/Intent;",
        &[JValue::Object(&uri)],
    )?;

    env.call_method(
        activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    )?;

    Ok(())
}
