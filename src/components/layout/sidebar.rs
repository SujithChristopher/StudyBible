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
) -> Element {
    
    let is_old_testament = |book: &Book| book.testament == Testament::OT;
    let is_new_testament = |book: &Book| book.testament == Testament::NT;

    let old_testament_books: Vec<&Book> = books.iter().filter(|book| is_old_testament(book)).collect();
    let new_testament_books: Vec<&Book> = books.iter().filter(|book| is_new_testament(book)).collect();

    rsx! {
        aside {
            class: format!("w-72 h-screen bg-secondary border-r border-primary flex-shrink-0 {}",
                if is_sidebar_open {
                    "fixed lg:relative z-40 lg:z-auto"
                } else {
                    "hidden lg:block"
                }
            ),
            
            div {
                class: "flex h-full flex-col",
                
                // Translation Selector Section
                div {
                    class: "p-4 border-b border-primary flex-shrink-0",
                    div {
                        class: "space-y-2",
                        label {
                            class: "text-xs font-semibold uppercase tracking-wider text-tertiary",
                            "Translation"
                        }
                        select {
                            class: "w-full px-3 py-2 bg-tertiary border-primary border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-primary",
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

                // Books Section
                div {
                    class: "flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600 scrollbar-track-transparent p-6",
                    nav {
                        class: "space-y-6",
                        
                        // Old Testament Section
                        div {
                            div {
                                class: "mb-3 flex items-center space-x-2 px-3 py-2",
                                div {
                                    class: "h-4 w-4 text-amber-600 dark:text-amber-400",
                                    "📖"
                                }
                                span {
                                    class: "text-sm font-bold uppercase tracking-wider text-amber-700 dark:text-amber-300",
                                    "Old Testament"
                                }
                                div {
                                    class: "flex-1 h-px bg-gradient-to-r from-amber-300 to-transparent opacity-50"
                                }
                            }
                            div {
                                class: "grid grid-cols-1 gap-1",
                                for book in old_testament_books {
                                    button {
                                        key: "{book.id}",
                                        class: if selected_book.as_ref().map(|b| b.id) == Some(book.id) {
                                            "group w-full rounded-lg px-3 py-2 text-left text-sm font-medium transition-all duration-200 bg-amber-100 dark:bg-amber-900 text-amber-700 dark:text-amber-300 shadow-sm ring-1 ring-amber-500 ring-opacity-30 border-l-2 border-amber-500"
                                        } else {
                                            "group w-full rounded-lg px-3 py-2 text-left text-sm font-medium transition-all duration-200 text-gray-600 dark:text-gray-300 hover:bg-amber-50 dark:hover:bg-amber-900 hover:bg-opacity-20 hover:text-amber-700 dark:hover:text-amber-300"
                                        },
                                        onclick: {
                                            let book = book.clone();
                                            move |_| on_select_book.call(book.clone())
                                        },
                                        div {
                                            class: "flex items-center justify-between",
                                            span {
                                                class: "truncate",
                                                "{book.name}"
                                            }
                                            div {
                                                class: "flex items-center gap-2",
                                                span {
                                                    class: "text-xs text-gray-500 dark:text-gray-400",
                                                    "{book.chapter_count}"
                                                }
                                                if selected_book.as_ref().map(|b| b.id) == Some(book.id) {
                                                    div {
                                                        class: "h-1.5 w-1.5 rounded-full bg-amber-500"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // New Testament Section  
                        div {
                            div {
                                class: "mb-3 flex items-center space-x-2 px-3 py-2",
                                div {
                                    class: "h-4 w-4 text-blue-600 dark:text-blue-400",
                                    "📖"
                                }
                                span {
                                    class: "text-sm font-bold uppercase tracking-wider text-blue-700 dark:text-blue-300",
                                    "New Testament"
                                }
                                div {
                                    class: "flex-1 h-px bg-gradient-to-r from-blue-300 to-transparent opacity-50"
                                }
                            }
                            div {
                                class: "grid grid-cols-1 gap-1",
                                for book in new_testament_books {
                                    button {
                                        key: "{book.id}",
                                        class: if selected_book.as_ref().map(|b| b.id) == Some(book.id) {
                                            "group w-full rounded-lg px-3 py-2 text-left text-sm font-medium transition-all duration-200 bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 shadow-sm ring-1 ring-blue-500 ring-opacity-30 border-l-2 border-blue-500"
                                        } else {
                                            "group w-full rounded-lg px-3 py-2 text-left text-sm font-medium transition-all duration-200 text-gray-600 dark:text-gray-300 hover:bg-blue-50 dark:hover:bg-blue-900 hover:bg-opacity-20 hover:text-blue-700 dark:hover:text-blue-300"
                                        },
                                        onclick: {
                                            let book = book.clone();
                                            move |_| on_select_book.call(book.clone())
                                        },
                                        div {
                                            class: "flex items-center justify-between",
                                            span {
                                                class: "truncate",
                                                "{book.name}"
                                            }
                                            div {
                                                class: "flex items-center gap-2",
                                                span {
                                                    class: "text-xs text-gray-500 dark:text-gray-400",
                                                    "{book.chapter_count}"
                                                }
                                                if selected_book.as_ref().map(|b| b.id) == Some(book.id) {
                                                    div {
                                                        class: "h-1.5 w-1.5 rounded-full bg-blue-500"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Footer with Bookmarks and Settings
                div {
                    class: "border-t border-gray-200 dark:border-gray-700 p-6 flex-shrink-0",
                    div {
                        class: "space-y-2",
                        button {
                            class: "w-full flex items-center justify-start space-x-3 px-4 py-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors",
                            onclick: move |_| on_open_bookmarks.call(()),
                            div {
                                class: "h-4 w-4",
                                "🔖"
                            }
                            span {
                                class: "text-sm text-gray-700 dark:text-gray-300",
                                "Bookmarks ({bookmarks.len()})"
                            }
                        }
                        button {
                            class: "w-full flex items-center justify-start space-x-3 px-4 py-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors",
                            onclick: move |_| on_open_settings.call(()),
                            div {
                                class: "h-4 w-4",
                                "⚙️"
                            }
                            span {
                                class: "text-sm text-gray-700 dark:text-gray-300",
                                "Settings"
                            }
                        }
                    }
                }
            }
        }
    }
}