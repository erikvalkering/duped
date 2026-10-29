use jni::objects::{JObject, JString};
use jni::JNIEnv;

/// Handles onActivityResult for folder picker
/// Returns Some(path) if a folder was picked, None otherwise
pub fn handle_folder_picker_result(env: &JNIEnv, data: JObject) -> Option<String> {
    if data.is_null() {
        return None;
    }
    // data.getData() returns a Uri
    let uri = env.call_method(data, "getData", "()Landroid/net/Uri;", &[]).ok()?.l().ok()?;
    // Use DocumentsContract.getTreeDocumentId(uri) to get the document ID
    let docs_contract = env.find_class("android/provider/DocumentsContract").ok()?;
    let doc_id_jstr = env.call_static_method(
        docs_contract,
        "getTreeDocumentId",
        "(Landroid/net/Uri;)Ljava/lang/String;",
        &[uri.into()],
    ).ok()?.l().ok()?;
    let doc_id: String = env.get_string(JString::from(doc_id_jstr)).ok()?.into();
    // For SAF URIs, reconstruct a path (best effort)
    Some(format!("/storage/emulated/0/{}", doc_id))
}
