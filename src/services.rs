use crate::types::*;
use std::collections::HashMap;
use directories::ProjectDirs;
use std::path::{Path, PathBuf};
use tokio::fs;
use thiserror::Error;
use reqwest::Client;
use serde::Deserialize;

/// Service for managing Bible data operations
pub struct BibleService {
    translations: Vec<Translation>,
    books_cache: HashMap<String, Vec<Book>>,
    verses_cache: HashMap<String, Vec<Verse>>,
    hb_index_map: HashMap<String, HbEntryMinimal>,
}

impl BibleService {
    pub fn new() -> Self {
        Self {
            translations: Vec::new(),
            books_cache: HashMap::new(),
            verses_cache: HashMap::new(),
            hb_index_map: HashMap::new(),
        }
    }


    /// Load available translations: prefer remote HB_index, fallback to bundled
    pub async fn load_translations(&mut self) -> Result<Vec<Translation>, String> {
        match self.fetch_and_cache_remote_index().await {
            Ok(list) => {
                self.translations = list.clone();
                Ok(list)
            }
            Err(err) => {
                eprintln!("[BibleService] Remote index failed: {}. Falling back to bundled.", err);
                let translations_json = include_str!("data/translations_index.json");
                match serde_json::from_str::<TranslationIndex>(translations_json) {
                    Ok(data) => {
                        self.translations = data.translations.clone();
                        Ok(data.translations)
                    },
                    Err(e) => Err(format!("Failed to load translations: {}", e)),
                }
            }
        }
    }

    async fn fetch_and_cache_remote_index(&mut self) -> Result<Vec<Translation>, FetchError> {
        let client = Client::new();
        let url = "https://raw.githubusercontent.com/SujithChristopher/HB_index/master/bible-translations-index.json";
        let resp = client.get(url).send().await?.error_for_status()?;
        let bytes = resp.bytes().await?;
        // Parse either top-level array or object with `translations`
        let entries = parse_hb_entries(&bytes)?;
        // Map to our Translation model
        let mut map = HashMap::new();
        let translations = entries
            .iter()
            .map(|e| Translation {
                id: e.id.clone(),
                name: e.name.clone(),
                abbreviation: e.abbr.clone().unwrap_or_default(),
                language: e.lang.clone().unwrap_or_else(|| "unknown".into()),
                language_name: e.lang_name.clone(),
                description: e.description.clone().unwrap_or_default(),
                bundled: false,
                priority: 0,
            })
            .collect::<Vec<_>>();
        for e in entries.into_iter() {
            map.insert(
                e.id,
                HbEntryMinimal { download_url: e.download_url },
            );
        }
        self.hb_index_map = map;
        Ok(translations)
    }

    pub async fn download_translation_xml(&self, translation_id: &str) -> Result<PathBuf, String> {
        let entry = self
            .hb_index_map
            .get(translation_id)
            .ok_or_else(|| format!("Translation '{}' not found in index", translation_id))?;
        let url = entry
            .download_url
            .as_ref()
            .ok_or_else(|| format!("No download URL for '{}'", translation_id))?;
        let dir = app_data_dir()?.join("translations");
        ensure_dir(&dir).await?;
        let dest = dir.join(format!("{}.xml", translation_id));
        // Skip if already exists
        if tokio::fs::try_exists(&dest).await.map_err(|e| e.to_string())? {
            return Ok(dest);
        }
        let client = Client::new();
        let resp = client.get(url).send().await.map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?;
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        fs::write(&dest, &bytes).await.map_err(|e| format!("Failed to write {}: {}", dest.display(), e))?;
        Ok(dest)
    }

    pub async fn is_translation_downloaded(&self, translation_id: &str) -> Result<bool, String> {
        let path = app_data_dir()?.join("translations").join(format!("{}.xml", translation_id));
        Ok(tokio::fs::try_exists(path).await.map_err(|e| e.to_string())?)
    }

    pub async fn ensure_default_niv(&mut self) -> Result<(), String> {
        let exists = self.is_translation_downloaded("niv").await?;
        if !exists {
            let _ = self.fetch_and_cache_remote_index().await; // refresh map just in case
            let _path = self.download_translation_xml("niv").await?;
        }
        Ok(())
    }

