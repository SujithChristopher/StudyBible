use crate::types::*;
use std::collections::HashMap;

/// Service for managing Bible data operations
pub struct BibleService {
    translations: Vec<Translation>,
    books_cache: HashMap<String, Vec<Book>>,
    verses_cache: HashMap<String, Vec<Verse>>,
}

impl BibleService {
    pub fn new() -> Self {
        Self {
            translations: Vec::new(),
            books_cache: HashMap::new(),
            verses_cache: HashMap::new(),
        }
    }

    /// Initialize the service by loading translations
    pub async fn initialize(&mut self) -> Result<(), String> {
        self.load_translations().await
    }

    /// Load available translations
    pub async fn load_translations(&mut self) -> Result<(), String> {
        let translations_json = include_str!("data/translations_index.json");
        match serde_json::from_str::<TranslationIndex>(translations_json) {
            Ok(data) => {
                self.translations = data.translations;
                Ok(())
            },
            Err(e) => Err(format!("Failed to load translations: {}", e)),
        }
    }

    /// Load books for a specific translation
    pub async fn load_books(&mut self, translation_id: &str) -> Result<Vec<Book>, String> {
        // Check cache first
        if let Some(cached_books) = self.books_cache.get(translation_id) {
            return Ok(cached_books.clone());
        }

        // Load from embedded JSON
        let books_json = match translation_id {
            "kjv" => include_str!("data/kjv_books.json"),
            "tamil" => include_str!("data/tamil_books.json"),
            _ => return Err(format!("Translation '{}' not supported", translation_id)),
        };

        match serde_json::from_str::<Vec<Book>>(books_json) {
            Ok(books) => {
                self.books_cache.insert(translation_id.to_string(), books.clone());
                Ok(books)
            },
            Err(e) => Err(format!("Failed to load books for {}: {}", translation_id, e)),
        }
    }

    /// Load verses for a specific translation, book, and chapter
    pub async fn load_verses(
        &mut self,
        translation_id: &str,
        book_id: u32,
        chapter: u32,
    ) -> Result<Vec<Verse>, String> {
        let cache_key = format!("{}_{}_{}",  translation_id, book_id, chapter);
        
        // Check cache first
        if let Some(cached_verses) = self.verses_cache.get(&cache_key) {
            return Ok(cached_verses.clone());
        }

        // Load verses from embedded JSON
        let verses_json = match translation_id {
            "kjv" => include_str!("data/kjv_verses.json"),
            "tamil" => include_str!("data/tamil_verses.json"),
            "niv" => include_str!("data/niv_verses.json"),
            "nkjv" => include_str!("data/nkjv_verses.json"),
            _ => return Err(format!("Translation '{}' not supported", translation_id)),
        };

        match serde_json::from_str::<Vec<Verse>>(verses_json) {
            Ok(all_verses) => {
                // Filter verses for the specific book and chapter
                let filtered_verses: Vec<Verse> = all_verses
                    .into_iter()
                    .filter(|v| v.book_id == book_id && v.chapter == chapter)
                    .collect();

                // Cache the filtered verses
                self.verses_cache.insert(cache_key, filtered_verses.clone());
                Ok(filtered_verses)
            },
            Err(e) => Err(format!("Failed to load verses for {}: {}", translation_id, e)),
        }
    }

    /// Get all available translations
    pub fn get_translations(&self) -> &Vec<Translation> {
        &self.translations
    }

    /// Get cached books for a translation
    pub fn get_cached_books(&self, translation_id: &str) -> Option<&Vec<Book>> {
        self.books_cache.get(translation_id)
    }

    /// Search for verses containing the query text across all books
    pub async fn search_verses(&mut self, translation_id: &str, query: &str) -> Result<Vec<Verse>, Box<dyn std::error::Error>> {
        let search_query = query.to_lowercase();
        let mut search_results = Vec::new();
        
        // Get all books for this translation
        let books = self.load_books(translation_id).await?;
        
        // Search through all chapters of all books (limit to first few books for performance)
        for book in books.iter().take(5) { // Limit search to first 5 books for demo
            for chapter in 1..=book.chapter_count.min(3) { // Limit to first 3 chapters per book
                match self.load_verses(translation_id, book.id, chapter).await {
                    Ok(verses) => {
                        for verse in verses {
                            if verse.text.to_lowercase().contains(&search_query) {
                                search_results.push(verse);
                            }
                        }
                    }
                    Err(_) => continue, // Skip chapters that fail to load
                }
            }
        }
        
        Ok(search_results)
    }
}

impl Default for BibleService {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper struct for parsing translations index JSON
#[derive(serde::Deserialize)]
struct TranslationIndex {
    translations: Vec<Translation>,
}

/// Global service instance management
pub struct ServiceManager {
    bible_service: BibleService,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            bible_service: BibleService::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), String> {
        self.bible_service.initialize().await
    }

    pub fn bible(&mut self) -> &mut BibleService {
        &mut self.bible_service
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}