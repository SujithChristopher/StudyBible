use dioxus::prelude::*;
use crate::types::*;

#[component]
pub fn SettingsModal(
    is_open: bool,
    settings: AppSettings,
    on_close: EventHandler<()>,
    on_save: EventHandler<AppSettings>,
) -> Element {
    let mut local_settings = use_signal(|| settings.clone());
    let mut active_tab = use_signal(|| "appearance");

    // Reset local settings when modal opens
    use_effect(move || {
        if is_open {
            local_settings.set(settings.clone());
        }
    });

    if !is_open {
        return rsx! {};
    }

    rsx! {
        // Modal overlay
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm",
            onclick: move |_| on_close.call(()),

            // Modal content
            div {
                class: "bg-white dark:bg-gray-800 rounded-2xl shadow-2xl max-w-3xl w-full max-h-[90vh] overflow-hidden",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    class: "flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700",
                    h2 {
                        class: "text-2xl font-bold text-gray-900 dark:text-white",
                        "⚙️ Settings"
                    }
                    button {
                        class: "p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors text-gray-500 dark:text-gray-400",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                // Tabs
                div {
                    class: "flex border-b border-gray-200 dark:border-gray-700 px-6",
                    button {
                        class: if *active_tab.read() == "appearance" {
                            "px-4 py-3 border-b-2 border-blue-500 text-blue-600 dark:text-blue-400 font-medium"
                        } else {
                            "px-4 py-3 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
                        },
                        onclick: move |_| active_tab.set("appearance"),
                        "Appearance"
                    }
                    button {
                        class: if *active_tab.read() == "reading" {
                            "px-4 py-3 border-b-2 border-blue-500 text-blue-600 dark:text-blue-400 font-medium"
                        } else {
                            "px-4 py-3 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
                        },
                        onclick: move |_| active_tab.set("reading"),
                        "Reading"
                    }
                    button {
                        class: if *active_tab.read() == "advanced" {
                            "px-4 py-3 border-b-2 border-blue-500 text-blue-600 dark:text-blue-400 font-medium"
                        } else {
                            "px-4 py-3 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
                        },
                        onclick: move |_| active_tab.set("advanced"),
                        "Advanced"
                    }
                }

                // Content
                div {
                    class: "p-6 overflow-y-auto max-h-[60vh]",

                    // Appearance Tab
                    if *active_tab.read() == "appearance" {
                        div {
                            class: "space-y-6",

                            // Theme Selection
                            div {
                                label {
                                    class: "block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3",
                                    "Theme"
                                }
                                div {
                                    class: "grid grid-cols-2 md:grid-cols-4 gap-3",

                                    // Light theme
                                    button {
                                        class: if matches!(local_settings.read().theme, Theme::Light) {
                                            "p-4 rounded-lg border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                        } else {
                                            "p-4 rounded-lg border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().theme = Theme::Light;
                                        },
                                        div {
                                            class: "w-full h-12 bg-white border border-gray-300 rounded mb-2"
                                        }
                                        div {
                                            class: "text-sm font-medium text-gray-900 dark:text-white",
                                            "Light"
                                        }
                                    }

                                    // Dark theme
                                    button {
                                        class: if matches!(local_settings.read().theme, Theme::Dark) {
                                            "p-4 rounded-lg border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                        } else {
                                            "p-4 rounded-lg border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().theme = Theme::Dark;
                                        },
                                        div {
                                            class: "w-full h-12 bg-gray-900 border border-gray-700 rounded mb-2"
                                        }
                                        div {
                                            class: "text-sm font-medium text-gray-900 dark:text-white",
                                            "Dark"
                                        }
                                    }

                                    // Sepia theme
                                    button {
                                        class: if matches!(local_settings.read().theme, Theme::Sepia) {
                                            "p-4 rounded-lg border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                        } else {
                                            "p-4 rounded-lg border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().theme = Theme::Sepia;
                                        },
                                        div {
                                            class: "w-full h-12 bg-amber-50 border border-amber-200 rounded mb-2"
                                        }
                                        div {
                                            class: "text-sm font-medium text-gray-900 dark:text-white",
                                            "Sepia"
                                        }
                                    }

                                    // Nord theme
                                    button {
                                        class: if matches!(local_settings.read().theme, Theme::Nord) {
                                            "p-4 rounded-lg border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                        } else {
                                            "p-4 rounded-lg border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().theme = Theme::Nord;
                                        },
                                        div {
                                            class: "w-full h-12 bg-slate-800 border border-slate-600 rounded mb-2"
                                        }
                                        div {
                                            class: "text-sm font-medium text-gray-900 dark:text-white",
                                            "Nord"
                                        }
                                    }

                                    // Dracula theme
                                    button {
                                        class: if matches!(local_settings.read().theme, Theme::Dracula) {
                                            "p-4 rounded-lg border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                        } else {
                                            "p-4 rounded-lg border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().theme = Theme::Dracula;
                                        },
                                        div {
                                            class: "w-full h-12 bg-purple-950 border border-purple-800 rounded mb-2"
                                        }
                                        div {
                                            class: "text-sm font-medium text-gray-900 dark:text-white",
                                            "Dracula"
                                        }
                                    }

                                    // Ocean theme
                                    button {
                                        class: if matches!(local_settings.read().theme, Theme::Ocean) {
                                            "p-4 rounded-lg border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                        } else {
                                            "p-4 rounded-lg border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().theme = Theme::Ocean;
                                        },
                                        div {
                                            class: "w-full h-12 bg-cyan-900 border border-cyan-700 rounded mb-2"
                                        }
                                        div {
                                            class: "text-sm font-medium text-gray-900 dark:text-white",
                                            "Ocean"
                                        }
                                    }

                                    // Forest theme
                                    button {
                                        class: if matches!(local_settings.read().theme, Theme::Forest) {
                                            "p-4 rounded-lg border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                        } else {
                                            "p-4 rounded-lg border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().theme = Theme::Forest;
                                        },
                                        div {
                                            class: "w-full h-12 bg-green-950 border border-green-800 rounded mb-2"
                                        }
                                        div {
                                            class: "text-sm font-medium text-gray-900 dark:text-white",
                                            "Forest"
                                        }
                                    }

                                    // Auto theme
                                    button {
                                        class: if matches!(local_settings.read().theme, Theme::Auto) {
                                            "p-4 rounded-lg border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                        } else {
                                            "p-4 rounded-lg border-2 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().theme = Theme::Auto;
                                        },
                                        div {
                                            class: "w-full h-12 bg-gradient-to-r from-white to-gray-900 border border-gray-400 rounded mb-2"
                                        }
                                        div {
                                            class: "text-sm font-medium text-gray-900 dark:text-white",
                                            "Auto"
                                        }
                                    }
                                }
                            }

                            // Font Family
                            div {
                                label {
                                    class: "block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2",
                                    "Font Family"
                                }
                                select {
                                    class: "w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-white",
                                    value: format!("{:?}", local_settings.read().font_family).to_lowercase(),
                                    onchange: move |evt| {
                                        local_settings.write().font_family = match evt.value().as_str() {
                                            "serif" => FontFamily::Serif,
                                            "mono" => FontFamily::Mono,
                                            _ => FontFamily::Sans,
                                        };
                                    },
                                    option { value: "sans", "Sans Serif (Default)" }
                                    option { value: "serif", "Serif (Traditional)" }
                                    option { value: "mono", "Monospace (Code)" }
                                }
                            }

                            // Font Size
                            div {
                                label {
                                    class: "block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2",
                                    "Font Size: {local_settings.read().font_size}px"
                                }
                                input {
                                    r#type: "range",
                                    min: "14",
                                    max: "24",
                                    step: "1",
                                    value: "{local_settings.read().font_size}",
                                    class: "w-full",
                                    oninput: move |evt| {
                                        if let Ok(size) = evt.value().parse::<f32>() {
                                            local_settings.write().font_size = size;
                                        }
                                    }
                                }
                            }

                            // Line Height
                            div {
                                label {
                                    class: "block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2",
                                    "Line Height: {local_settings.read().line_height}"
                                }
                                input {
                                    r#type: "range",
                                    min: "1.2",
                                    max: "2.0",
                                    step: "0.1",
                                    value: "{local_settings.read().line_height}",
                                    class: "w-full",
                                    oninput: move |evt| {
                                        if let Ok(height) = evt.value().parse::<f32>() {
                                            local_settings.write().line_height = height;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Reading Tab
                    if *active_tab.read() == "reading" {
                        div {
                            class: "space-y-6",

                            // Verse Number Style
                            div {
                                label {
                                    class: "block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2",
                                    "Verse Number Style"
                                }
                                select {
                                    class: "w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-white",
                                    value: format!("{:?}", local_settings.read().verse_number_style).to_lowercase(),
                                    onchange: move |evt| {
                                        local_settings.write().verse_number_style = match evt.value().as_str() {
                                            "inline" => VerseNumberStyle::Inline,
                                            "hidden" => VerseNumberStyle::Hidden,
                                            _ => VerseNumberStyle::Badge,
                                        };
                                    },
                                    option { value: "badge", "Badge (Default)" }
                                    option { value: "inline", "Inline" }
                                    option { value: "hidden", "Hidden" }
                                }
                            }

                            // Parallel View Default Layout
                            div {
                                label {
                                    class: "block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2",
                                    "Parallel View Layout"
                                }
                                div {
                                    class: "flex gap-3",
                                    button {
                                        class: if local_settings.read().parallel_layout_columns {
                                            "flex-1 p-3 border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20 rounded-lg"
                                        } else {
                                            "flex-1 p-3 border-2 border-gray-200 dark:border-gray-700 rounded-lg hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().parallel_layout_columns = true;
                                        },
                                        div {
                                            class: "text-center",
                                            div { class: "text-2xl mb-1", "⫿" }
                                            div { class: "text-sm text-gray-700 dark:text-gray-300", "Columns" }
                                        }
                                    }
                                    button {
                                        class: if !local_settings.read().parallel_layout_columns {
                                            "flex-1 p-3 border-2 border-blue-500 bg-blue-50 dark:bg-blue-900/20 rounded-lg"
                                        } else {
                                            "flex-1 p-3 border-2 border-gray-200 dark:border-gray-700 rounded-lg hover:border-gray-300 dark:hover:border-gray-600"
                                        },
                                        onclick: move |_| {
                                            local_settings.write().parallel_layout_columns = false;
                                        },
                                        div {
                                            class: "text-center",
                                            div { class: "text-2xl mb-1", "☰" }
                                            div { class: "text-sm text-gray-700 dark:text-gray-300", "Rows" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Advanced Tab
                    if *active_tab.read() == "advanced" {
                        div {
                            class: "space-y-6",

                            div {
                                class: "p-4 bg-gray-50 dark:bg-gray-900 rounded-lg",
                                p {
                                    class: "text-sm text-gray-600 dark:text-gray-400",
                                    "Advanced settings coming soon: bookmarks export/import, data management, and more."
                                }
                            }
                        }
                    }
                }

                // Footer
                div {
                    class: "flex items-center justify-end gap-3 p-6 border-t border-gray-200 dark:border-gray-700",
                    button {
                        class: "px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-500 rounded-lg transition-colors",
                        onclick: move |_| {
                            local_settings.set(AppSettings::default());
                        },
                        "Reset to Defaults"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-500 text-white hover:bg-blue-600 rounded-lg transition-colors font-medium",
                        onclick: move |_| {
                            on_save.call(local_settings.read().clone());
                            on_close.call(());
                        },
                        "Save Changes"
                    }
                }
            }
        }
    }
}
