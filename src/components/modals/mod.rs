use dioxus::prelude::*;
use crate::types::Translation;
use crate::services::BibleService;

#[component]
pub fn TranslationsModal(
    is_open: bool,
    translations: Vec<Translation>,
    on_close: EventHandler<()>,
) -> Element {
    if !is_open { return rsx! { }; }

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
            div { class: "bg-secondary rounded-xl shadow-xl w-full max-w-2xl p-6 border border-primary",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-lg font-semibold text-primary", "Download Translations" }
                    button { class: "px-3 py-1 rounded bg-tertiary hover:bg-accent-secondary", onclick: move |_| on_close.call(()), "Close" }
                }
                div { class: "max-h-[60vh] overflow-y-auto space-y-2",
                    for t in translations {
                        TranslationRow { translation: t.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn TranslationRow(translation: Translation) -> Element {
    let mut is_downloading = use_signal(|| false);
    let mut downloaded = use_signal(|| false);
    let name = translation.name.clone();
    let lang_label = translation.language_name.clone().unwrap_or(translation.language.clone());
    let abbr = translation.abbreviation.clone();
    let id_for_status = translation.id.clone();

    use_effect(move || {
        let id = id_for_status.clone();
        spawn(async move {
            let svc = BibleService::new();
            match svc.is_translation_downloaded(&id).await {
                Ok(v) => downloaded.set(v),
                Err(_) => downloaded.set(false),
            }
        });
    });

    rsx! {
        div { class: "flex items-center justify-between p-3 rounded border border-primary bg-secondary",
            div { class: "min-w-0",
                div { class: "font-medium text-primary truncate", "{name}" }
                div { class: "text-xs text-secondary truncate", "{lang_label} • {abbr}" }
            }
            if *downloaded.read() {
                span { class: "text-xs px-2 py-1 rounded bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-200", "Downloaded" }
            } else {
                button {
                    class: "px-3 py-1 rounded bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50",
                    disabled: *is_downloading.read(),
                    onclick: move |_| {
                        let id = translation.id.clone();
                        is_downloading.set(true);
                        spawn(async move {
                            let svc = BibleService::new();
                            let res = svc.download_translation_xml(&id).await;
                            is_downloading.set(false);
                            if res.is_ok() { downloaded.set(true); }
                        });
                    },
                    if *is_downloading.read() { "Downloading…" } else { "Download" }
                }
            }
        }
    }
}