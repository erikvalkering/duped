use jni::{
    JNIEnv,
    objects::{JObject, JString},
};

pub struct PickedFolder {
    pub tree_uri: String,
}

/// Extracts the selected directory URI from the picker result.
pub fn handle_folder_picker_result(
    env: &mut JNIEnv<'_>,
    data: JObject<'_>,
) -> Option<PickedFolder> {
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

    let tree_uri: String = env.get_string(&uri_jstring).ok()?.into();

    Some(PickedFolder { tree_uri })
}

/// Converts a primary-storage SAF tree URI into a local filesystem
/// path suitable for `walkdir`.
///
/// Examples:
///
/// ```text
/// content://com.android.externalstorage.documents/tree/primary%3ADownload
/// -> /storage/emulated/0/Download
///
/// content://com.android.externalstorage.documents/tree/primary%3ADocuments%2FMyFolder
/// -> /storage/emulated/0/Documents/MyFolder
/// ```
///
/// This works only for Android's external-storage provider and its
/// `primary` root. Other SAF providers may be cloud, USB, or otherwise
/// non-filesystem-backed, so their URIs cannot be converted reliably.
pub fn tree_uri_to_local_path(tree_uri: &str) -> Result<String, String> {
    const EXTERNAL_STORAGE_AUTHORITY: &str = "com.android.externalstorage.documents";

    let uri = url::Url::parse(tree_uri).map_err(|err| format!("Invalid tree URI: {err}"))?;

    if uri.scheme() != "content" {
        return Err(format!("Expected a content:// URI, got {}", uri.scheme()));
    }

    let Some(authority) = uri.host_str() else {
        return Err("Tree URI has no authority".to_string());
    };

    if authority != EXTERNAL_STORAGE_AUTHORITY {
        return Err(format!("Unsupported storage provider: {authority}"));
    }

    let path = uri.path();

    let Some(encoded_tree_id) = path.split_once("/tree/").map(|(_, rest)| rest) else {
        return Err("Tree URI does not contain a /tree/ segment".to_string());
    };

    let tree_id = percent_decode(encoded_tree_id)?;

    // Android's external-storage provider uses `root:relative/path`.
    let Some(relative_path) = tree_id.strip_prefix("primary:") else {
        return Err(format!(
            "Only the primary storage root is supported; got '{tree_id}'"
        ));
    };

    if relative_path.contains("..") {
        return Err("Refusing unsafe relative path in tree URI".to_string());
    }

    let relative_path = relative_path.trim_start_matches('/').trim_end_matches('/');

    if relative_path.is_empty() {
        Ok("/storage/emulated/0".to_string())
    } else {
        Ok(format!("/storage/emulated/0/{relative_path}"))
    }
}

fn percent_decode(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();

    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err("Invalid percent encoding".to_string());
            }

            let high = hex_digit(bytes[index + 1])?;
            let low = hex_digit(bytes[index + 2])?;

            output.push((high << 4) | low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(output).map_err(|err| format!("Invalid UTF-8 in tree ID: {err}"))
}

fn hex_digit(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(format!("Invalid hex digit in tree URI: {}", byte as char)),
    }
}
