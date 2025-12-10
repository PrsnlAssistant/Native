//! Media picker for cross-platform image selection

use super::types::SelectedMedia;

// Base64 encoding only needed for desktop and web implementations
#[cfg(any(feature = "desktop", target_arch = "wasm32"))]
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Pick an image file using the native file picker (desktop only)
/// Returns None if the user cancels or an error occurs
///
/// This only compiles when:
/// - Not targeting WASM (web has its own implementation)
/// - Not targeting Android (Android has its own implementation)
/// - The `desktop` feature is enabled (which enables `rfd`)
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android"), feature = "desktop"))]
pub async fn pick_image() -> Option<SelectedMedia> {
    use tracing::info;

    // Use rfd for native file picking
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
        .set_title("Select an image")
        .pick_file()
        .await?;

    info!("Selected file: {}", file.file_name());

    let data = file.read().await;
    let filename = file.file_name();

    // Determine mimetype from extension
    let mimetype = get_mimetype_from_filename(&filename);

    let base64_data = BASE64.encode(&data);

    Some(SelectedMedia {
        data: base64_data,
        mimetype,
        filename,
    })
}

/// Pick an image file on Android/Mobile (placeholder - not implemented)
/// Android requires JNI calls or Activity intents for file picking
/// This compiles for:
/// - Android target
/// - OR mobile feature without desktop feature (allows testing mobile on desktop)
#[cfg(any(
    target_os = "android",
    all(feature = "mobile", not(feature = "desktop"), not(target_arch = "wasm32"))
))]
pub async fn pick_image() -> Option<SelectedMedia> {
    use tracing::warn;
    warn!("Image picker not yet implemented for mobile platforms");
    // TODO: Implement Android file picker using JNI/Activity intents
    None
}

/// Pick an image file using web file input
#[cfg(target_arch = "wasm32")]
pub async fn pick_image() -> Option<SelectedMedia> {
    use wasm_bindgen::JsCast;
    use web_sys::{window, HtmlInputElement, File, FileReader};
    use tracing::info;

    let window = window()?;
    let document = window.document()?;

    // Create a hidden file input
    let input: HtmlInputElement = document
        .create_element("input")
        .ok()?
        .dyn_into()
        .ok()?;

    input.set_type("file");
    input.set_accept("image/*");

    // Trigger the file picker
    input.click();

    // Wait for file selection using a promise
    let (tx, rx) = futures_channel::oneshot::channel();
    let tx = std::rc::Rc::new(std::cell::RefCell::new(Some(tx)));

    let onchange = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
        if let Some(tx) = tx.borrow_mut().take() {
            let _ = tx.send(());
        }
    }) as Box<dyn FnMut(_)>);

    input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
    onchange.forget();

    // Wait for selection
    let _ = rx.await;

    let files = input.files()?;
    let file: File = files.get(0)?;

    let filename = file.name();
    let mimetype = file.type_();

    info!("Selected file: {} ({})", filename, mimetype);

    // Read file as base64
    let reader = FileReader::new().ok()?;
    reader.read_as_array_buffer(&file).ok()?;

    let (tx, rx) = futures_channel::oneshot::channel();
    let tx = std::rc::Rc::new(std::cell::RefCell::new(Some(tx)));
    let reader_clone = reader.clone();

    let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
        if let Some(tx) = tx.borrow_mut().take() {
            let result = reader_clone.result().ok();
            let _ = tx.send(result);
        }
    }) as Box<dyn FnMut(_)>);

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    let result = rx.await.ok()??;
    let array_buffer = result.dyn_into::<js_sys::ArrayBuffer>().ok()?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let data: Vec<u8> = uint8_array.to_vec();

    let base64_data = BASE64.encode(&data);

    let mimetype = if mimetype.is_empty() {
        get_mimetype_from_filename(&filename)
    } else {
        mimetype
    };

    Some(SelectedMedia {
        data: base64_data,
        mimetype,
        filename,
    })
}

/// Get MIME type from filename extension
/// Only used by desktop and web implementations
#[cfg(any(feature = "desktop", target_arch = "wasm32"))]
fn get_mimetype_from_filename(filename: &str) -> String {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }.to_string()
}
