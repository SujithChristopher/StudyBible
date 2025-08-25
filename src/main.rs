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
    // Core app state
    let mut is_dark_theme = use_signal(|| false);
    let mut is_sidebar_open = use_signal(|| true);
    let mut is_loading = use_signal(|| true);
    let mut load_error = use_signal(|| None::<String>);
    
    // Bible data state
    let mut translations = use_signal(|| Vec::<Translation>::new());
    let mut books = use_signal(|| Vec::<Book>::new());
    let mut verses = use_signal(|| Vec::<Verse>::new());
    let mut selected_book = use_signal(|| None::<Book>);
    let mut selected_translation = use_signal(|| None::<Translation>);
    let mut selected_chapter = use_signal(|| 1);
    let bookmarks = use_signal(|| Vec::<Bookmark>::new());
    let _highlights = use_signal(|| Vec::<TextHighlight>::new());
    
    // UI state
    let mut zoom_level = use_signal(|| 1.0);
    let mut is_parallel_view = use_signal(|| false);
    let mut search_query = use_signal(|| String::new());
    
    // Initialize data on startup
    use_effect(move || {
        spawn(async move {
            let mut bible_service = BibleService::new();
            
            match bible_service.load_translations().await {
                Ok(trans_list) => {
                    translations.set(trans_list.clone());
                    if let Some(first_translation) = trans_list.first() {
                        selected_translation.set(Some(first_translation.clone()));
                        
                        // Load books for the first translation
                        match bible_service.load_books(&first_translation.id).await {
                            Ok(books_list) => {
                                books.set(books_list.clone());
                                if let Some(first_book) = books_list.first() {
                                    selected_book.set(Some(first_book.clone()));
                                    
                                    // Load first chapter
                                    match bible_service.load_verses(&first_translation.id, first_book.id, 1).await {
                                        Ok(verses_list) => {
                                            verses.set(verses_list);
                                            is_loading.set(false);
                                        }
                                        Err(e) => {
                                            load_error.set(Some(format!("Failed to load verses: {}", e)));
                                            is_loading.set(false);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                load_error.set(Some(format!("Failed to load books: {}", e)));
                                is_loading.set(false);
                            }
                        }
                    }
                }
                Err(e) => {
                    load_error.set(Some(format!("Failed to load translations: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    });

    // Event handlers
    let mut on_book_select = move |book: Book| {
        selected_book.set(Some(book.clone()));
        selected_chapter.set(1);
        
        let translation_id = selected_translation.read().as_ref().map(|t| t.id.clone());
        if let Some(trans_id) = translation_id {
            spawn(async move {
                let mut bible_service = BibleService::new();
                match bible_service.load_verses(&trans_id, book.id, 1).await {
                    Ok(verses_list) => verses.set(verses_list),
                    Err(e) => load_error.set(Some(format!("Failed to load verses: {}", e))),
                }
            });
        }
    };

    let mut on_translation_select = move |translation_id: String| {
        if let Some(translation) = translations.read().iter().find(|t| t.id == translation_id) {
            selected_translation.set(Some(translation.clone()));
            
            spawn(async move {
                let mut bible_service = BibleService::new();
                match bible_service.load_books(&translation_id).await {
                    Ok(books_list) => {
                        books.set(books_list.clone());
                        if let Some(first_book) = books_list.first() {
                            selected_book.set(Some(first_book.clone()));
                            selected_chapter.set(1);
                        }
                    }
                    Err(e) => load_error.set(Some(format!("Failed to load books: {}", e))),
                }
            });
        }
    };

    rsx! {
        // Include CSS
        document::Link { rel: "stylesheet", href: asset!("assets/tailwind.css") }
        document::Link { rel: "stylesheet", href: asset!("assets/main.css") }
        
        // Dark mode scope wrapper so all children inherit `.dark`
        div {
            class: if *is_dark_theme.read() { "dark" } else { "" },

            // App root container
            div { 
                class: "min-h-screen flex bg-primary text-primary theme-transition",

                // Sidebar
                Sidebar {
                is_sidebar_open: *is_sidebar_open.read(),
                is_dark: *is_dark_theme.read(),
                books: books.read().clone(),
                bookmarks: bookmarks.read().clone(),
                translations: translations.read().clone(),
                selected_book: selected_book.read().clone(),
                selected_translation: selected_translation.read().clone(),
                on_select_book: move |book: Book| on_book_select(book),
                on_select_translation: move |id: String| on_translation_select(id),
                on_open_bookmarks: move |_| {},
                on_open_settings: move |_| {}
                }

                // Mobile sidebar overlay
                if *is_sidebar_open.read() {
                    div {
                        class: "fixed inset-0 bg-black bg-opacity-50 z-30 lg:hidden",
                        onclick: move |_| is_sidebar_open.set(false)
                    }
                }

                // Main content area
                div {
                    class: "flex-1 flex flex-col",
                
                // Header
                Header {
                    is_sidebar_open: *is_sidebar_open.read(),
                    set_is_sidebar_open: move |open: bool| is_sidebar_open.set(open),
                    search_query: search_query.read().clone(),
                    set_search_query: move |query: String| search_query.set(query),
                    on_search: move |_| {},
                    is_parallel_view: *is_parallel_view.read(),
                    on_toggle_parallel_view: move |_| {
                        let current = *is_parallel_view.read();
                        is_parallel_view.set(!current);
                    },
                    has_secondary_translation: false,
                    selected_book: selected_book.read().clone(),
                    selected_chapter: *selected_chapter.read(),
                    selected_translation: selected_translation.read().clone(),
                    on_prev_chapter: move |_| {},
                    on_next_chapter: move |_| {},
                    zoom_level: *zoom_level.read(),
                    on_zoom_in: move |_| {
                        let current = *zoom_level.read();
                        zoom_level.set((current + 0.1).min(2.0));
                    },
                    on_zoom_out: move |_| {
                        let current = *zoom_level.read();
                        zoom_level.set((current - 0.1).max(0.5));
                    },
                    on_reset_zoom: move |_| zoom_level.set(1.0),
                    is_dark: *is_dark_theme.read(),
                    set_is_dark: move |dark: bool| is_dark_theme.set(dark)
                }

                // Loading state
                if *is_loading.read() {
                    div {
                        class: "flex-1 flex items-center justify-center",
                        div {
                            class: "text-center",
                            div { class: "animate-spin w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full mx-auto mb-4" }
                            p { class: "text-lg text-gray-600 dark:text-gray-400", "Loading StudyBible..." }
                        }
                    }
                }
                // Error state
                else if let Some(error) = &*load_error.read() {
                    div {
                        class: "flex-1 flex items-center justify-center",
                        div {
                            class: "text-center max-w-md mx-auto bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-700 rounded-lg p-6",
                            h3 { class: "text-lg font-semibold text-red-800 dark:text-red-200 mb-2", "Error Loading Bible Data" }
                            p { class: "text-sm text-red-600 dark:text-red-300", "{error}" }
                        }
                    }
                }
                // Main content
                else {
                    main {
                        class: "flex-1 overflow-auto bg-secondary theme-transition",
                        div {
                            class: "max-w-4xl mx-auto p-8",
                            
                            if let Some(book) = &*selected_book.read() {
                                div {
                                    // Chapter header
                                    div {
                                        class: "mb-8 pb-6 border-b border-primary",
                                        h1 {
                                            class: "text-3xl font-bold text-primary mb-2",
                                            "{book.name} {selected_chapter.read()}"
                                        }
                                        if let Some(translation) = &*selected_translation.read() {
                                            p {
                                                class: "text-secondary",
                                                "{translation.name}"
                                            }
                                        }
                                    }
                                    
                                    // Verses
                                    div {
                                        class: "space-y-4",
                                        style: format!("font-size: {}rem; line-height: 1.6;", 1.125 * *zoom_level.read()),
                                        for verse in verses.read().iter() {
                                            div {
                                                key: "{verse.id}",
                                                class: "flex gap-4 items-start group hover:bg-tertiary rounded-lg p-4 transition-colors theme-transition",
                                                div {
                                                    class: "flex-shrink-0 w-8 h-8 bg-blue-500 text-white rounded-lg flex items-center justify-center text-sm font-bold",
                                                    "{verse.verse}"
                                                }
                                                p {
                                                    class: "text-primary leading-relaxed",
                                                    "{verse.text}"
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                div {
                                    class: "text-center py-12",
                                    h2 {
                                        class: "text-xl font-semibold text-gray-600 dark:text-gray-400 mb-4",
                                        "Welcome to StudyBible"
                                    }
                                    p {
                                        class: "text-gray-500 dark:text-gray-500",
                                        "Select a book from the sidebar to begin reading."
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
}