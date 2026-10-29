use jni::{
    JNIEnv,
    objects::{JObject, JValue},
};

pub const FOLDER_PICKER_REQUEST_CODE: i32 = 42;

/// Launches Android's directory picker.
pub fn open_folder_picker(env: &mut JNIEnv<'_>, activity: &JObject<'_>) -> jni::errors::Result<()> {
    let action = env.new_string("android.intent.action.OPEN_DOCUMENT_TREE")?;

    let intent = env.new_object(
        "android/content/Intent",
        "(Ljava/lang/String;)V",
        &[JValue::Object(action.as_ref())],
    )?;

    env.call_method(
        activity,
        "startActivityForResult",
        "(Landroid/content/Intent;I)V",
        &[
            JValue::Object(&intent),
            JValue::Int(FOLDER_PICKER_REQUEST_CODE),
        ],
    )?;

    Ok(())
}
