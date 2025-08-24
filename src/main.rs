use dioxus::prelude::*;

mod types;
mod data;
mod services;
mod components;

use types::*;
use services::*;
use components::layout::{Header, Sidebar};

fn main() {
    // Initialize the Dioxus desktop app
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // Include CSS
        document::Link { rel: "stylesheet", href: asset!("assets/main.css") }
        
        MainApp {}
    }
}

#[component]
fn MainApp() -> Element {
    // Initialize app state
    let mut service_manager = use_signal(|| ServiceManager::new());
    let mut is_loading = use_signal(|| true);
    let mut load_error = use_signal(|| Option::<String>::None);
    let mut translations = use_signal(|| Vec::<Translation>::new());
    let mut books = use_signal(|| Vec::<Book>::new());
    let mut verses = use_signal(|| Vec::<Verse>::new());
    let mut selected_translation = use_signal(|| Option::<Translation>::None);
    let mut selected_book = use_signal(|| Option::<Book>::None);
    let mut selected_chapter = use_signal(|| 1u32);
    
    // Theme management
    let mut is_dark_theme = use_signal(|| false);
    let mut is_sidebar_open = use_signal(|| false);
    let mut search_query = use_signal(|| String::new());
    let mut is_parallel_view = use_signal(|| false);
    let mut secondary_translation = use_signal(|| Option::<Translation>::None);
    let mut parallel_verses = use_signal(|| Vec::<Vec<Verse>>::new());
    let mut bookmarks = use_signal(|| Vec::<Bookmark>::new());
    let mut highlights = use_signal(|| Vec::<TextHighlight>::new());
    let mut selected_verse_for_highlight = use_signal(|| Option::<Verse>::None);
    let mut is_highlight_menu_open = use_signal(|| false);
    let mut zoom_level = use_signal(|| 1.0f32);
    
    // Modal states
    let mut is_settings_open = use_signal(|| false);
    let mut is_bookmarks_open = use_signal(|| false);
    let mut is_search_open = use_signal(|| false);

    // Initialize data on first load
    use_effect(move || {
        spawn(async move {
            let mut manager = service_manager.write();
            match manager.initialize().await {
                Ok(()) => {
                    let loaded_translations = manager.bible().get_translations().clone();
                    translations.set(loaded_translations.clone());
                    
                    // Set default translation (KJV if available)
                    if let Some(kjv) = loaded_translations.iter().find(|t| t.id == "kjv") {
                        selected_translation.set(Some(kjv.clone()));
                        
                        // Set up secondary translation for parallel view (Tamil if available)
                        if let Some(tamil) = loaded_translations.iter().find(|t| t.id == "tamil") {
                            secondary_translation.set(Some(tamil.clone()));
                        }
                        
                        // Load books for KJV
                        match manager.bible().load_books("kjv").await {
                            Ok(loaded_books) => {
                                books.set(loaded_books.clone());
                                
                                // Set default book (Genesis if available)
                                if let Some(genesis) = loaded_books.first() {
                                    selected_book.set(Some(genesis.clone()));
                                    
                                    // Load verses for Genesis chapter 1
                                    match manager.bible().load_verses("kjv", genesis.id, 1).await {
                                        Ok(loaded_verses) => {
                                            verses.set(loaded_verses.clone());
                                            
                                            // Also load parallel verses if we have a secondary translation
                                            if let Some(secondary_trans) = secondary_translation.read().clone() {
                                                match manager.bible().load_verses(&secondary_trans.id, genesis.id, 1).await {
                                                    Ok(secondary_verses) => {
                                                        let mut current_parallel = parallel_verses.write();
                                                        current_parallel.clear();
                                                        current_parallel.push(loaded_verses);
                                                        current_parallel.push(secondary_verses);
                                                    },
                                                    Err(_) => {
                                                        // Secondary translation failed, just use primary
                                                        let mut current_parallel = parallel_verses.write();
                                                        current_parallel.clear();
                                                        current_parallel.push(loaded_verses);
                                                    }
                                                }
                                            } else {
                                                // No secondary translation, just store primary
                                                let mut current_parallel = parallel_verses.write();
                                                current_parallel.clear();
                                                current_parallel.push(loaded_verses);
                                            }
                                            
                                            is_loading.set(false);
                                        },
                                        Err(e) => {
                                            load_error.set(Some(format!("Failed to load verses: {}", e)));
                                            is_loading.set(false);
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                load_error.set(Some(format!("Failed to load books: {}", e)));
                                is_loading.set(false);
                            }
                        }
                    }
                },
                Err(e) => {
                    load_error.set(Some(format!("Failed to initialize: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    });

    // Handle book selection
    let mut on_book_select = move |book: Book| {
        selected_book.set(Some(book.clone()));
        selected_chapter.set(1);
        
        if let Some(translation) = selected_translation.read().clone() {
            let translation_id = translation.id.clone();
            let book_id = book.id;
            spawn(async move {
                let mut manager = service_manager.write();
                match manager.bible().load_verses(&translation_id, book_id, 1).await {
                    Ok(loaded_verses) => verses.set(loaded_verses),
                    Err(e) => load_error.set(Some(format!("Failed to load verses: {}", e))),
                }
            });
        }
    };

    // Handle chapter navigation
    let mut on_chapter_change = move |chapter: u32| {
        selected_chapter.set(chapter);
        
        let translation_opt = selected_translation.read().clone();
        let book_opt = selected_book.read().clone();
        let is_parallel = *is_parallel_view.read();
        let secondary_trans_opt = secondary_translation.read().clone();
        
        if let (Some(translation), Some(book)) = (translation_opt, book_opt) {
            let translation_id = translation.id.clone();
            let book_id = book.id;
            
            // Load primary translation
            spawn(async move {
                let mut manager = service_manager.write();
                match manager.bible().load_verses(&translation_id, book_id, chapter).await {
                    Ok(loaded_verses) => {
                        verses.set(loaded_verses.clone());
                        
                        // If parallel view is enabled, also store in parallel array
                        if is_parallel {
                            let mut current_parallel = parallel_verses.write();
                            current_parallel.clear();
                            current_parallel.push(loaded_verses);
                            
                            // Add empty slot for secondary translation
                            if current_parallel.len() < 2 {
                                current_parallel.push(Vec::new());
                            }
                        }
                    },
                    Err(e) => load_error.set(Some(format!("Failed to load verses: {}", e))),
                }
            });
            
            // Load secondary translation if parallel view is enabled
            if is_parallel && secondary_trans_opt.is_some() {
                let secondary_trans = secondary_trans_opt.unwrap();
                let secondary_id = secondary_trans.id.clone();
                spawn(async move {
                    let mut manager = service_manager.write();
                    match manager.bible().load_verses(&secondary_id, book_id, chapter).await {
                        Ok(loaded_verses) => {
                            let mut current_parallel = parallel_verses.write();
                            // Ensure we have at least 2 slots
                            while current_parallel.len() < 2 {
                                current_parallel.push(Vec::new());
                            }
                            current_parallel[1] = loaded_verses;
                        },
                        Err(e) => load_error.set(Some(format!("Failed to load parallel verses: {}", e))),
                    }
                });
            }
        }
    };

    // Additional event handlers for new features
    let mut on_translation_select = move |translation_id: String| {
        let translation = translations.read().iter().find(|t| t.id == translation_id).cloned();
        if let Some(translation) = translation {
            selected_translation.set(Some(translation.clone()));
            
            // Reload current book/chapter with new translation
            if let Some(book) = selected_book.read().clone() {
                let translation_id = translation.id.clone();
                let book_id = book.id;
                let chapter = *selected_chapter.read();
                spawn(async move {
                    let mut manager = service_manager.write();
                    match manager.bible().load_verses(&translation_id, book_id, chapter).await {
                        Ok(loaded_verses) => verses.set(loaded_verses),
                        Err(e) => load_error.set(Some(format!("Failed to load verses: {}", e))),
                    }
                });
            }
        }
    };

    let mut on_search = move || {
        let query = search_query.read().clone();
        if query.trim().is_empty() {
            return;
        }
        
        let translation_opt = selected_translation.read().clone();
        if let Some(translation) = translation_opt {
            let translation_id = translation.id.clone();
            spawn(async move {
                let mut manager = service_manager.write();
                match manager.bible().search_verses(&translation_id, &query).await {
                    Ok(search_results) => {
                        verses.set(search_results);
                        // Clear book/chapter selection to indicate we're in search mode
                        selected_book.set(None);
                        selected_chapter.set(0);
                    }
                    Err(e) => load_error.set(Some(format!("Search failed: {}", e))),
                }
            });
        }
    };

    let mut on_toggle_parallel_view = move || {
        let current_state = *is_parallel_view.read();
        is_parallel_view.set(!current_state);
        
        // Load parallel verses when enabled
        if !current_state && secondary_translation.read().is_some() {
            let secondary_trans = secondary_translation.read().clone().unwrap();
            let primary_trans = selected_translation.read().clone();
            let book_opt = selected_book.read().clone();
            let chapter = *selected_chapter.read();
            
            if let (Some(book), Some(primary_translation)) = (book_opt, primary_trans) {
                let secondary_id = secondary_trans.id.clone();
                let primary_id = primary_translation.id.clone();
                let book_id = book.id;
                
                spawn(async move {
                    let mut manager = service_manager.write();
                    
                    // Load both primary and secondary verses
                    let primary_result = manager.bible().load_verses(&primary_id, book_id, chapter).await;
                    let secondary_result = manager.bible().load_verses(&secondary_id, book_id, chapter).await;
                    
                    match (primary_result, secondary_result) {
                        (Ok(primary_verses), Ok(secondary_verses)) => {
                            let mut current_parallel = parallel_verses.write();
                            current_parallel.clear();
                            current_parallel.push(primary_verses);
                            current_parallel.push(secondary_verses);
                        },
                        (Ok(primary_verses), Err(_)) => {
                            // Secondary failed, just use primary
                            let mut current_parallel = parallel_verses.write();
                            current_parallel.clear();
                            current_parallel.push(primary_verses);
                            current_parallel.push(Vec::new()); // Empty secondary
                        },
                        (Err(e), _) => {
                            load_error.set(Some(format!("Failed to load parallel verses: {}", e)));
                        }
                    }
                });
            }
        }
    };

    let mut on_prev_chapter = move || {
        let current_chapter = *selected_chapter.read();
        if current_chapter > 1 {
            on_chapter_change(current_chapter - 1);
        } else {
            // TODO: Navigate to previous book's last chapter
        }
    };

    let mut on_next_chapter = move || {
        let current_chapter = *selected_chapter.read();
        if let Some(book) = selected_book.read().clone() {
            if current_chapter < book.chapter_count {
                on_chapter_change(current_chapter + 1);
            } else {
                // TODO: Navigate to next book's first chapter
            }
        }
    };

    let mut on_zoom_in = move || {
        let current_zoom = *zoom_level.read();
        if current_zoom < 3.0 {
            zoom_level.set((current_zoom + 0.1).min(3.0));
        }
    };

    let mut on_zoom_out = move || {
        let current_zoom = *zoom_level.read();
        if current_zoom > 0.5 {
            zoom_level.set((current_zoom - 0.1).max(0.5));
        }
    };

    let mut on_reset_zoom = move || {
        zoom_level.set(1.0);
    };

    let mut on_open_bookmarks = move || {
        is_bookmarks_open.set(true);
    };

    let mut on_open_settings = move || {
        is_settings_open.set(true);
    };

    let mut on_clear_search = move || {
        search_query.set(String::new());
        // Reload the current book/chapter if we have one
        if let (Some(book), Some(translation)) = (selected_book.read().clone(), selected_translation.read().clone()) {
            let chapter = *selected_chapter.read();
            if chapter > 0 {
                on_chapter_change(chapter);
            } else {
                // If no chapter selected, go to chapter 1
                on_chapter_change(1);
            }
        }
    };

    let mut on_verse_click = move |verse: Verse| {
        selected_verse_for_highlight.set(Some(verse));
        is_highlight_menu_open.set(true);
    };

    let mut on_highlight_verse = move |color: HighlightColor| {
        let verse_opt = selected_verse_for_highlight.read().clone();
        if let Some(verse) = verse_opt {
            let highlight = TextHighlight {
                id: format!("{}-{}-{}", verse.book_id, verse.chapter, verse.verse),
                user_id: None,
                translation_id: verse.translation_id.clone(),
                book_id: verse.book_id,
                chapter: verse.chapter,
                verse: verse.verse,
                text: verse.text.clone(),
                color,
                start_index: 0,
                end_index: verse.text.len(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            highlights.write().push(highlight);
            is_highlight_menu_open.set(false);
            selected_verse_for_highlight.set(None);
        }
    };

    let get_verse_highlight_color = |verse: &Verse| -> Option<HighlightColor> {
        highlights
            .read()
            .iter()
            .find(|h| {
                h.book_id == verse.book_id
                    && h.chapter == verse.chapter
                    && h.verse == verse.verse
            })
            .map(|h| h.color.clone())
    };
    
    
    rsx! {
        // Include CSS
        document::Link { rel: "stylesheet", href: asset!("assets/main.css") }
        
        div { 
            class: format!("min-h-screen flex bg-primary text-primary theme-transition {}", 
                if *is_dark_theme.read() { 
                    "theme-dark" 
                } else { 
                    "theme-light" 
                }
            ),
            
            // Sidebar - Always visible on desktop, toggleable on mobile
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
                on_open_bookmarks: move |_| on_open_bookmarks(),
                on_open_settings: move |_| on_open_settings()
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
                    on_search: move |_| on_search(),
                    is_parallel_view: *is_parallel_view.read(),
                    on_toggle_parallel_view: move |_| on_toggle_parallel_view(),
                    has_secondary_translation: secondary_translation.read().is_some(),
                    selected_book: selected_book.read().clone(),
                    selected_chapter: *selected_chapter.read(),
                    selected_translation: selected_translation.read().clone(),
                    on_prev_chapter: move |_| on_prev_chapter(),
                    on_next_chapter: move |_| on_next_chapter(),
                    zoom_level: *zoom_level.read(),
                    on_zoom_in: move |_| on_zoom_in(),
                    on_zoom_out: move |_| on_zoom_out(),
                    on_reset_zoom: move |_| on_reset_zoom(),
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
                            p { class: "text-lg text-gray-600", "Loading StudyBible..." }
                            p { class: "text-sm text-gray-500", "Please wait while we prepare your Bible study experience." }
                        }
                    }
                }
                // Error state
                else if let Some(error) = &*load_error.read() {
                    div {
                        class: "flex-1 flex items-center justify-center",
                        div {
                            class: "text-center max-w-md mx-auto bg-red-50 border border-red-200 rounded-lg p-6",
                            h3 { class: "text-lg font-semibold text-red-800 mb-2", "Error Loading Bible Data" }
                            p { class: "text-sm text-red-600", "{error}" }
                        }
                    }
                }
                // Main content
                else {
                    main {
                        class: "flex-1 overflow-auto",
                    
                    // Bible reader area
                    div {
                        class: "flex-1 p-6 overflow-y-auto",
                        if let (Some(book), Some(translation)) = (&*selected_book.read(), &*selected_translation.read()) {
                            div {
                                class: "max-w-4xl mx-auto",
                                
                                // Chapter header
                                div {
                                    class: "mb-8 pb-6 border-b border-gray-200 dark:border-gray-700",
                                    h1 {
                                        class: "text-3xl font-bold text-gray-800 dark:text-gray-200 mb-2",
                                        if *selected_chapter.read() == 0 && !search_query.read().is_empty() {
                                            "Search Results: '{search_query.read()}'"
                                        } else {
                                            "{book.name} {selected_chapter.read()}"
                                        }
                                    }
                                    div {
                                        class: "flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400",
                                        span { "{translation.abbreviation}" }
                                        span { "•" }
                                        span { "{translation.name}" }
                                        if *selected_chapter.read() == 0 && !search_query.read().is_empty() {
                                            span { "•" }
                                            span { "{verses.len()} results" }
                                        }
                                    }
                                    
                                    // Chapter navigation
                                    div {
                                        class: "flex items-center gap-2 mt-4",
                                        button {
                                            class: "px-3 py-1 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded text-sm transition-colors",
                                            disabled: *selected_chapter.read() <= 1,
                                            onclick: move |_| on_chapter_change(*selected_chapter.read() - 1),
                                            "Previous"
                                        }
                                        span {
                                            class: "px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-300 rounded text-sm font-medium",
                                            "Chapter {selected_chapter.read()}"
                                        }
                                        button {
                                            class: "px-3 py-1 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded text-sm transition-colors",
                                            disabled: *selected_chapter.read() >= book.chapter_count,
                                            onclick: move |_| on_chapter_change(*selected_chapter.read() + 1),
                                            "Next"
                                        }
                                    }
                                }
                                
                                // Verses - Support both single and parallel view
                                if *is_parallel_view.read() && secondary_translation.read().is_some() {
                                    // Parallel view layout
                                    div {
                                        class: "grid grid-cols-1 lg:grid-cols-2 gap-8",
                                        style: format!("font-size: {}rem; line-height: 1.6;", 1.125 * *zoom_level.read()),
                                        
                                        // Primary translation column
                                        div {
                                            class: "space-y-4",
                                            div {
                                                class: "mb-4 pb-2 border-b border-gray-200 dark:border-gray-700",
                                                h3 {
                                                    class: "text-lg font-semibold text-gray-800 dark:text-gray-200",
                                                    "{translation.abbreviation} - {translation.name}"
                                                }
                                            }
                                            for verse in verses.iter() {
                                                div {
                                                    key: "{verse.id}-primary",
                                                    class: format!("flex gap-4 items-start group hover:bg-gray-50 dark:hover:bg-gray-800 hover:bg-opacity-50 rounded-lg p-3 transition-colors cursor-pointer {}",
                                                        if let Some(highlight_color) = get_verse_highlight_color(&verse) {
                                                            match highlight_color {
                                                                HighlightColor::Yellow => "bg-yellow-100 dark:bg-yellow-900 bg-opacity-30",
                                                                HighlightColor::Green => "bg-green-100 dark:bg-green-900 bg-opacity-30",
                                                                HighlightColor::Blue => "bg-blue-100 dark:bg-blue-900 bg-opacity-30",
                                                                HighlightColor::Pink => "bg-pink-100 dark:bg-pink-900 bg-opacity-30",
                                                                HighlightColor::Purple => "bg-purple-100 dark:bg-purple-900 bg-opacity-30",
                                                            }
                                                        } else {
                                                            ""
                                                        }
                                                    ),
                                                    onclick: {
                                                        let verse = verse.clone();
                                                        move |_| on_verse_click(verse.clone())
                                                    },
                                                    div {
                                                        class: "flex-shrink-0 w-8 h-8 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-300 rounded-full flex items-center justify-center text-sm font-medium",
                                                        "{verse.verse}"
                                                    }
                                                    p {
                                                        class: "text-gray-800 dark:text-gray-200 leading-relaxed",
                                                        "{verse.text}"
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Secondary translation column
                                        div {
                                            class: "space-y-4",
                                            if let Some(secondary_trans) = &*secondary_translation.read() {
                                                div {
                                                    class: "mb-4 pb-2 border-b border-gray-200 dark:border-gray-700",
                                                    h3 {
                                                        class: "text-lg font-semibold text-gray-800 dark:text-gray-200",
                                                        "{secondary_trans.abbreviation} - {secondary_trans.name}"
                                                    }
                                                }
                                                if parallel_verses.read().len() > 1 {
                                                    for verse in parallel_verses.read()[1].iter() {
                                                        div {
                                                            key: "{verse.id}-secondary",
                                                            class: format!("flex gap-4 items-start group hover:bg-gray-50 dark:hover:bg-gray-800 hover:bg-opacity-50 rounded-lg p-3 transition-colors cursor-pointer {}",
                                                                if let Some(highlight_color) = get_verse_highlight_color(&verse) {
                                                                    match highlight_color {
                                                                        HighlightColor::Yellow => "bg-yellow-100 dark:bg-yellow-900 bg-opacity-30",
                                                                        HighlightColor::Green => "bg-green-100 dark:bg-green-900 bg-opacity-30",
                                                                        HighlightColor::Blue => "bg-blue-100 dark:bg-blue-900 bg-opacity-30",
                                                                        HighlightColor::Pink => "bg-pink-100 dark:bg-pink-900 bg-opacity-30",
                                                                        HighlightColor::Purple => "bg-purple-100 dark:bg-purple-900 bg-opacity-30",
                                                                    }
                                                                } else {
                                                                    ""
                                                                }
                                                            ),
                                                            onclick: {
                                                                let verse = verse.clone();
                                                                move |_| on_verse_click(verse.clone())
                                                            },
                                                            div {
                                                                class: "flex-shrink-0 w-8 h-8 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-300 rounded-full flex items-center justify-center text-sm font-medium",
                                                                "{verse.verse}"
                                                            }
                                                            p {
                                                                class: "text-gray-800 dark:text-gray-200 leading-relaxed",
                                                                "{verse.text}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Single view layout
                                    div {
                                        class: "space-y-4",
                                        style: format!("font-size: {}rem; line-height: 1.6;", 1.125 * *zoom_level.read()),
                                        for verse in verses.iter() {
                                            div {
                                                key: "{verse.id}",
                                                class: format!("flex gap-4 items-start group hover:bg-gray-50 dark:hover:bg-gray-800 hover:bg-opacity-50 rounded-lg p-3 transition-colors cursor-pointer {}",
                                                    if let Some(highlight_color) = get_verse_highlight_color(&verse) {
                                                        match highlight_color {
                                                            HighlightColor::Yellow => "bg-yellow-100 dark:bg-yellow-900 bg-opacity-30",
                                                            HighlightColor::Green => "bg-green-100 dark:bg-green-900 bg-opacity-30",
                                                            HighlightColor::Blue => "bg-blue-100 dark:bg-blue-900 bg-opacity-30",
                                                            HighlightColor::Pink => "bg-pink-100 dark:bg-pink-900 bg-opacity-30",
                                                            HighlightColor::Purple => "bg-purple-100 dark:bg-purple-900 bg-opacity-30",
                                                        }
                                                    } else {
                                                        ""
                                                    }
                                                ),
                                                onclick: {
                                                    let verse = verse.clone();
                                                    move |_| on_verse_click(verse.clone())
                                                },
                                                div {
                                                    class: "flex-shrink-0 w-8 h-8 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-300 rounded-full flex items-center justify-center text-sm font-medium",
                                                    "{verse.verse}"
                                                }
                                                p {
                                                    class: "text-gray-800 dark:text-gray-200 leading-relaxed",
                                                    "{verse.text}"
                                                }
                                            }
                                        }
                                        
                                        if verses.is_empty() {
                                            div {
                                                class: "text-center py-12",
                                                p { class: "text-gray-500", "No verses found for this chapter." }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            div {
                                class: "max-w-4xl mx-auto text-center py-12",
                                h2 {
                                    class: "text-2xl font-bold text-gray-800 mb-4",
                                    "Select a Bible Book"
                                }
                                p {
                                    class: "text-gray-600",
                                    "Choose a book from the sidebar to begin reading."
                                }
                            }
                        }
                    }

                    // Footer
                    footer {
                        class: "bg-gray-100 dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 p-4 text-center text-sm text-gray-600 dark:text-gray-400",
                        "Built with Dioxus 0.6 • StudyBible v0.1.0 • {translations.len()} translation(s) available"
                    }
                    }
                }
            }

            // Highlight Menu Modal
            if *is_highlight_menu_open.read() {
                div {
                    class: "fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm",
                    onclick: move |_| is_highlight_menu_open.set(false),
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 max-w-md mx-4",
                        onclick: move |e| e.stop_propagation(),
                        if let Some(verse) = &*selected_verse_for_highlight.read() {
                            div {
                                h3 {
                                    class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4",
                                    "Highlight Verse"
                                }
                                div {
                                    class: "mb-4 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg",
                                    p {
                                        class: "text-sm text-gray-600 dark:text-gray-400 mb-2",
                                        if let Some(book) = &*selected_book.read() {
                                            "{book.name} {verse.chapter}:{verse.verse}"
                                        } else {
                                            "Book {verse.book_id} {verse.chapter}:{verse.verse}"
                                        }
                                    }
                                    p {
                                        class: "text-gray-900 dark:text-gray-100",
                                        "{verse.text}"
                                    }
                                }
                                div {
                                    class: "space-y-2",
                                    p {
                                        class: "text-sm font-medium text-gray-700 dark:text-gray-300 mb-3",
                                        "Choose highlight color:"
                                    }
                                    div {
                                        class: "grid grid-cols-5 gap-2",
                                        button {
                                            class: "w-12 h-12 bg-yellow-300 hover:bg-yellow-400 rounded-lg transition-colors",
                                            onclick: move |_| on_highlight_verse(HighlightColor::Yellow),
                                            title: "Yellow"
                                        }
                                        button {
                                            class: "w-12 h-12 bg-green-300 hover:bg-green-400 rounded-lg transition-colors",
                                            onclick: move |_| on_highlight_verse(HighlightColor::Green),
                                            title: "Green"
                                        }
                                        button {
                                            class: "w-12 h-12 bg-blue-300 hover:bg-blue-400 rounded-lg transition-colors",
                                            onclick: move |_| on_highlight_verse(HighlightColor::Blue),
                                            title: "Blue"
                                        }
                                        button {
                                            class: "w-12 h-12 bg-red-300 hover:bg-red-400 rounded-lg transition-colors",
                                            onclick: move |_| on_highlight_verse(HighlightColor::Pink),
                                            title: "Red"
                                        }
                                        button {
                                            class: "w-12 h-12 bg-purple-300 hover:bg-purple-400 rounded-lg transition-colors",
                                            onclick: move |_| on_highlight_verse(HighlightColor::Purple),
                                            title: "Purple"
                                        }
                                    }
                                }
                                div {
                                    class: "flex justify-end mt-6 gap-3",
                                    button {
                                        class: "px-4 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200",
                                        onclick: move |_| is_highlight_menu_open.set(false),
                                        "Cancel"
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
