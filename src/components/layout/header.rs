use dioxus::prelude::*;
use crate::types::*;

#[component]
pub fn Header(
    is_sidebar_open: bool,
    set_is_sidebar_open: EventHandler<bool>,
    search_query: String,
    set_search_query: EventHandler<String>,
    on_search: EventHandler<()>,
    is_parallel_view: bool,
    on_toggle_parallel_view: EventHandler<()>,
    has_secondary_translation: bool,
    secondary_translation: Option<Translation>,
    on_select_secondary_translation: EventHandler<String>,
    is_parallel_by_columns: bool,
    on_toggle_parallel_layout: EventHandler<()>,
    selected_book: Option<Book>,
    selected_chapter: u32,
    selected_translation: Option<Translation>,
    on_prev_chapter: EventHandler<()>,
    on_next_chapter: EventHandler<()>,
    zoom_level: f32,
    on_zoom_in: EventHandler<()>,
    on_zoom_out: EventHandler<()>,
    on_reset_zoom: EventHandler<()>,
    is_dark: bool,
    set_is_dark: EventHandler<bool>,
    on_select_chapter: EventHandler<u32>,
) -> Element {
    rsx! {
        header {
            class: "sticky top-0 z-40 w-full bg-secondary border-primary border-b backdrop-blur-xl theme-transition flex-shrink-0",
            
            div {
                class: "flex h-20 items-center px-4 sm:px-6",
                
                // Mobile menu toggle
                button {
                    class: "mr-4 lg:hidden p-2 rounded-lg bg-tertiary hover:bg-accent-secondary text-primary theme-transition",
                    onclick: move |_| set_is_sidebar_open.call(!is_sidebar_open),
                    if is_sidebar_open {
                        "✕"
                    } else {
                        "☰"
                    }
                }

                // Desktop sidebar toggle
                button {
                    class: "mr-4 hidden lg:inline-flex p-2 rounded-lg bg-tertiary hover:bg-accent-secondary text-primary theme-transition",
                    onclick: move |_| set_is_sidebar_open.call(!is_sidebar_open),
                    title: if is_sidebar_open { "Hide sidebar" } else { "Show sidebar" },
                    if is_sidebar_open { "⟨" } else { "☰" }
                }

                // Chapter navigation (desktop)
                div {
                    class: "flex items-center space-x-4",
                    div {
                        class: "hidden md:flex items-center gap-2 mr-2",
                        button {
                            class: format!("p-2 rounded-lg transition-colors {}",
                                if is_dark {
                                    "hover:bg-gray-800 text-gray-400"
                                } else {
                                    "hover:bg-gray-100 text-gray-600"
                                }
                            ),
                            title: "Previous chapter",
                            onclick: move |_| on_prev_chapter.call(()),
                            "◀"
                        }
                        button {
                            class: format!("p-2 rounded-lg transition-colors {}",
                                if is_dark {
                                    "hover:bg-gray-800 text-gray-400"
                                } else {
                                    "hover:bg-gray-100 text-gray-600"
                                }
                            ),
                            title: "Next chapter", 
                            onclick: move |_| on_next_chapter.call(()),
                            "▶"
                        }
                    }
                    // Chapter dropdown
                    if let Some(book) = &selected_book {
                        select {
                            class: format!("hidden md:block px-2 py-1 rounded border text-sm {}",
                                if is_dark { "bg-gray-800 border-gray-700 text-gray-100" } else { "bg-white border-gray-300 text-gray-900" }
                            ),
                            value: "{selected_chapter}",
                            onchange: move |evt| {
                                if let Ok(ch) = evt.value().parse::<u32>() {
                                    on_select_chapter.call(ch);
                                }
                            },
                            for ch in 1..=book.chapter_count {
                                option { value: "{ch}", "{ch}" }
                            }
                        }
                    }
                    
                    // Secondary translation selector (shown when parallel available)
                    if has_secondary_translation {
                        select {
                            class: format!("hidden md:block px-2 py-1 rounded border text-sm {}",
                                if is_dark { "bg-gray-800 border-gray-700 text-gray-100" } else { "bg-white border-gray-300 text-gray-900" }
                            ),
                            value: secondary_translation.as_ref().map(|t| t.id.as_str()).unwrap_or(""),
                            onchange: move |evt| on_select_secondary_translation.call(evt.value()),
                            option { value: "", "Single" }
                            option { value: "kjv", "KJV" }
                            option { value: "tamil", "Tamil" }
                            option { value: "niv", "NIV" }
                            option { value: "nkjv", "NKJV" }
                        }
                    }

                    // Logo and title
                    div {
                        class: "flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 via-blue-600 to-blue-700 text-white shadow-xl",
                        "📖"
                    }
                    div {
                        h1 {
                            class: format!("text-2xl font-bold bg-gradient-to-r bg-clip-text text-transparent {}",
                                if is_dark {
                                    "from-gray-100 via-blue-400 to-gray-100"
                                } else {
                                    "from-gray-900 via-blue-600 to-gray-900"
                                }
                            ),
                            "StudyBible"
                        }
                        p {
                            class: format!("text-xs font-medium {}",
                                if is_dark {
                                    "text-gray-400"
                                } else {
                                    "text-gray-500"
                                }
                            ),
                            if let Some(book) = &selected_book {
                                "{book.name} {selected_chapter}"
                            } else {
                                "Bible Study App"
                            }
                        }
                    }
                }

                // Right side controls
                div {
                    class: "flex flex-1 items-center justify-end space-x-4",
                    
                    // Search input
                    div {
                        class: "w-full max-w-lg",
                        div {
                            class: "relative group",
                            div {
                                class: "absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-gray-400 transition-colors group-focus-within:text-blue-500",
                                "🔍"
                            }
                            if !search_query.is_empty() {
                                button {
                                    class: "absolute right-4 top-1/2 h-5 w-5 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors",
                                    onclick: move |_| {
                                        set_search_query.call(String::new());
                                    },
                                    title: "Clear search",
                                    "✕"
                                }
                            }
                            input {
                                r#type: "search",
                                placeholder: if let Some(translation) = &selected_translation {
                                    format!("Search {}...", translation.abbreviation)
                                } else {
                                    "Search...".to_string()
                                },
                                value: "{search_query}",
                                class: format!("h-12 w-full rounded-2xl pl-12 pr-4 border focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder:text-gray-400 {}",
                                    if is_dark {
                                        "bg-gray-800 border-gray-700 text-white"
                                    } else {
                                        "bg-gray-50 border-gray-200 text-gray-900"
                                    }
                                ),
                                oninput: move |evt| set_search_query.call(evt.value()),
                                onkeydown: move |evt| {
                                    if evt.key() == Key::Enter {
                                        on_search.call(());
                                    }
                                }
                            }
                        }
                    }

                    // Zoom controls
                    div {
                        class: format!("hidden sm:flex items-center gap-2 px-3 py-2 rounded-lg {}",
                            if is_dark {
                                "bg-gray-800"
                            } else {
                                "bg-gray-50"
                            }
                        ),
                        button {
                            class: format!("p-1 rounded transition-colors {}",
                                if is_dark {
                                    "hover:bg-gray-700 text-gray-400"
                                } else {
                                    "hover:bg-gray-200 text-gray-600"
                                }
                            ),
                            onclick: move |_| on_zoom_out.call(()),
                            title: "Zoom out",
                            "−"
                        }
                        span {
                            class: format!("text-xs font-medium min-w-[3rem] text-center {}",
                                if is_dark {
                                    "text-gray-400"
                                } else {
                                    "text-gray-600"
                                }
                            ),
                            "{(zoom_level * 100.0) as i32}%"
                        }
                        button {
                            class: format!("p-1 rounded transition-colors {}",
                                if is_dark {
                                    "hover:bg-gray-700 text-gray-400"
                                } else {
                                    "hover:bg-gray-200 text-gray-600"
                                }
                            ),
                            onclick: move |_| on_zoom_in.call(()),
                            title: "Zoom in",
                            "+"
                        }
                        if zoom_level != 1.0 {
                            button {
                                class: "p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors text-gray-600 dark:text-gray-400 ml-1",
                                onclick: move |_| on_reset_zoom.call(()),
                                title: "Reset zoom",
                                "↻"
                            }
                        }
                    }

                    // Parallel view toggle
                    if has_secondary_translation {
                        div {
                            class: "flex items-center gap-2",
                            button {
                                class: if is_parallel_view {
                                    "px-3 py-2 bg-blue-500 text-white rounded-lg flex items-center gap-2 hover:bg-blue-600 transition-colors"
                                } else {
                                    format!("px-3 py-2 rounded-lg flex items-center gap-2 transition-colors {}",
                                        if is_dark {
                                            "bg-gray-800 text-gray-300 hover:bg-gray-700"
                                        } else {
                                            "bg-gray-100 text-gray-700 hover:bg-gray-200"
                                        }
                                    )
                                },
                                onclick: move |_| on_toggle_parallel_view.call(()),
                                "📖"
                                span {
                                    class: "hidden md:inline text-sm",
                                    if is_parallel_view { "Single View" } else { "Parallel View" }
                                }
                            }
                            // Layout toggle
                            if is_parallel_view {
                                button {
                                    class: format!("px-3 py-2 rounded-lg text-sm {}",
                                        if is_dark { "bg-gray-800 text-gray-300 hover:bg-gray-700" } else { "bg-gray-100 text-gray-700 hover:bg-gray-200" }
                                    ),
                                    onclick: move |_| on_toggle_parallel_layout.call(()),
                                    if is_parallel_by_columns { "Columns" } else { "Rows" }
                                }
                            }
                        }
                    }

                    // Dark mode toggle
                    button {
                        class: format!("p-3 rounded-lg transition-colors {}",
                            if is_dark {
                                "bg-gray-800 hover:bg-gray-700"
                            } else {
                                "bg-gray-100 hover:bg-gray-200"
                            }
                        ),
                        onclick: move |_| {
                            set_is_dark.call(!is_dark);
                        },
                        title: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
                        if is_dark { "☀️" } else { "🌙" }
                    }
                }
            }
        }
    }
}