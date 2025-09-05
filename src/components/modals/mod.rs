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

    let mut search_query = use_signal(|| String::new());
    let mut selected_language = use_signal(|| String::new());
    
    // Get unique languages for filter dropdown
    let languages = {
        let mut langs: Vec<(String, String)> = translations
            .iter()
            .map(|t| (t.language.clone(), t.language_name.clone().unwrap_or(t.language.clone())))
            .collect();
        langs.sort_by(|a, b| a.1.cmp(&b.1));
        langs.dedup();
        langs
    };

    // Group translations by language
    let grouped_translations = {
        let query = search_query.read().to_lowercase();
        let lang_filter = selected_language.read().clone();
        
        let mut filtered: Vec<Translation> = translations
            .iter()
            .filter(|t| {
                let matches_search = query.is_empty() || 
                    t.name.to_lowercase().contains(&query) ||
                    t.language_name.as_ref().unwrap_or(&t.language).to_lowercase().contains(&query);
                let matches_language = lang_filter.is_empty() || t.language == lang_filter;
                matches_search && matches_language
            })
            .cloned()
            .collect();
        
        filtered.sort_by(|a, b| {
            a.language_name.as_ref().unwrap_or(&a.language)
                .cmp(&b.language_name.as_ref().unwrap_or(&b.language))
                .then_with(|| a.name.cmp(&b.name))
        });
        
        filtered
    };

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
            div { class: "bg-secondary rounded-xl shadow-xl w-full max-w-4xl p-6 border border-primary",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-xl font-semibold text-primary", "Bible Translations Library" }
                    button { class: "px-4 py-2 rounded bg-tertiary hover:bg-accent-secondary text-sm", onclick: move |_| on_close.call(()), "Close" }
                }
                
                // Search and filter controls
                div { class: "mb-4 space-y-3",
                    div { class: "flex gap-3",
                        input {
                            class: "flex-1 px-3 py-2 border border-primary rounded bg-secondary text-primary placeholder-secondary text-sm",
                            placeholder: "Search translations or languages...",
                            value: "{search_query.read()}",
                            oninput: move |evt| search_query.set(evt.value())
                        }
                        select {
                            class: "px-3 py-2 border border-primary rounded bg-secondary text-primary text-sm min-w-[150px]",
                            value: "{selected_language.read()}",
                            onchange: move |evt| selected_language.set(evt.value()),
                            option { value: "", "All Languages" }
                            for (code, name) in languages {
                                option { value: "{code}", "{name}" }
                            }
                        }
                    }
                    div { class: "text-xs text-secondary",
                        "Found {grouped_translations.len()} translations"
                        if !search_query.read().is_empty() || !selected_language.read().is_empty() {
                            span { " (filtered)" }
                        }
                    }
                }
                
                // Translations list
                div { class: "max-h-[60vh] overflow-y-auto space-y-2",
                    for t in &grouped_translations {
                        TranslationRow { translation: t.clone() }
                    }
                    if grouped_translations.is_empty() {
                        div { class: "text-center py-8 text-secondary",
                            "No translations found matching your criteria"
                        }
                    }
                }
                
                div { class: "mt-4 pt-4 border-t border-primary text-xs text-secondary",
                    "Translations are downloaded from the Holy Bible collection and stored locally for offline reading."
                }
            }
        }
    }
}

#[component]
fn TranslationRow(translation: Translation) -> Element {
    let mut is_downloading = use_signal(|| false);
    let mut downloaded = use_signal(|| false);
    let mut download_error = use_signal(|| None::<String>);
    let name = translation.name.clone();
    let lang_label = translation.language_name.clone().unwrap_or(translation.language.clone());
    let abbr = translation.abbreviation.clone();
    let desc = translation.description.clone();
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
        div { class: "p-4 rounded-lg border border-primary bg-secondary hover:bg-tertiary transition-colors",
            div { class: "flex items-start justify-between gap-4",
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-2 mb-1",
                        h3 { class: "font-semibold text-primary truncate", "{name}" }
                        span { class: "text-xs px-2 py-0.5 rounded bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200 shrink-0", "{abbr}" }
                    }
                    div { class: "text-sm text-secondary mb-1", "{lang_label}" }
                    if !desc.is_empty() && desc != name {
                        div { class: "text-xs text-secondary opacity-75 line-clamp-2", "{desc}" }
                    }
                    if let Some(error) = download_error.read().as_ref() {
                        div { class: "text-xs text-red-600 dark:text-red-400 mt-2", "Error: {error}" }
                    }
                }
                
                div { class: "flex flex-col items-end gap-2",
                    if *downloaded.read() {
                        div { class: "flex items-center gap-2",
                            span { class: "text-xs px-3 py-1 rounded-full bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-200 font-medium", "✓ Downloaded" }
                        }
                    } else {
                        button {
                            class: "px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm font-medium transition-colors",
                            disabled: *is_downloading.read(),
                            onclick: move |_| {
                                let id = translation.id.clone();
                                is_downloading.set(true);
                                download_error.set(None);
                                spawn(async move {
                                    let svc = BibleService::new();
                                    let res = svc.download_translation_xml(&id).await;
                                    is_downloading.set(false);
                                    match res {
                                        Ok(_) => downloaded.set(true),
                                        Err(e) => download_error.set(Some(e)),
                                    }
                                });
                            },
                            if *is_downloading.read() { 
                                span { class: "flex items-center gap-2",
                                    div { class: "w-3 h-3 border border-white border-t-transparent rounded-full animate-spin" }
                                    "Downloading…"
                                }
                            } else { 
                                "Download" 
                            }
                        }
                    }
                }
            }
        }
    }
}