use dioxus::prelude::*;
use crate::types::*;

#[component]
pub fn Sidebar(
    is_sidebar_open: bool,
    is_dark: bool,
    books: Vec<Book>,
    bookmarks: Vec<Bookmark>,
    translations: Vec<Translation>,
    selected_book: Option<Book>,
    selected_translation: Option<Translation>,
    on_select_book: EventHandler<Book>,
    on_select_translation: EventHandler<String>,
    on_open_bookmarks: EventHandler<()>,
    on_open_settings: EventHandler<()>,
    // New: open translations modal
    // For now reuse settings button to open translations modal from parent
    on_toggle_sidebar: EventHandler<()>,
) -> Element {
    // Separate books by testament
    let old_testament_books: Vec<&Book> = books.iter().filter(|book| book.testament == Testament::OT).collect();
    let new_testament_books: Vec<&Book> = books.iter().filter(|book| book.testament == Testament::NT).collect();

    rsx! {
        // Clean vertical sidebar
        aside {
            class: format!(
                "w-80 h-screen bg-secondary border-r border-primary flex flex-col theme-transition lg:sticky lg:top-0 {}",
                if is_sidebar_open {
                    // Mobile: slide-over overlay; Desktop: static sticky sidebar
                    "fixed inset-y-0 left-0 w-80 z-40 lg:static"
                } else {
                    // Closed on all sizes
                    "hidden"
                }
            ),
            
            // Header with translation selector and collapse button
            div {
                class: "p-4 border-b border-primary bg-secondary theme-transition flex items-center gap-3",
                // Collapse button (desktop and mobile)
                button {
                    class: if is_dark { "p-2 rounded-lg hover:bg-tertiary text-secondary" } else { "p-2 rounded-lg hover:bg-tertiary text-secondary" },
                    title: if is_sidebar_open { "Hide sidebar" } else { "Show sidebar" },
                    onclick: move |_| on_toggle_sidebar.call(()),
                    "✕"
                }
                div {
                    class: "space-y-3 grow",
                    
                    label {
                        class: "block text-xs font-semibold uppercase tracking-wider text-gray-700 dark:text-gray-300",
                        "TRANSLATION"
                    }
                    select {
                        class: "w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900 dark:text-white",
                        value: selected_translation.as_ref().map(|t| t.id.as_str()).unwrap_or(""),
                        onchange: move |evt| {
                            on_select_translation.call(evt.value());
                        },
                        for translation in &translations {
                            option {
                                key: "{translation.id}",
                                value: "{translation.id}",
                                "{translation.abbreviation} - {translation.name}"
                            }
                        }
                    }
                }
            }
            
            // Scrollable books section
            div {
                class: "flex-1 overflow-y-auto bg-secondary theme-transition",
                
                // Old Testament section
                if !old_testament_books.is_empty() {
                    div {
                        class: "p-4",
                        
                        // Section header
                        div {
                            class: "flex items-center gap-2 mb-4",
                            span {
                                class: "text-blue-600 dark:text-blue-400",
                                "📖"
                            }
                            h3 {
                                class: "text-sm font-bold uppercase tracking-wider text-gray-800 dark:text-gray-200",
                                "OLD TESTAMENT"
                            }
                        }
                        
                        // Books list - VERTICAL layout
                        div {
                            class: "space-y-1",
                            for book in old_testament_books {
                                button {
                                    key: "{book.id}",
                                    class: if selected_book.as_ref().map(|b| b.id) == Some(book.id) {
                                        "w-full flex items-center justify-between px-3 py-2 text-left text-sm bg-blue-100 dark:bg-blue-800 text-blue-900 dark:text-blue-100 rounded border-l-4 border-blue-500"
                                    } else {
                                        "w-full flex items-center justify-between px-3 py-2 text-left text-sm bg-white dark:bg-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-500 rounded border-l-4 border-transparent hover:border-gray-300 dark:hover:border-gray-400 transition-colors"
                                    },
                                    onclick: {
                                        let book = book.clone();
                                        move |_| on_select_book.call(book.clone())
                                    },
                                    span { 
                                        class: "font-medium",
                                        "{book.name}" 
                                    }
                                    span {
                                        class: "text-xs text-gray-500 dark:text-gray-400",
                                        "{book.chapter_count}"
                                    }
                                }
                            }
                        }
                    }
                }
                
                // New Testament section
                if !new_testament_books.is_empty() {
                    div {
                        class: "p-4",
                        
                        // Section header
                        div {
                            class: "flex items-center gap-2 mb-4",
                            span {
                                class: "text-purple-600 dark:text-purple-400",
                                "✝️"
                            }
                            h3 {
                                class: "text-sm font-bold uppercase tracking-wider text-gray-800 dark:text-gray-200",
                                "NEW TESTAMENT"
                            }
                        }
                        
                        // Books list - VERTICAL layout
                        div {
                            class: "space-y-1",
                            for book in new_testament_books {
                                button {
                                    key: "{book.id}",
                                    class: if selected_book.as_ref().map(|b| b.id) == Some(book.id) {
                                        "w-full flex items-center justify-between px-3 py-2 text-left text-sm bg-purple-100 dark:bg-purple-800 text-purple-900 dark:text-purple-100 rounded border-l-4 border-purple-500"
                                    } else {
                                        "w-full flex items-center justify-between px-3 py-2 text-left text-sm bg-white dark:bg-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-500 rounded border-l-4 border-transparent hover:border-gray-300 dark:hover:border-gray-400 transition-colors"
                                    },
                                    onclick: {
                                        let book = book.clone();
                                        move |_| on_select_book.call(book.clone())
                                    },
                                    span { 
                                        class: "font-medium",
                                        "{book.name}" 
                                    }
                                    span {
                                        class: "text-xs text-gray-500 dark:text-gray-400",
                                        "{book.chapter_count}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Footer with action buttons
            div {
                class: "border-t border-gray-300 dark:border-gray-600 p-4 bg-white dark:bg-gray-900 space-y-2",
                button {
                    class: "w-full flex items-center gap-3 px-3 py-2 text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors",
                    onclick: move |_| on_open_bookmarks.call(()),
                    span { "🔖" }
                    span { "Bookmarks" }
                    if bookmarks.len() > 0 {
                        span {
                            class: "ml-auto text-xs bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300 px-2 py-1 rounded-full",
                            "{bookmarks.len()}"
                        }
                    }
                }
                button {
                    class: "w-full flex items-center gap-3 px-3 py-2 text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors",
                    onclick: move |_| on_open_settings.call(()),
                    span { "⬇️" }
                    span { "Translations" }
                }
            }
        }
    }
}