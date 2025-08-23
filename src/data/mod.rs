use crate::types::*;
use std::collections::HashMap;

/// Bible data management module
pub struct BibleDataManager {
    translations: Vec<Translation>,
    books: HashMap<String, Vec<Book>>, // translation_id -> books
    verses: HashMap<String, Vec<Verse>>, // "{translation_id}_{book_id}_{chapter}" -> verses
    bookmarks: Vec<Bookmark>,
    highlights: Vec<TextHighlight>,
}

impl BibleDataManager {
    pub fn new() -> Self {
        Self {
            translations: Vec::new(),
            books: HashMap::new(),
            verses: HashMap::new(),
            bookmarks: Vec::new(),
            highlights: Vec::new(),
        }
    }

    /// Load translations from the translations index file
    pub async fn load_translations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let translations_json = include_str!("translations_index.json");
        let translations_data: TranslationIndex = serde_json::from_str(translations_json)?;
        self.translations = translations_data.translations;
        Ok(())
    }

    /// Load books for a specific translation
    pub async fn load_books(&mut self, translation_id: &str) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
        // Load from embedded JSON files based on translation
        let books_json = match translation_id {
            "kjv" => include_str!("kjv_books.json"),
            "tamil" => include_str!("tamil_books.json"),
            _ => return Err(format!("Translation '{}' not found", translation_id).into()),
        };

        let books: Vec<Book> = serde_json::from_str(books_json)?;
        self.books.insert(translation_id.to_string(), books.clone());
        Ok(books)
    }

    /// Load verses for a specific translation, book, and chapter
    pub async fn load_verses(
        &mut self,
        translation_id: &str,
        book_id: u32,
        chapter: u32,
    ) -> Result<Vec<Verse>, Box<dyn std::error::Error>> {
        let cache_key = format!("{}_{}_{}",  translation_id, book_id, chapter);
        
        // Check if verses are already cached
        if let Some(cached_verses) = self.verses.get(&cache_key) {
            return Ok(cached_verses.clone());
        }

        // Load verses from embedded JSON files
        let verses_json = match translation_id {
            "kjv" => include_str!("kjv_verses.json"),
            "tamil" => include_str!("tamil_verses.json"),
            "niv" => include_str!("niv_verses.json"),
            "nkjv" => include_str!("nkjv_verses.json"),
            _ => return Err(format!("Translation '{}' not found", translation_id).into()),
        };

        // Parse all verses and filter by book and chapter
        let all_verses: Vec<Verse> = serde_json::from_str(verses_json)?;
        let filtered_verses: Vec<Verse> = all_verses
            .into_iter()
            .filter(|v| v.book_id == book_id && v.chapter == chapter)
            .collect();

        // Cache the verses
        self.verses.insert(cache_key, filtered_verses.clone());
        Ok(filtered_verses)
    }

    /// Get all available translations
    pub fn get_translations(&self) -> &Vec<Translation> {
        &self.translations
    }

    /// Get books for a specific translation
    pub fn get_books(&self, translation_id: &str) -> Option<&Vec<Book>> {
        self.books.get(translation_id)
    }

    /// Search verses across a translation
    pub async fn search_verses(
        &self,
        translation_id: &str,
        query: &str,
    ) -> Result<SearchResult, Box<dyn std::error::Error>> {
        // For now, this is a placeholder implementation
        // In a full implementation, this would search through all verses
        Ok(SearchResult {
            verses: Vec::new(),
            total_count: 0,
            query: query.to_string(),
            translation_id: translation_id.to_string(),
        })
    }

    /// Add a bookmark
    pub fn add_bookmark(&mut self, bookmark: Bookmark) {
        self.bookmarks.push(bookmark);
    }

    /// Remove a bookmark
    pub fn remove_bookmark(&mut self, bookmark_id: &str) {
        self.bookmarks.retain(|b| b.id != bookmark_id);
    }

    /// Get all bookmarks
    pub fn get_bookmarks(&self) -> &Vec<Bookmark> {
        &self.bookmarks
    }

    /// Add a highlight
    pub fn add_highlight(&mut self, highlight: TextHighlight) {
        self.highlights.push(highlight);
    }

    /// Remove a highlight
    pub fn remove_highlight(&mut self, highlight_id: &str) {
        self.highlights.retain(|h| h.id != highlight_id);
    }

    /// Get highlights for a specific verse
    pub fn get_verse_highlights(
        &self,
        translation_id: &str,
        book_id: u32,
        chapter: u32,
        verse: u32,
    ) -> Vec<&TextHighlight> {
        self.highlights
            .iter()
            .filter(|h| {
                h.translation_id == translation_id
                    && h.book_id == book_id
                    && h.chapter == chapter
                    && h.verse == verse
            })
            .collect()
    }
}

/// Helper struct for parsing translations index JSON
#[derive(serde::Deserialize)]
struct TranslationIndex {
    translations: Vec<Translation>,
}

impl Default for BibleDataManager {
    fn default() -> Self {
        Self::new()
    }
}