    /// Load books for a specific translation
    pub async fn load_books(&mut self, translation_id: &str) -> Result<Vec<Book>, String> {
        // Check cache first
        if let Some(cached_books) = self.books_cache.get(translation_id) {
            return Ok(cached_books.clone());
        }

        // Load from embedded JSON
        // Most translations share the same book structure. Reuse KJV books for
        // other English translations as a reasonable default.
        let books_json = match translation_id {
            "kjv" | "niv" | "nkjv" => include_str!("data/kjv_books.json"),
            "tamil" => include_str!("data/tamil_books.json"),
            other => {
                // Fallback to KJV book list to avoid breaking the UI for unknown
                // but structurally similar translations.
                eprintln!("[BibleService] Falling back to KJV book list for translation: {}", other);
                include_str!("data/kjv_books.json")
            }
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

        // Prefer downloaded XML if available
        if let Ok(true) = self.is_translation_downloaded(translation_id).await {
            if let Ok(path) = self.xml_path_for_translation(translation_id) {
                match Self::parse_verses_from_xml(&path, translation_id, book_id, chapter).await {
                    Ok(list) if !list.is_empty() => {
                        self.verses_cache.insert(cache_key, list.clone());
                        return Ok(list);
                    }
                    Ok(_) => { /* fall through to JSON fallback */ }
                    Err(e) => {
                        eprintln!("[BibleService] XML parse failed for {}: {}. Falling back to bundled JSON.", translation_id, e);
                    }
                }
            }
        }

        // Load verses from embedded JSON (fallback)
        let verses_json = match translation_id {
            "kjv" => include_str!("data/kjv_verses.json"),
            "tamil" => include_str!("data/tamil_verses.json"),
            "niv" => include_str!("data/niv_verses.json"),
            "nkjv" => include_str!("data/nkjv_verses.json"),
            other => {
                eprintln!("[BibleService] Translation '{}' not bundled. Falling back to KJV verses.", other);
                include_str!("data/kjv_verses.json")
            }
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

    fn xml_path_for_translation(&self, translation_id: &str) -> Result<PathBuf, String> {
        Ok(app_data_dir()?.join("translations").join(format!("{}.xml", translation_id)))
    }

    async fn parse_verses_from_xml(
        path: &Path,
        translation_id: &str,
        target_book_id: u32,
        target_chapter: u32,
    ) -> Result<Vec<Verse>, String> {
        use quick_xml::events::Event;
        use quick_xml::name::QName;
        use quick_xml::Reader;
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
        let mut reader = Reader::from_reader(BufReader::new(file));
        #[allow(deprecated)]
        {
            // quick-xml >=0.38 uses config_mut(); some versions expose a setter, else set field
            let cfg = reader.config_mut();
            cfg.trim_text(true);
        }

        let mut buf = Vec::new();
        let mut verses: Vec<Verse> = Vec::new();

        // We assume OSIS-like: <verse osisID="Gen.1.1">text</verse>
        // Map osis book code to our book_id via abbreviation from bundled books list
        let osis_to_book_id = Self::osis_book_map()?;

        let mut current_osis_id: Option<String> = None;
        let mut collecting_text = false;
        let mut text_acc = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    if e.name() == QName(b"verse") {
                        // Find osisID attribute
                        let mut osis_id: Option<String> = None;
                        for attr in e.attributes().with_checks(false) {
                            if let Ok(a) = attr {
                                if a.key == QName(b"osisID") {
                                    if let Ok(val) = a.unescape_value() {
                                        osis_id = Some(val.to_string());
                                        break;
                                    }
                                }
                            }
                        }
                        if let Some(oid) = osis_id {
                            // Parse e.g., Gen.1.1
                            if let Some((book_code, ch, vs)) = Self::parse_osis(&oid) {
                                if let Some(&bid) = osis_to_book_id.get(book_code.as_str()) {
                                    if bid == target_book_id && ch == target_chapter {
                                        current_osis_id = Some(oid);
                                        if matches!(e, quick_xml::events::BytesStart { .. }) {
                                            // will collect until End(verse)
                                            collecting_text = true;
                                            text_acc.clear();
                                        } else {
                                            // Empty tag -> push immediately
                                            verses.push(Verse {
                                                id: format!("{}:{}:{}:{}", translation_id, bid, ch, vs),
                                                translation_id: translation_id.to_string(),
                                                book_id: bid,
                                                chapter: ch,
                                                verse: vs,
                                                text: String::new(),
                                            });
                                            collecting_text = false;
                                            current_osis_id = None;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    if collecting_text {
                        let decoded = reader.decoder().decode(e.as_ref()).unwrap_or_default();
                        text_acc.push_str(&decoded);
                    }
                }
                Ok(Event::End(e)) => {
                    if e.name() == QName(b"verse") && collecting_text {
                        if let Some(oid) = current_osis_id.take() {
                            if let Some((book_code, ch, vs)) = Self::parse_osis(&oid) {
                                if let Some(&bid) = osis_to_book_id.get(book_code.as_str()) {
                                    if bid == target_book_id && ch == target_chapter {
                                        verses.push(Verse {
                                            id: format!("{}:{}:{}:{}", translation_id, bid, ch, vs),
                                            translation_id: translation_id.to_string(),
                                            book_id: bid,
                                            chapter: ch,
                                            verse: vs,
                                            text: text_acc.trim().to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        collecting_text = false;
                        text_acc.clear();
                    }
                }
                Err(e) => return Err(format!("XML error at pos {}: {}", reader.buffer_position(), e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(verses)
    }

    fn parse_osis(osis: &str) -> Option<(String, u32, u32)> {
        // Expect BOOK.CHAPTER.VERSE
        let mut parts = osis.split('.');
        let book = parts.next()?.to_string();
        let ch = parts.next()?.parse().ok()?;
        let vs = parts.next()?.parse().ok()?;
        Some((book, ch, vs))
    }

    fn osis_book_map() -> Result<std::collections::HashMap<String, u32>, String> {
        // Reuse bundled KJV book list for ID mapping and abbreviations
        let books_json = include_str!("data/kjv_books.json");
        let books: Vec<Book> = serde_json::from_str(books_json).map_err(|e| format!("Book map parse error: {}", e))?;
        let mut map = std::collections::HashMap::new();
        for b in books {
            map.insert(b.abbreviation.clone(), b.id);
        }
        Ok(map)
    }

    // Note: access translations via `load_translations` return value

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

// HB_index structures (partial)
#[derive(Debug, Clone, Deserialize)]
struct HbEntry {
    id: String,
    name: String,
    #[serde(default)]
    abbr: Option<String>,
    #[serde(default)]
    lang: Option<String>,
    #[serde(default)]
    lang_name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    download_url: Option<String>,
}

#[derive(Debug)]
struct HbEntryMinimal {
    download_url: Option<String>,
}

#[derive(Error, Debug)]
enum FetchError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

fn parse_hb_entries(bytes: &[u8]) -> Result<Vec<HbEntry>, FetchError> {
    // Try top-level array first
    if let Ok(list) = serde_json::from_slice::<Vec<HbEntry>>(bytes) {
        return Ok(list);
    }
    // Try object with translations
    #[derive(Deserialize)]
    struct Wrapper { translations: Vec<HbEntry> }
    if let Ok(wrapped) = serde_json::from_slice::<Wrapper>(bytes) {
        return Ok(wrapped.translations);
    }
    // Try map of id -> entry
    if let Ok(map) = serde_json::from_slice::<std::collections::HashMap<String, HbEntry>>(bytes) {
        let mut out: Vec<HbEntry> = map.into_values().collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        return Ok(out);
    }
    // Try line-delimited JSON (one HbEntry per line)
    if let Ok(text) = std::str::from_utf8(bytes) {
        let mut list: Vec<HbEntry> = Vec::new();
        for line in text.lines() {
            let s = line.trim();
            if s.starts_with('{') && s.ends_with('}') {
                if let Ok(entry) = serde_json::from_str::<HbEntry>(s) {
                    list.push(entry);
                }
            }
        }
        if !list.is_empty() {
            list.sort_by(|a, b| a.id.cmp(&b.id));
            return Ok(list);
        }
    }
    // If all formats fail, return serde error from object attempt for context
    let wrapped: Wrapper = serde_json::from_slice(bytes)?;
    Ok(wrapped.translations)
}

// Storage helpers
fn app_data_dir() -> Result<PathBuf, String> {
    let proj = ProjectDirs::from("dev", "StudyBible", "StudyBible").ok_or_else(|| "Cannot determine user data directory".to_string())?;
    let dir = proj.data_dir().join("StudyBible");
    Ok(dir)
}

async fn ensure_dir(path: &Path) -> Result<(), String> {
    if let Err(e) = fs::create_dir_all(path).await {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(format!("Failed to create directory {}: {}", path.display(), e));
        }
    }
    Ok(())
}

// Removed unused ServiceManager wrapper