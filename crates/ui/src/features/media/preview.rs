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
            class: "shrink-0 py-2 px-4 bg-bg-secondary border-t border-border flex items-center gap-3",

            // Thumbnail
            {
                let img_src = format!("data:{};base64,{}", media.mimetype, media.data);
                rsx! {
                    div {
                        class: "w-15 h-15 rounded-lg overflow-hidden bg-bg-tertiary shrink-0",
                        img {
                            src: "{img_src}",
                            class: "w-full h-full object-cover",
                        }
                    }
                }
            }

            // File info
            div {
                class: "flex-1 min-w-0",
                p {
                    class: "m-0 text-text-white text-sm overflow-hidden text-ellipsis whitespace-nowrap",
                    "{media.filename}"
                }
                p {
                    class: "mt-1 mb-0 text-text-muted text-xs",
                    "{media.mimetype}"
                }
            }

            // Remove button
            button {
                onclick: move |_| on_remove.call(()),
                class: "bg-error border-none rounded-full w-8 h-8 text-text-white cursor-pointer flex items-center justify-center shrink-0 hover:opacity-80 transition-opacity",
                "x"
            }
        }
    }
}
