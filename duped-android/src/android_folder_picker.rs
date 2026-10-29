use jni::objects::{JObject, JValue};
use jni::JNIEnv;

/// Launches the Android folder picker (ACTION_OPEN_DOCUMENT_TREE)
pub fn open_folder_picker(env: &JNIEnv, context: JObject) {
    let intent_class = env.find_class("android/content/Intent").unwrap();
    let action = env.new_string("android.intent.action.OPEN_DOCUMENT_TREE").unwrap();
    let intent = env.new_object(
        intent_class,
        "(Ljava/lang/String;)V",
        &[JValue::from(action)],
    ).unwrap();
    env.call_method(
        context,
        "startActivityForResult",
        "(Landroid/content/Intent;I)V",
        &[intent.into(), JValue::from(42)], // 42 = request code
    ).unwrap();
}
