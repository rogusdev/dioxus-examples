use dioxus::prelude::*;

use futures_util::{AsyncWriteExt, StreamExt};

use wasm_bindgen_futures::JsFuture;
use wasm_streams::writable::WritableStream;
use web_sys::wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use async_zip::base::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};

#[wasm_bindgen]
extern "C" {
    // Dioxus.toml script: "https://cdn.jsdelivr.net/npm/streamsaver@2.0.6/StreamSaver.min.js"
    #[wasm_bindgen(js_namespace = streamSaver, js_name = createWriteStream)]
    fn create_write_stream(filename: &str) -> web_sys::WritableStream;
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut msg = use_signal(|| String::new());

    rsx! {
        document::Script { src: "https://cdn.jsdelivr.net/npm/streamsaver@2.0.6/StreamSaver.min.js" }

        button {
            onclick: move |_| {
                let filename_urls = vec![
                    ("small.jpg".to_owned(), "https://picsum.photos/200/300".to_owned()),
                    ("big.jpg".to_owned(), "https://picsum.photos/600/800".to_owned()),
                    ("medium.jpg".to_owned(), "https://picsum.photos/300/400".to_owned()),
                ];

                async move {
                    let filename = "files.zip".to_owned();
                    match save_file_stream(filename).await {
                        Ok(None) => {}
                        Ok(Some((filename, stream))) => {
                            msg.set(format!("Writing to {filename}..."));

                            let report = move |filename| {
                                msg.set(format!("Added {filename} to zip"));
                            };

                            if let Err(e) = write_zip(stream, filename_urls, report).await {
                                msg.set(format!("{e}"));
                            } else {
                                msg.set(format!("Written to {filename}!"));
                            }
                        }
                        Err(e) => msg.set(format!("Failed getting file stream: {e:?}")),
                    }
                }
            },

            "Zip"
        }

        div {
            "{msg()}"
        }
    }
}

async fn save_file_stream(
    filename: String,
) -> Result<Option<(String, web_sys::WritableStream)>, String> {
    let opts = web_sys::SaveFilePickerOptions::new();
    opts.set_suggested_name(Some(&filename));

    // cargo add web_sys -F Window,etc
    if let Some(promise) =
        web_sys::window().and_then(|w| w.show_save_file_picker_with_options(&opts).ok())
    {
        // cargo add wasm-bindgen-futures
        match JsFuture::from(promise).await {
            Ok(js_value) => match js_value.dyn_into::<web_sys::FileSystemFileHandle>() {
                Ok(handle) => {
                    let filename = handle.name();
                    let promise = handle.create_writable();
                    match JsFuture::from(promise).await {
                        Ok(js_value) => {
                            match js_value.dyn_into::<web_sys::FileSystemWritableFileStream>() {
                                Ok(stream) => Ok(Some((filename, stream.into()))),
                                Err(e) => {
                                    Err(format!("Failed to receive file write stream: {e:?}"))
                                }
                            }
                        }
                        Err(e) => Err(format!("Failed to promise file write stream: {e:?}")),
                    }
                }
                Err(e) => {
                    if let Some(dom_exception) = e.dyn_ref::<web_sys::DomException>() {
                        match dom_exception.name().as_str() {
                            "AbortError" => Err(format!(
                                "Selected non-file or inaccessible file: {}",
                                dom_exception.message()
                            )),
                            _ => Err(format!("Unhandled DOMException: {dom_exception:?}")),
                        }
                    } else {
                        Err(format!("Failed to convert to file handle: {e:?}"))
                    }
                }
            },
            Err(e) => {
                if let Some(dom_exception) = e.dyn_ref::<web_sys::DomException>() {
                    match dom_exception.name().as_str() {
                        "AbortError" => Ok(None),
                        _ => Err(format!("Unhandled DOMException: {dom_exception:?}")),
                    }
                } else {
                    Err(format!("Failed to finish promise for file handle: {e:?}"))
                }
            }
        }
    } else {
        // fallback to streamsaver
        // https://github.com/jimmywarting/StreamSaver.js
        let stream = create_write_stream(&filename);
        Ok(Some((filename, stream)))
    }
}

async fn write_zip<F>(
    stream: web_sys::WritableStream,
    filename_urls: Vec<(String, String)>,
    mut report: F,
) -> Result<(), String>
where
    F: FnMut(String),
{
    // cargo add reqwest -F stream
    let client = reqwest::Client::new();

    // cargo add wasm_streams
    let writable = WritableStream::from_raw(stream).into_async_write();

    // cargo add async_zip -F full-wasm
    let mut zip_writer = ZipFileWriter::new(writable);

    for (filename, url) in filename_urls {
        if let Ok(response) = client.get(&url).send().await {
            let entry = ZipEntryBuilder::new(filename.clone().into(), Compression::Stored);
            let mut writer = match zip_writer.write_entry_stream(entry).await {
                Ok(writer) => writer,
                Err(e) => {
                    Err(format!("Error creating entry for {filename}: {e:?}"))?
                }
            };

            let mut body = response.bytes_stream();
            // cargo add futures_util
            while let Some(chunk) = body.next().await {
                if let Ok(bytes) = chunk {
                    if let Err(e) = writer.write_all(&bytes).await {
                        Err(format!("Failed to write chunk for {filename}: {e:?}"))?
                    }
                }
            }

            if let Err(e) = writer.close().await {
                Err(format!("Failed to close {filename} entry writer: {e:?}"))?
            } else {
                report(filename);
            }
        } else {
            Err(format!(
                "Failed to get response for {filename} from url: {url}"
            ))?
        }
    }

    match zip_writer.close().await {
        Ok(mut writer) => {
            if let Err(e) = writer.close().await {
                Err(format!(
                    "Writer close failed, probably did not finish writing file! {e:?}"
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!("Zip write failed: {e:?}")),
    }
}
