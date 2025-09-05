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


    /// Load available translations: prefer local HB_index, fallback to remote, then bundled
    pub async fn load_translations(&mut self) -> Result<Vec<Translation>, String> {
        // Try local HB_index submodule first
        match self.fetch_local_hb_index().await {
            Ok(list) => {
                self.translations = list.clone();
                return Ok(list);
            }
            Err(err) => {
                eprintln!("[BibleService] Local HB_index failed: {}. Trying remote.", err);
            }
        }

        // Try remote HB_index as fallback
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
        
        // Try new HB_index format first
        if let Ok(hb_index) = serde_json::from_slice::<HbIndex>(&bytes) {
            let mut map = HashMap::new();
            let mut translations = Vec::new();
            
            for lang in hb_index.languages {
                for trans in lang.translations {
                    // Create Translation from HbTranslation
                    let translation = Translation {
                        id: trans.id.clone(),
                        name: trans.name.clone(),
                        abbreviation: extract_abbreviation(&trans.name),
                        language: lang.iso_code.clone().unwrap_or_else(|| lang.language.clone().to_lowercase()),
                        language_name: Some(lang.native_name.clone().unwrap_or(lang.language.clone())),
                        description: trans.metadata.as_ref()
                            .and_then(|m| m.info.clone())
                            .unwrap_or_else(|| trans.name.clone()),
                        bundled: false,
                        priority: 0,
                    };
                    translations.push(translation);
                    
                    // Store download URL mapping
                    map.insert(
                        trans.id,
                        HbEntryMinimal { download_url: trans.download_url },
                    );
                }
            }
            
            self.hb_index_map = map;
            return Ok(translations);
        }
        
        // Fallback to legacy format
        let entries = parse_hb_entries(&bytes)?;
        let mut map = HashMap::new();
        let translations = entries
            .iter()
            .map(|e| Translation {
                id: e.id.clone(),
                name: e.name.clone(),
                abbreviation: e.abbr.clone().unwrap_or_else(|| extract_abbreviation(&e.name)),
                language: e.lang.clone().unwrap_or_else(|| "unknown".into()),
                language_name: e.lang_name.clone(),
                description: e.description.clone().unwrap_or_else(|| e.name.clone()),
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

    async fn fetch_local_hb_index(&mut self) -> Result<Vec<Translation>, String> {
        // Try to read local HB_index submodule
        let local_path = std::path::Path::new("HB_index/bible-translations-index.json");
        if !local_path.exists() {
            return Err("Local HB_index file not found".to_string());
        }

        let content = tokio::fs::read_to_string(local_path).await
            .map_err(|e| format!("Failed to read local HB_index: {}", e))?;

        // Parse the local HB_index format
        let hb_index: HbIndex = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse local HB_index: {}", e))?;

        let mut map = HashMap::new();
        let mut translations = Vec::new();
        
        for lang in hb_index.languages {
            for trans in lang.translations {
                // Create Translation from HbTranslation
                let translation = Translation {
                    id: trans.id.clone(),
                    name: trans.name.clone(),
                    abbreviation: extract_abbreviation(&trans.name),
                    language: lang.iso_code.clone().unwrap_or_else(|| lang.language.clone().to_lowercase()),
                    language_name: Some(lang.native_name.clone().unwrap_or(lang.language.clone())),
                    description: trans.metadata.as_ref()
                        .and_then(|m| m.info.clone())
                        .unwrap_or_else(|| trans.name.clone()),
                    bundled: false,
                    priority: 0,
                };
                translations.push(translation);
                
                // Store download URL mapping
                map.insert(
                    trans.id,
                    HbEntryMinimal { download_url: trans.download_url },
                );
            }
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

    pub async fn ensure_default_translation(&mut self) -> Result<(), String> {
        // Try to ensure we have at least one translation downloaded
        // First refresh the remote index
        if let Err(_) = self.fetch_and_cache_remote_index().await {
            // If remote fails, we'll rely on bundled fallback in main.rs
            return Ok(());
        }
        
        // Look for any already downloaded translations
        let data_dir = app_data_dir()?.join("translations");
        if let Ok(entries) = tokio::fs::read_dir(&data_dir).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("xml") {
                    // We have at least one XML file downloaded
                    return Ok(());
                }
            }
        }
        
        // If no translations downloaded, try to download the first available one
        if let Some((id, _)) = self.hb_index_map.iter().next() {
            let id = id.clone();
            match self.download_translation_xml(&id).await {
                Ok(_) => println!("Downloaded default translation: {}", id),
                Err(e) => println!("Failed to download default translation {}: {}", id, e),
            }
        }
        
        Ok(())
    }

    /// Load books for a specific translation
    pub async fn load_books(&mut self, translation_id: &str) -> Result<Vec<Book>, String> {
        // Check cache first
        if let Some(cached_books) = self.books_cache.get(translation_id) {
            return Ok(cached_books.clone());
        }

        // Prefer downloaded XML if available
        if let Ok(true) = self.is_translation_downloaded(translation_id).await {
            if let Ok(path) = self.xml_path_for_translation(translation_id) {
                match Self::parse_books_from_xml(&path).await {
                    Ok(books) if !books.is_empty() => {
                        self.books_cache.insert(translation_id.to_string(), books.clone());
                        return Ok(books);
                    }
                    Ok(_) => { /* fall through to standard books */ }
                    Err(e) => {
                        eprintln!("[BibleService] XML book parse failed for {}: {}. Using standard book list.", translation_id, e);
                    }
                }
            }
        }

        // Use standard Bible book list (66 books)
        let books = Self::get_standard_bible_books();
        self.books_cache.insert(translation_id.to_string(), books.clone());
        Ok(books)
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

        // If XML parsing failed or no verses found, return error
        // No more JSON fallback for verses - we require XML downloads
        Err(format!("No verses found for {} book {} chapter {} - translation may need to be downloaded", translation_id, book_id, chapter))
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

    async fn parse_books_from_xml(path: &Path) -> Result<Vec<Book>, String> {
        use quick_xml::events::Event;
        use quick_xml::name::QName;
        use quick_xml::Reader;
        use std::fs::File;
        use std::io::BufReader;
        use std::collections::HashMap;

        let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
        let mut reader = Reader::from_reader(BufReader::new(file));
        #[allow(deprecated)]
        {
            let cfg = reader.config_mut();
            cfg.trim_text(true);
        }

        let mut buf = Vec::new();
        let mut book_map: HashMap<String, (u32, u32)> = HashMap::new(); // osis_code -> (max_chapter, book_id)
        let osis_to_book_id = Self::osis_book_map()?;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    if e.name() == QName(b"verse") {
                        // Find osisID attribute
                        for attr in e.attributes().with_checks(false) {
                            if let Ok(a) = attr {
                                if a.key == QName(b"osisID") {
                                    if let Ok(val) = a.unescape_value() {
                                        if let Some((book_code, ch, _vs)) = Self::parse_osis(&val) {
                                            if let Some(&bid) = osis_to_book_id.get(book_code.as_str()) {
                                                let entry = book_map.entry(book_code).or_insert((0, bid));
                                                entry.0 = entry.0.max(ch);
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => return Err(format!("XML error at pos {}: {}", reader.buffer_position(), e)),
                _ => {}
            }
            buf.clear();
        }

        // Convert to Book structs using standard book names
        let standard_books = Self::get_standard_bible_books();
        let mut found_books = Vec::new();
        
        for book in standard_books {
            if let Some(&(chapter_count, _)) = book_map.get(&book.abbreviation) {
                let mut book_with_chapters = book;
                book_with_chapters.chapter_count = chapter_count;
                found_books.push(book_with_chapters);
            }
        }

        found_books.sort_by_key(|b| b.order_index);
        Ok(found_books)
    }

    fn get_standard_bible_books() -> Vec<Book> {
        use crate::types::Testament;
        
        vec![
            // Old Testament Books
            Book { id: 1, name: "Genesis".to_string(), abbreviation: "Gen".to_string(), testament: Testament::OT, order_index: 1, chapter_count: 50 },
            Book { id: 2, name: "Exodus".to_string(), abbreviation: "Exod".to_string(), testament: Testament::OT, order_index: 2, chapter_count: 40 },
            Book { id: 3, name: "Leviticus".to_string(), abbreviation: "Lev".to_string(), testament: Testament::OT, order_index: 3, chapter_count: 27 },
            Book { id: 4, name: "Numbers".to_string(), abbreviation: "Num".to_string(), testament: Testament::OT, order_index: 4, chapter_count: 36 },
            Book { id: 5, name: "Deuteronomy".to_string(), abbreviation: "Deut".to_string(), testament: Testament::OT, order_index: 5, chapter_count: 34 },
            Book { id: 6, name: "Joshua".to_string(), abbreviation: "Josh".to_string(), testament: Testament::OT, order_index: 6, chapter_count: 24 },
            Book { id: 7, name: "Judges".to_string(), abbreviation: "Judg".to_string(), testament: Testament::OT, order_index: 7, chapter_count: 21 },
            Book { id: 8, name: "Ruth".to_string(), abbreviation: "Ruth".to_string(), testament: Testament::OT, order_index: 8, chapter_count: 4 },
            Book { id: 9, name: "1 Samuel".to_string(), abbreviation: "1Sam".to_string(), testament: Testament::OT, order_index: 9, chapter_count: 31 },
            Book { id: 10, name: "2 Samuel".to_string(), abbreviation: "2Sam".to_string(), testament: Testament::OT, order_index: 10, chapter_count: 24 },
            Book { id: 11, name: "1 Kings".to_string(), abbreviation: "1Kgs".to_string(), testament: Testament::OT, order_index: 11, chapter_count: 22 },
            Book { id: 12, name: "2 Kings".to_string(), abbreviation: "2Kgs".to_string(), testament: Testament::OT, order_index: 12, chapter_count: 25 },
            Book { id: 13, name: "1 Chronicles".to_string(), abbreviation: "1Chr".to_string(), testament: Testament::OT, order_index: 13, chapter_count: 29 },
            Book { id: 14, name: "2 Chronicles".to_string(), abbreviation: "2Chr".to_string(), testament: Testament::OT, order_index: 14, chapter_count: 36 },
            Book { id: 15, name: "Ezra".to_string(), abbreviation: "Ezra".to_string(), testament: Testament::OT, order_index: 15, chapter_count: 10 },
            Book { id: 16, name: "Nehemiah".to_string(), abbreviation: "Neh".to_string(), testament: Testament::OT, order_index: 16, chapter_count: 13 },
            Book { id: 17, name: "Esther".to_string(), abbreviation: "Esth".to_string(), testament: Testament::OT, order_index: 17, chapter_count: 10 },
            Book { id: 18, name: "Job".to_string(), abbreviation: "Job".to_string(), testament: Testament::OT, order_index: 18, chapter_count: 42 },
            Book { id: 19, name: "Psalms".to_string(), abbreviation: "Ps".to_string(), testament: Testament::OT, order_index: 19, chapter_count: 150 },
            Book { id: 20, name: "Proverbs".to_string(), abbreviation: "Prov".to_string(), testament: Testament::OT, order_index: 20, chapter_count: 31 },
            Book { id: 21, name: "Ecclesiastes".to_string(), abbreviation: "Eccl".to_string(), testament: Testament::OT, order_index: 21, chapter_count: 12 },
            Book { id: 22, name: "Song of Solomon".to_string(), abbreviation: "Song".to_string(), testament: Testament::OT, order_index: 22, chapter_count: 8 },
            Book { id: 23, name: "Isaiah".to_string(), abbreviation: "Isa".to_string(), testament: Testament::OT, order_index: 23, chapter_count: 66 },
            Book { id: 24, name: "Jeremiah".to_string(), abbreviation: "Jer".to_string(), testament: Testament::OT, order_index: 24, chapter_count: 52 },
            Book { id: 25, name: "Lamentations".to_string(), abbreviation: "Lam".to_string(), testament: Testament::OT, order_index: 25, chapter_count: 5 },
            Book { id: 26, name: "Ezekiel".to_string(), abbreviation: "Ezek".to_string(), testament: Testament::OT, order_index: 26, chapter_count: 48 },
            Book { id: 27, name: "Daniel".to_string(), abbreviation: "Dan".to_string(), testament: Testament::OT, order_index: 27, chapter_count: 12 },
            Book { id: 28, name: "Hosea".to_string(), abbreviation: "Hos".to_string(), testament: Testament::OT, order_index: 28, chapter_count: 14 },
            Book { id: 29, name: "Joel".to_string(), abbreviation: "Joel".to_string(), testament: Testament::OT, order_index: 29, chapter_count: 3 },
            Book { id: 30, name: "Amos".to_string(), abbreviation: "Amos".to_string(), testament: Testament::OT, order_index: 30, chapter_count: 9 },
            Book { id: 31, name: "Obadiah".to_string(), abbreviation: "Obad".to_string(), testament: Testament::OT, order_index: 31, chapter_count: 1 },
            Book { id: 32, name: "Jonah".to_string(), abbreviation: "Jonah".to_string(), testament: Testament::OT, order_index: 32, chapter_count: 4 },
            Book { id: 33, name: "Micah".to_string(), abbreviation: "Mic".to_string(), testament: Testament::OT, order_index: 33, chapter_count: 7 },
            Book { id: 34, name: "Nahum".to_string(), abbreviation: "Nah".to_string(), testament: Testament::OT, order_index: 34, chapter_count: 3 },
            Book { id: 35, name: "Habakkuk".to_string(), abbreviation: "Hab".to_string(), testament: Testament::OT, order_index: 35, chapter_count: 3 },
            Book { id: 36, name: "Zephaniah".to_string(), abbreviation: "Zeph".to_string(), testament: Testament::OT, order_index: 36, chapter_count: 3 },
            Book { id: 37, name: "Haggai".to_string(), abbreviation: "Hag".to_string(), testament: Testament::OT, order_index: 37, chapter_count: 2 },
            Book { id: 38, name: "Zechariah".to_string(), abbreviation: "Zech".to_string(), testament: Testament::OT, order_index: 38, chapter_count: 14 },
            Book { id: 39, name: "Malachi".to_string(), abbreviation: "Mal".to_string(), testament: Testament::OT, order_index: 39, chapter_count: 4 },
            
            // New Testament Books
            Book { id: 40, name: "Matthew".to_string(), abbreviation: "Matt".to_string(), testament: Testament::NT, order_index: 40, chapter_count: 28 },
            Book { id: 41, name: "Mark".to_string(), abbreviation: "Mark".to_string(), testament: Testament::NT, order_index: 41, chapter_count: 16 },
            Book { id: 42, name: "Luke".to_string(), abbreviation: "Luke".to_string(), testament: Testament::NT, order_index: 42, chapter_count: 24 },
            Book { id: 43, name: "John".to_string(), abbreviation: "John".to_string(), testament: Testament::NT, order_index: 43, chapter_count: 21 },
            Book { id: 44, name: "Acts".to_string(), abbreviation: "Acts".to_string(), testament: Testament::NT, order_index: 44, chapter_count: 28 },
            Book { id: 45, name: "Romans".to_string(), abbreviation: "Rom".to_string(), testament: Testament::NT, order_index: 45, chapter_count: 16 },
            Book { id: 46, name: "1 Corinthians".to_string(), abbreviation: "1Cor".to_string(), testament: Testament::NT, order_index: 46, chapter_count: 16 },
            Book { id: 47, name: "2 Corinthians".to_string(), abbreviation: "2Cor".to_string(), testament: Testament::NT, order_index: 47, chapter_count: 13 },
            Book { id: 48, name: "Galatians".to_string(), abbreviation: "Gal".to_string(), testament: Testament::NT, order_index: 48, chapter_count: 6 },
            Book { id: 49, name: "Ephesians".to_string(), abbreviation: "Eph".to_string(), testament: Testament::NT, order_index: 49, chapter_count: 6 },
            Book { id: 50, name: "Philippians".to_string(), abbreviation: "Phil".to_string(), testament: Testament::NT, order_index: 50, chapter_count: 4 },
            Book { id: 51, name: "Colossians".to_string(), abbreviation: "Col".to_string(), testament: Testament::NT, order_index: 51, chapter_count: 4 },
            Book { id: 52, name: "1 Thessalonians".to_string(), abbreviation: "1Thess".to_string(), testament: Testament::NT, order_index: 52, chapter_count: 5 },
            Book { id: 53, name: "2 Thessalonians".to_string(), abbreviation: "2Thess".to_string(), testament: Testament::NT, order_index: 53, chapter_count: 3 },
            Book { id: 54, name: "1 Timothy".to_string(), abbreviation: "1Tim".to_string(), testament: Testament::NT, order_index: 54, chapter_count: 6 },
            Book { id: 55, name: "2 Timothy".to_string(), abbreviation: "2Tim".to_string(), testament: Testament::NT, order_index: 55, chapter_count: 4 },
            Book { id: 56, name: "Titus".to_string(), abbreviation: "Titus".to_string(), testament: Testament::NT, order_index: 56, chapter_count: 3 },
            Book { id: 57, name: "Philemon".to_string(), abbreviation: "Phlm".to_string(), testament: Testament::NT, order_index: 57, chapter_count: 1 },
            Book { id: 58, name: "Hebrews".to_string(), abbreviation: "Heb".to_string(), testament: Testament::NT, order_index: 58, chapter_count: 13 },
            Book { id: 59, name: "James".to_string(), abbreviation: "Jas".to_string(), testament: Testament::NT, order_index: 59, chapter_count: 5 },
            Book { id: 60, name: "1 Peter".to_string(), abbreviation: "1Pet".to_string(), testament: Testament::NT, order_index: 60, chapter_count: 5 },
            Book { id: 61, name: "2 Peter".to_string(), abbreviation: "2Pet".to_string(), testament: Testament::NT, order_index: 61, chapter_count: 3 },
            Book { id: 62, name: "1 John".to_string(), abbreviation: "1John".to_string(), testament: Testament::NT, order_index: 62, chapter_count: 5 },
            Book { id: 63, name: "2 John".to_string(), abbreviation: "2John".to_string(), testament: Testament::NT, order_index: 63, chapter_count: 1 },
            Book { id: 64, name: "3 John".to_string(), abbreviation: "3John".to_string(), testament: Testament::NT, order_index: 64, chapter_count: 1 },
            Book { id: 65, name: "Jude".to_string(), abbreviation: "Jude".to_string(), testament: Testament::NT, order_index: 65, chapter_count: 1 },
            Book { id: 66, name: "Revelation".to_string(), abbreviation: "Rev".to_string(), testament: Testament::NT, order_index: 66, chapter_count: 22 },
        ]
    }

    fn osis_book_map() -> Result<std::collections::HashMap<String, u32>, String> {
        // Use standard Bible book list for ID mapping and abbreviations
        let books = Self::get_standard_bible_books();
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

// HB_index structures
#[derive(Debug, Clone, Deserialize)]
struct HbLanguage {
    language: String,
    #[serde(default)]
    native_name: Option<String>,
    #[serde(default)]
    iso_code: Option<String>,
    translations: Vec<HbTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
struct HbTranslation {
    id: String,
    name: String,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    download_url: Option<String>,
    #[serde(default)]
    file_size_bytes: Option<u64>,
    #[serde(default)]
    testament_coverage: Option<HbTestamentCoverage>,
    #[serde(default)]
    metadata: Option<HbMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
struct HbTestamentCoverage {
    #[serde(default)]
    old_testament: bool,
    #[serde(default)]
    new_testament: bool,
    #[serde(default)]
    total_books: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct HbMetadata {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    year: Option<u32>,
    #[serde(default)]
    info: Option<String>,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    link: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct HbIndex {
    languages: Vec<HbLanguage>,
}

// Legacy structure for backwards compatibility
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

fn extract_abbreviation(name: &str) -> String {
    // Extract common abbreviations from full names
    if name.contains("King James") { return "KJV".to_string(); }
    if name.contains("New International") { return "NIV".to_string(); }
    if name.contains("New King James") { return "NKJV".to_string(); }
    if name.contains("English Standard") { return "ESV".to_string(); }
    if name.contains("New American") { return "NASB".to_string(); }
    if name.contains("Revised Standard") { return "RSV".to_string(); }
    if name.contains("American Standard") { return "ASV".to_string(); }
    if name.contains("New Living") { return "NLT".to_string(); }
    
    // For other translations, use first letters of significant words
    let words: Vec<&str> = name.split_whitespace()
        .filter(|w| !["Bible", "Version", "Translation", "The", "of", "and"].contains(w))
        .take(3)
        .collect();
    
    if words.is_empty() {
        name.chars().take(3).collect::<String>().to_uppercase()
    } else {
        words.iter()
            .map(|w| w.chars().next().unwrap_or('?').to_ascii_uppercase())
            .collect()
    }
}

// Removed unused ServiceManager wrapper