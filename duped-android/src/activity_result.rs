use jni::{
    JNIEnv,
    objects::{JObject, JString, JValue},
};

/// Processes the Intent returned by the folder picker.
///
/// Call this only after checking the request code and confirming
/// that the activity result is RESULT_OK.
///
/// The returned path preserves the original implementation's
/// best-effort behavior; it is not a general SAF URI-to-path
/// conversion.
pub fn handle_folder_picker_result(env: &mut JNIEnv<'_>, data: JObject<'_>) -> Option<String> {
    if data.is_null() {
        return None;
    }

    // data.getData() returns an android.net.Uri.
    let uri = env
        .call_method(&data, "getData", "()Landroid/net/Uri;", &[])
        .ok()?
        .l()
        .ok()?;

    if uri.is_null() {
        return None;
    }

    let docs_contract = env.find_class("android/provider/DocumentsContract").ok()?;

    let doc_id_jstr = env
        .call_static_method(
            docs_contract,
            "getTreeDocumentId",
            "(Landroid/net/Uri;)Ljava/lang/String;",
            &[JValue::Object(&uri)],
        )
        .ok()?
        .l()
        .ok()?;

    if doc_id_jstr.is_null() {
        return None;
    }

    let doc_id_jstr = JString::from(doc_id_jstr);

    let doc_id: String = env.get_string(&doc_id_jstr).ok()?.into();

    // Preserved from the original code.
    // This is not a reliable conversion for arbitrary providers.
    Some(format!("/storage/emulated/0/{doc_id}"))
}
