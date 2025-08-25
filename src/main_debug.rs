use dioxus::prelude::*;

mod types;
mod data;
mod services;
mod components;

use types::*;
use services::*;
use components::layout::{Header, Sidebar};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Start with minimal state - just test the basic UI without complex data loading
    let is_dark_theme = use_signal(|| false);
    let is_sidebar_open = use_signal(|| true);
    
    // Simple test data instead of complex loading
    let translations = use_signal(|| vec![
        Translation {
            id: "KJV".to_string(),
            name: "King James Version".to_string(),
            abbreviation: "KJV".to_string(),
            language: "en".to_string(),
        }
    ]);
    
    let books = use_signal(|| vec![
        Book {
            id: 1,
            name: "Genesis".to_string(),
            testament: Testament::OT,
            chapter_count: 50,
        },
        Book {
            id: 2, 
            name: "Exodus".to_string(),
            testament: Testament::OT,
            chapter_count: 40,
        }
    ]);
    
    let selected_book = use_signal(|| Some(Book {
        id: 1,
        name: "Genesis".to_string(),
        testament: Testament::OT,
        chapter_count: 50,
    }));
    
    let selected_translation = use_signal(|| Some(Translation {
        id: "KJV".to_string(),
        name: "King James Version".to_string(),
        abbreviation: "KJV".to_string(),
        language: "en".to_string(),
    }));
    
    let bookmarks = use_signal(|| Vec::<Bookmark>::new());

    rsx! {
        div { 
            class: "min-h-screen flex bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100",
            
            // Sidebar
            Sidebar {
                is_sidebar_open: *is_sidebar_open.read(),
                is_dark: *is_dark_theme.read(),
                books: books.read().clone(),
                bookmarks: bookmarks.read().clone(),
                translations: translations.read().clone(),
                selected_book: selected_book.read().clone(),
                selected_translation: selected_translation.read().clone(),
                on_select_book: move |book: Book| selected_book.set(Some(book)),
                on_select_translation: move |id: String| {
                    // Simple translation selection
                },
                on_open_bookmarks: move |_| {},
                on_open_settings: move |_| {}
            }

            // Main content area
            div {
                class: "flex-1 flex flex-col",
                
                // Simple header
                div {
                    class: "p-4 border-b border-gray-200 dark:border-gray-700",
                    h1 {
                        class: "text-xl font-bold",
                        "StudyBible - Debug Version"
                    }
                    button {
                        class: "ml-4 px-3 py-1 text-sm bg-blue-500 text-white rounded",
                        onclick: move |_| is_sidebar_open.set(!*is_sidebar_open.read()),
                        if *is_sidebar_open.read() { "Hide Sidebar" } else { "Show Sidebar" }
                    }
                }
                
                // Simple main content
                div {
                    class: "flex-1 p-8",
                    if let Some(book) = &*selected_book.read() {
                        div {
                            h2 {
                                class: "text-2xl font-bold mb-4",
                                "{book.name}"
                            }
                            p {
                                class: "text-gray-600 dark:text-gray-400",
                                "This is a debug version to test if the basic UI works without data loading."
                            }
                            p {
                                class: "mt-2 text-gray-600 dark:text-gray-400",
                                "Book: {book.name}, Chapters: {book.chapter_count}"
                            }
                        }
                    } else {
                        div {
                            class: "text-center text-gray-500",
                            "No book selected"
                        }
                    }
                }
            }
        }
    }
}