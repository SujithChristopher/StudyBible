use serde::{Deserialize, Serialize};

/// Core Bible data types and interfaces
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Translation {
    pub id: String,
    pub name: String,
    pub abbreviation: String,
    pub language: String,
    pub language_name: Option<String>,
    pub description: String,
    #[serde(default)]
    pub bundled: bool,
    #[serde(default)]
    pub priority: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub id: u32,
    pub name: String,
    pub abbreviation: String,
    pub testament: Testament,
    pub order_index: u32,
    pub chapter_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Verse {
    pub id: String,
    pub translation_id: String,
    pub book_id: u32,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerseWithBook {
    #[serde(flatten)]
    pub verse: Verse,
    pub book_name: String,
    pub book_abbreviation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub user_id: Option<String>,
    pub translation_id: String,
    pub book_id: u32,
    pub chapter: u32,
    pub verse: u32,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleReference {
    pub book_id: u32,
    pub chapter: u32,
    pub verse: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub verses: Vec<VerseWithBook>,
    pub total_count: usize,
    pub query: String,
    pub translation_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub translation_id: String,
    pub book_id: u32,
    pub chapter: u32,
    pub completed_verses: Vec<u32>,
    pub last_read_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextHighlight {
    pub id: String,
    pub user_id: Option<String>,
    pub translation_id: String,
    pub book_id: u32,
    pub chapter: u32,
    pub verse: u32,
    pub text: String, // The highlighted text
    pub color: HighlightColor,
    pub start_index: usize,
    pub end_index: usize,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HighlightColorOption {
    pub name: HighlightColor,
    pub label: String,
    pub bg: String,
    pub border: String,
    pub hover: String,
}

// Enums
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Testament {
    OT, // Old Testament
    NT, // New Testament
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerseDisplayMode {
    VerseByVerse,
    Paragraph,
    Parallel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranslationStatus {
    Core,
    Downloaded,
    Available,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HighlightColor {
    Yellow,
    Green,
    Blue,
    Pink,
    Purple,
}

impl HighlightColor {
    pub fn get_styles(&self) -> HighlightColorOption {
        match self {
            HighlightColor::Yellow => HighlightColorOption {
                name: HighlightColor::Yellow,
                label: "Yellow".to_string(),
                bg: "bg-yellow-200/60".to_string(),
                border: "border-yellow-300".to_string(),
                hover: "hover:bg-yellow-200/80".to_string(),
            },
            HighlightColor::Green => HighlightColorOption {
                name: HighlightColor::Green,
                label: "Green".to_string(),
                bg: "bg-green-200/60".to_string(),
                border: "border-green-300".to_string(),
                hover: "hover:bg-green-200/80".to_string(),
            },
            HighlightColor::Blue => HighlightColorOption {
                name: HighlightColor::Blue,
                label: "Blue".to_string(),
                bg: "bg-blue-200/60".to_string(),
                border: "border-blue-300".to_string(),
                hover: "hover:bg-blue-200/80".to_string(),
            },
            HighlightColor::Pink => HighlightColorOption {
                name: HighlightColor::Pink,
                label: "Pink".to_string(),
                bg: "bg-pink-200/60".to_string(),
                border: "border-pink-300".to_string(),
                hover: "hover:bg-pink-200/80".to_string(),
            },
            HighlightColor::Purple => HighlightColorOption {
                name: HighlightColor::Purple,
                label: "Purple".to_string(),
                bg: "bg-purple-200/60".to_string(),
                border: "border-purple-300".to_string(),
                hover: "hover:bg-purple-200/80".to_string(),
            },
        }
    }
}

/// Reader preferences for customizing the reading experience
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReaderPreferences {
    pub font_size: f32,
    pub line_height: f32,
    pub show_verse_badges: bool,
}

impl Default for ReaderPreferences {
    fn default() -> Self {
        Self {
            font_size: 18.0,
            line_height: 1.6,
            show_verse_badges: true,
        }
    }
}

/// App state for managing the current UI state
#[derive(Debug, Clone)]
pub struct AppState {
    pub selected_book: Option<Book>,
    pub selected_chapter: u32,
    pub selected_translation: Option<Translation>,
    pub secondary_translation: Option<Translation>,
    pub books: Vec<Book>,
    pub translations: Vec<Translation>,
    pub verses: Vec<Verse>,
    pub bookmarks: Vec<Bookmark>,
    pub highlights: Vec<TextHighlight>,
    pub preferences: ReaderPreferences,
    pub is_parallel_view: bool,
    pub is_sidebar_open: bool,
    pub is_dark_theme: bool,
    pub zoom_level: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            selected_book: None,
            selected_chapter: 1,
            selected_translation: None,
            secondary_translation: None,
            books: Vec::new(),
            translations: Vec::new(),
            verses: Vec::new(),
            bookmarks: Vec::new(),
            highlights: Vec::new(),
            preferences: ReaderPreferences::default(),
            is_parallel_view: false,
            is_sidebar_open: false,
            is_dark_theme: false,
            zoom_level: 1.0,
        }
    }
}