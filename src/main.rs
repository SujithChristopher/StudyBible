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
    let mut is_parallel_by_columns = use_signal(|| true);
    let mut secondary_translation = use_signal(|| None::<Translation>);
    let mut secondary_verses = use_signal(|| Vec::<Verse>::new());
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
        // refresh secondary if selected
        if let Some(sec) = &*secondary_translation.read() {
            let sec_id = sec.id.clone();
            let bid = book.id;
            let ch = 1u32;
            spawn(async move {
                let mut svc = BibleService::new();
                match svc.load_verses(&sec_id, bid, ch).await {
                    Ok(vs) => secondary_verses.set(vs),
                    Err(_) => secondary_verses.set(Vec::new()),
                }
            });
        } else {
            secondary_verses.set(Vec::new());
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
                            // Load verses for the newly selected translation/book
                            let tid = translation_id.clone();
                            let bid = first_book.id;
                            let ch = 1u32;
                            spawn(async move {
                                let mut svc = BibleService::new();
                                match svc.load_verses(&tid, bid, ch).await {
                                    Ok(vs) => verses.set(vs),
                                    Err(e) => load_error.set(Some(format!("{}", e))),
                                }
                            });
                            // refresh secondary if selected
                            if let Some(sec) = &*secondary_translation.read() {
                                let sec_id = sec.id.clone();
                                let bid2 = bid;
                                let ch2 = ch;
                                spawn(async move {
                                    let mut svc = BibleService::new();
                                    match svc.load_verses(&sec_id, bid2, ch2).await {
                                        Ok(vs) => secondary_verses.set(vs),
                                        Err(_) => secondary_verses.set(Vec::new()),
                                    }
                                });
                            } else {
                                secondary_verses.set(Vec::new());
                            }
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
                on_open_settings: move |_| {},
                on_toggle_sidebar: move |_| {
                    let current = *is_sidebar_open.read();
                    is_sidebar_open.set(!current)
                }
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
                    on_search: move |_| {
                        let trans_id_opt = selected_translation.read().as_ref().map(|t| t.id.clone());
                        let q = search_query.read().clone();
                        let books_snapshot = books.read().clone();
                        if let Some(tid) = trans_id_opt {
                            if !q.trim().is_empty() {
                                spawn(async move {
                                    let mut bible_service = BibleService::new();
                                    match bible_service.search_verses(&tid, &q).await {
                                        Ok(results) => {
                                            if let Some(v) = results.first() {
                                                if let Some(book) = books_snapshot.iter().find(|b| b.id == v.book_id).cloned() {
                                                    selected_book.set(Some(book.clone()));
                                                    selected_chapter.set(v.chapter);
                                                    let mut svc = BibleService::new();
                                                    match svc.load_verses(&tid, v.book_id, v.chapter).await {
                                                        Ok(list) => verses.set(list),
                                                        Err(err) => load_error.set(Some(format!("{}", err))),
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => load_error.set(Some(format!("Search failed: {}", e))),
                                    }
                                });
                            }
                        }
                    },
                    is_parallel_view: *is_parallel_view.read(),
                    on_toggle_parallel_view: move |_| {
                        let current = *is_parallel_view.read();
                        let new_val = !current;
                        is_parallel_view.set(new_val);
                        // If turning on parallel view without a secondary selected, auto-pick one and load it
                        if new_val && secondary_translation.read().is_none() {
                            // choose first different from primary
                            let primary_id_opt = selected_translation.read().as_ref().map(|t| t.id.clone());
                            if let Some(default_trans) = translations
                                .read()
                                .iter()
                                .find(|t| Some(t.id.clone()) != primary_id_opt)
                                .cloned()
                            {
                                // set selection
                                secondary_translation.set(Some(default_trans.clone()));
                                if let Some(book) = &*selected_book.read() {
                                    let bid = book.id;
                                    let ch = *selected_chapter.read();
                                    let sid = default_trans.id.clone();
                                    spawn(async move {
                                        let mut svc = BibleService::new();
                                        match svc.load_verses(&sid, bid, ch).await {
                                            Ok(vs) => secondary_verses.set(vs),
                                            Err(_) => secondary_verses.set(Vec::new()),
                                        }
                                    });
                                }
                            }
                        }
                    },
                    has_secondary_translation: true,
                    secondary_translation: secondary_translation.read().clone(),
                    on_select_secondary_translation: move |tid: String| {
                        if tid.is_empty() {
                            secondary_translation.set(None);
                            secondary_verses.set(Vec::new());
                        } else {
                            if let Some(book) = &*selected_book.read() {
                                let ch = *selected_chapter.read();
                                let bid = book.id;
                                let tid_clone = tid.clone();
                                spawn(async move {
                                    let mut svc = BibleService::new();
                                    match svc.load_verses(&tid_clone, bid, ch).await {
                                        Ok(vs) => secondary_verses.set(vs),
                                        Err(_) => secondary_verses.set(Vec::new()),
                                    }
                                });
                            }
                            if let Some(t) = translations.read().iter().find(|t| t.id == tid).cloned() {
                                secondary_translation.set(Some(t));
                            }
                        }
                    },
                    is_parallel_by_columns: *is_parallel_by_columns.read(),
                    on_toggle_parallel_layout: move |_| {
                        let v = *is_parallel_by_columns.read();
                        is_parallel_by_columns.set(!v);
                    },
                    selected_book: selected_book.read().clone(),
                    selected_chapter: *selected_chapter.read(),
                    selected_translation: selected_translation.read().clone(),
                    on_prev_chapter: move |_| {
                        if let Some(book) = &*selected_book.read() {
                            let current = *selected_chapter.read();
                            if current > 1 {
                                let new_ch = current - 1;
                                selected_chapter.set(new_ch);
                                if let Some(trans) = &*selected_translation.read() {
                                    let tid = trans.id.clone();
                                    let bid = book.id;
                                    let mut verses_sig = verses.clone();
                                    let mut load_err = load_error.clone();
                                    spawn(async move {
                                        let mut svc = BibleService::new();
                                        match svc.load_verses(&tid, bid, new_ch).await {
                                            Ok(vs) => verses_sig.set(vs),
                                            Err(e) => load_err.set(Some(format!("{}", e))),
                                        }
                                    });
                                }
                                // refresh secondary
                                if let Some(sec) = &*secondary_translation.read() {
                                    let sec_id = sec.id.clone();
                                    let bid2 = book.id;
                                    let ch2 = new_ch;
                                    spawn(async move {
                                        let mut svc = BibleService::new();
                                        match svc.load_verses(&sec_id, bid2, ch2).await {
                                            Ok(vs) => secondary_verses.set(vs),
                                            Err(_) => secondary_verses.set(Vec::new()),
                                        }
                                    });
                                } else {
                                    secondary_verses.set(Vec::new());
                                }
                            }
                        }
                    },
                    on_next_chapter: move |_| {
                        if let Some(book) = &*selected_book.read() {
                            let current = *selected_chapter.read();
                            if current < book.chapter_count {
                                let new_ch = current + 1;
                                selected_chapter.set(new_ch);
                                if let Some(trans) = &*selected_translation.read() {
                                    let tid = trans.id.clone();
                                    let bid = book.id;
                                    let mut verses_sig = verses.clone();
                                    let mut load_err = load_error.clone();
                                    spawn(async move {
                                        let mut svc = BibleService::new();
                                        match svc.load_verses(&tid, bid, new_ch).await {
                                            Ok(vs) => verses_sig.set(vs),
                                            Err(e) => load_err.set(Some(format!("{}", e))),
                                        }
                                    });
                                }
                                // refresh secondary
                                if let Some(sec) = &*secondary_translation.read() {
                                    let sec_id = sec.id.clone();
                                    let bid2 = book.id;
                                    let ch2 = new_ch;
                                    spawn(async move {
                                        let mut svc = BibleService::new();
                                        match svc.load_verses(&sec_id, bid2, ch2).await {
                                            Ok(vs) => secondary_verses.set(vs),
                                            Err(_) => secondary_verses.set(Vec::new()),
                                        }
                                    });
                                } else {
                                    secondary_verses.set(Vec::new());
                                }
                            }
                        }
                    },
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
                    set_is_dark: move |dark: bool| is_dark_theme.set(dark),
                    on_select_chapter: move |ch: u32| {
                        if let Some(book) = &*selected_book.read() {
                            if ch >= 1 && ch <= book.chapter_count {
                                selected_chapter.set(ch);
                                if let Some(trans) = &*selected_translation.read() {
                                    let tid = trans.id.clone();
                                    let bid = book.id;
                                    spawn(async move {
                                        let mut svc = BibleService::new();
                                        match svc.load_verses(&tid, bid, ch).await {
                                            Ok(vs) => verses.set(vs),
                                            Err(e) => load_error.set(Some(format!("{}", e))),
                                        }
                                    });
                                }
                                // refresh secondary
                                if let Some(sec) = &*secondary_translation.read() {
                                    let sec_id = sec.id.clone();
                                    let bid2 = book.id;
                                    let ch2 = ch;
                                    spawn(async move {
                                        let mut svc = BibleService::new();
                                        match svc.load_verses(&sec_id, bid2, ch2).await {
                                            Ok(vs) => secondary_verses.set(vs),
                                            Err(_) => secondary_verses.set(Vec::new()),
                                        }
                                    });
                                } else {
                                    secondary_verses.set(Vec::new());
                                }
                            }
                        }
                    }
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
                            class: format!("{} mx-auto p-8", if *is_parallel_view.read() && *is_parallel_by_columns.read() { "max-w-6xl" } else { "max-w-4xl" }),
                            
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
                                    // Secondary translation picker shown above verses when parallel view is active
                                    if *is_parallel_view.read() {
                                        div { class: "flex items-center justify-between mb-4",
                                            if let Some(primary) = &*selected_translation.read() {
                                                span { class: "text-sm text-secondary", "Primary: {primary.name}" }
                                            }
                                            div { class: "text-right",
                                                select {
                                                    class: "px-2 py-1 rounded border text-sm bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-700 text-gray-900 dark:text-gray-100",
                                                    value: secondary_translation.read().as_ref().map(|t| t.id.as_str()).unwrap_or(""),
                                                    onchange: move |evt| {
                                                        let tid = evt.value();
                                                        if tid.is_empty() {
                                                            secondary_translation.set(None);
                                                            secondary_verses.set(Vec::new());
                                                        } else {
                                                            is_parallel_view.set(true);
                                                            if let Some(book) = &*selected_book.read() {
                                                                let ch = *selected_chapter.read();
                                                                let bid = book.id;
                                                                let tid_clone = tid.clone();
                                                                spawn(async move {
                                                                    let mut svc = BibleService::new();
                                                                    match svc.load_verses(&tid_clone, bid, ch).await {
                                                                        Ok(vs) => secondary_verses.set(vs),
                                                                        Err(_) => secondary_verses.set(Vec::new()),
                                                                    }
                                                                });
                                                            }
                                                            if let Some(t) = translations.read().iter().find(|t| t.id == tid).cloned() {
                                                                secondary_translation.set(Some(t));
                                                            }
                                                        }
                                                    },
                                                    option { value: "", "None" }
                                                    for t in translations.read().iter() {
                                                        if let Some(primary) = &*selected_translation.read() {
                                                            if t.id != primary.id {
                                                                option { value: "{t.id}", "{t.name}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    div {
                                        class: "space-y-4",
                                        style: format!("font-size: {}rem; line-height: 1.6;", 1.125 * *zoom_level.read()),
                                        if *is_parallel_view.read() && *is_parallel_by_columns.read() {
                                            // Two columns: render row per verse so heights are aligned across columns
                                            div { class: "space-y-3",
                                                for verse in verses.read().iter() {
                                                    div { class: "grid grid-cols-1 lg:grid-cols-2 gap-4 lg:gap-6", key: "row2-{verse.id}",
                                                        // Left cell (primary)
                                                        div { class: "flex gap-3 items-start bg-secondary rounded-lg p-4 border border-gray-200 dark:border-gray-700 w-full",
                                                            div { class: "w-8 h-8 bg-blue-500 text-white rounded-full flex items-center justify-center text-sm font-bold tabular-nums flex-shrink-0", "{verse.verse}" }
                                                            p { class: "text-primary leading-relaxed min-h-[2rem] flex items-start flex-1", "{verse.text}" }
                                                        }
                                                        // Right cell (secondary or placeholder)
                                                        if let Some(sv) = secondary_verses.read().iter().find(|sv| sv.verse == verse.verse).cloned() {
                                                            div { class: "flex gap-3 items-start bg-secondary rounded-lg p-4 border border-gray-200 dark:border-gray-700 w-full",
                                                                div { class: "w-8 h-8 bg-purple-500 text-white rounded-full flex items-center justify-center text-sm font-bold tabular-nums flex-shrink-0", "{sv.verse}" }
                                                                p { class: "text-primary leading-relaxed min-h-[2rem] flex items-start flex-1", "{sv.text}" }
                                                            }
                                                        } else {
                                                            div { class: "flex gap-3 items-start bg-secondary rounded-lg p-4 border border-gray-200 dark:border-gray-700 opacity-50 w-full",
                                                                div { class: "w-8 h-8 bg-gray-400 text-white rounded-full flex items-center justify-center text-sm font-bold tabular-nums flex-shrink-0", "" }
                                                                p { class: "text-secondary leading-relaxed min-h-[2rem] flex items-start flex-1", "" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else if *is_parallel_view.read() && !*is_parallel_by_columns.read() {
                                            // Rows: primary verse then secondary under it if available
                                            div { class: "space-y-4",
                                                for verse in verses.read().iter() {
                                                    div { class: "bg-secondary rounded-lg border border-gray-200 dark:border-gray-700", key: "row-{verse.id}",
                                                        // Primary verse
                                                        div { class: "p-4 border-b border-gray-200 dark:border-gray-700",
                                                            div { class: "flex gap-3 items-start",
                                                                div { class: "w-8 h-8 bg-blue-500 text-white rounded-full flex items-center justify-center text-sm font-bold tabular-nums flex-shrink-0", "{verse.verse}" }
                                                                p { class: "text-primary leading-relaxed", "{verse.text}" }
                                                            }
                                                        }
                                                        // Secondary verse (if available)
                                                        if let Some(sv) = secondary_verses.read().iter().find(|sv| sv.verse == verse.verse).cloned() {
                                                            div { class: "p-4 bg-gray-50 dark:bg-gray-800",
                                                                div { class: "flex gap-3 items-start",
                                                                    div { class: "w-8 h-8 bg-purple-500 text-white rounded-full flex items-center justify-center text-sm font-bold tabular-nums flex-shrink-0", "{sv.verse}" }
                                                                    p { class: "text-primary leading-relaxed", "{sv.text}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            // Single view
                                            div { class: "space-y-3",
                                                for verse in verses.read().iter() {
                                                    div {
                                                        key: "{verse.id}",
                                                        class: "flex gap-4 items-start group hover:bg-tertiary rounded-lg p-4 transition-colors theme-transition bg-secondary border border-gray-200 dark:border-gray-700",
                                                        div {
                                                            class: "flex-shrink-0 w-8 h-8 bg-blue-500 text-white rounded-full flex items-center justify-center text-sm font-bold tabular-nums",
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