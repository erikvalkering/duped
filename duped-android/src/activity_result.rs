use jni::{
    JNIEnv,
    objects::{JObject, JString, JValue},
};

/// Extracts the selected directory URI from the picker result.
///
/// SAF returns a `content://` tree URI, not necessarily a local
/// filesystem path. This function returns the URI string; it does not
/// attempt an unreliable URI-to-path conversion.
pub fn handle_folder_picker_result(env: &mut JNIEnv<'_>, data: JObject<'_>) -> Option<String> {
    if data.is_null() {
        return None;
    }

    // Intent.getData() returns an android.net.Uri.
    let uri = env
        .call_method(&data, "getData", "()Landroid/net/Uri;", &[])
        .ok()?
        .l()
        .ok()?;

    if uri.is_null() {
        return None;
    }

    let uri_string = env
        .call_method(&uri, "toString", "()Ljava/lang/String;", &[])
        .ok()?
        .l()
        .ok()?;

    if uri_string.is_null() {
        return None;
    }

    let uri_jstring = JString::from(uri_string);

    let uri_text: String = env.get_string(&uri_jstring).ok()?.into();

    Some(uri_text)
}

/// Extracts the tree document ID from a SAF tree URI.
///
/// Useful for diagnostics, but not a reliable conversion to a
/// filesystem path.
pub fn tree_document_id(env: &mut JNIEnv<'_>, uri: &JObject<'_>) -> Option<String> {
    let docs_contract = env.find_class("android/provider/DocumentsContract").ok()?;

    let doc_id = env
        .call_static_method(
            docs_contract,
            "getTreeDocumentId",
            "(Landroid/net/Uri;)Ljava/lang/String;",
            &[JValue::Object(uri)],
        )
        .ok()?
        .l()
        .ok()?;

    if doc_id.is_null() {
        return None;
    }

    let doc_id_jstring = JString::from(doc_id);

    let doc_id_text: String = env.get_string(&doc_id_jstring).ok()?.into();

    Some(doc_id_text)
}
