//! Media preview component

use dioxus::prelude::*;
use super::types::SelectedMedia;

/// Preview of selected media with remove button
#[component]
pub fn MediaPreview(
    media: SelectedMedia,
    on_remove: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "flex-shrink: 0; padding: 8px 16px; background: #1a1a2e; border-top: 1px solid #2d2d44; display: flex; align-items: center; gap: 12px;",

            // Thumbnail
            {
                let img_src = format!("data:{};base64,{}", media.mimetype, media.data);
                rsx! {
                    div {
                        style: "width: 60px; height: 60px; border-radius: 8px; overflow: hidden; background: #2d2d44; flex-shrink: 0;",
                        img {
                            src: "{img_src}",
                            style: "width: 100%; height: 100%; object-fit: cover;",
                        }
                    }
                }
            }

            // File info
            div {
                style: "flex: 1; min-width: 0;",
                p {
                    style: "margin: 0; color: white; font-size: 0.875rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{media.filename}"
                }
                p {
                    style: "margin: 4px 0 0 0; color: #888; font-size: 0.75rem;",
                    "{media.mimetype}"
                }
            }

            // Remove button
            button {
                onclick: move |_| on_remove.call(()),
                style: "background: #f44336; border: none; border-radius: 50%; width: 32px; height: 32px; color: white; cursor: pointer; display: flex; align-items: center; justify-content: center; flex-shrink: 0;",
                "x"
            }
        }
    }
}
