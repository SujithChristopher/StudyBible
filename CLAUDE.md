# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
StudyBible is a native Rust desktop Bible study application built with Dioxus 0.7. It features multi-translation support, text highlighting, bookmarks, search functionality, and parallel view for comparing translations. The project was migrated from React/Tauri to pure Dioxus for native performance.

## Development Commands

### Building and Running
```bash
# Run development server (hot reload enabled)
dx serve

# Build for desktop release
dx build --release

# Clean build artifacts
cargo clean
```

### CSS Development
```bash
# Build Tailwind CSS
npm run build:css

# Watch for CSS changes
npm run watch:css
```

### Dependencies Management
```bash
# Add Rust dependency
cargo add <dependency>

# Check for compilation errors
cargo check

# Install Dioxus CLI (if needed)
cargo binstall dioxus-cli@0.7.0-rc.0 --force
```

## Architecture Overview

### Core Application Structure
- **Entry Point**: [src/main.rs](src/main.rs) - Contains the main `App` component with all state management
- **Type Definitions**: [src/types.rs](src/types.rs) - Core data structures for Bible content and UI state
- **Services**: [src/services.rs](src/services.rs) - `BibleService` handles data loading and caching operations
- **Components**: [src/components/](src/components/) - UI components organized by functionality
- **Data**: [src/data/](src/data/) - Embedded Bible translations in JSON format

### State Management Pattern
The application uses Dioxus signals for reactive state management:
- `use_signal()` creates reactive state values
- State is passed down through component props
- Event handlers update state via `EventHandler<T>` callbacks
- All Bible data loading is asynchronous using `spawn()` for non-blocking operations

### Data Architecture
- **Embedded Data**: Bible translations are embedded as JSON files in `src/data/`
- **Caching Layer**: `BibleService` implements in-memory caching for books and verses
- **Async Loading**: Data loading uses Rust's async/await with error handling
- **Translation Support**: KJV, NIV, NKJV, and Tamil translations are bundled

### Component Architecture
- **Layout Components**: `Header` and `Sidebar` provide navigation and controls
- **Modular Design**: Components receive data and event handlers as props
- **Responsive UI**: Uses Tailwind CSS classes with dark mode support
- **Theme System**: CSS custom properties in `assets/main.css` for theming

## Key Implementation Details

### Bible Data Loading Flow

1. App initialization loads translations index from [src/data/translations_index.json](src/data/translations_index.json)
2. First translation is selected automatically
3. Books are loaded and cached for the selected translation
4. First chapter of first book is loaded
5. User interactions trigger additional data loading via event handlers

### Search Implementation

- Limited scope search (first 5 books, first 3 chapters) for performance
- Case-insensitive text matching in [src/services.rs:120-144](src/services.rs#L120-L144)
- Results navigate to matching verses automatically

### Parallel View Feature

- Supports side-by-side translation comparison
- Column-based and verse-by-verse layout options
- Secondary translation loaded independently when parallel view is enabled

### Styling Approach

- Tailwind CSS v4 for utility-first styling
- Custom CSS properties for theme variables in [assets/main.css](assets/main.css)
- Glass morphism effects via backdrop-blur and transparency
- Dark/light theme support via CSS custom properties and `.dark` class

## File Organization

### Source Structure
```
src/
├── main.rs              # App component and main entry point
├── types.rs             # Data structures and enums
├── services.rs          # BibleService and data management
├── components/
│   ├── layout/          # Header, Sidebar components
│   ├── modals/          # Modal components (future)
│   └── ui/              # Reusable UI components (future)
└── data/               # Embedded Bible JSON files
```

### Asset Files

- [assets/main.css](assets/main.css) - Custom CSS and theme variables
- [assets/tailwind.css](assets/tailwind.css) - Generated Tailwind styles
- [input.css](input.css) - Tailwind source file

## Development Guidelines

### Adding New Translations

1. Add JSON files to [src/data/](src/data/) directory (e.g., `translation_id_verses.json`, `translation_id_metadata.json`)
2. Update [src/data/translations_index.json](src/data/translations_index.json) with translation metadata
3. Modify `BibleService::load_books()` and `load_verses()` match statements in [src/services.rs](src/services.rs)
4. Test data loading and display

### Component Development

- Follow Dioxus component patterns with `#[component]` attribute
- Use `rsx!` macro for JSX-like syntax
- Pass event handlers as `EventHandler<T>` props
- Implement proper error states and loading indicators

### State Updates

- Use `spawn()` for async operations that update state
- Handle errors gracefully with user-friendly messages
- Cache data appropriately to avoid unnecessary reloading
- Update dependent state when primary selections change (e.g., secondary translation when changing chapters)

### CSS and Theming

- Use Tailwind utility classes for styling
- Reference theme variables from [assets/main.css](assets/main.css) (`var(--bg-primary)`, etc.)
- Support both dark and light modes via `.dark` class scope
- Maintain responsive design principles with mobile-first approach

## Platform and Build Configuration

### Dioxus Configuration ([Dioxus.toml](Dioxus.toml))

- Desktop-first application with cross-platform support
- Bundle configuration for Windows, macOS, Linux, and Android
- Application identifier: `com.studybible.app`
- Default platform: `desktop`

### Cargo Dependencies ([Cargo.toml](Cargo.toml))

- Dioxus 0.7.0-rc.0 with desktop and mobile features
- Serde for JSON serialization/deserialization
- Tokio for async runtime
- Theme crate for additional styling utilities
- Chrono for date/time handling

## Android Development

### Build Configuration

- Android bundle is configured in [Dioxus.toml](Dioxus.toml) with keystore settings
- Package name: `com.studybible.app`
- Keystore file: `studybible-release-key.keystore`

### Java Version Requirement

- **IMPORTANT**: Android builds require Java 17 (not Java 24)
- Java 17 path on Windows: `C:\Program Files\Eclipse Adoptium\jdk-17.0.11.9-hotspot`
- Set `JAVA_HOME` to Java 17 before running Android builds

### Android Build Command

```bash
# On Windows (PowerShell or CMD)
set JAVA_HOME=C:\Program Files\Eclipse Adoptium\jdk-17.0.11.9-hotspot
dx serve --platform android
```

The application is designed as a comprehensive Bible study tool with room for expansion into bookmarking, highlighting, and other advanced features while maintaining excellent performance through native Rust implementation.
