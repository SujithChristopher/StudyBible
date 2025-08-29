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
- **Entry Point**: `src/main.rs` - Contains the main `App` component with all state management
- **Type Definitions**: `src/types.rs` - Core data structures for Bible content and UI state
- **Services**: `src/services.rs` - `BibleService` handles data loading and caching operations
- **Components**: `src/components/` - UI components organized by functionality
- **Data**: `src/data/` - Embedded Bible translations in JSON format

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
1. App initialization loads translations index
2. First translation is selected automatically
3. Books are loaded and cached for the selected translation  
4. First chapter of first book is loaded
5. User interactions trigger additional data loading

### Search Implementation
- Limited scope search (first 5 books, first 3 chapters) for performance
- Case-insensitive text matching
- Results navigate to matching verses automatically

### Parallel View Feature
- Supports side-by-side translation comparison
- Column-based and verse-by-verse layout options
- Secondary translation loaded independently

### Styling Approach
- Tailwind CSS for utility-first styling
- Custom CSS properties for theme variables
- Glass morphism effects via backdrop-blur and transparency
- Dark/light theme support via CSS classes

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
- `assets/main.css` - Custom CSS and theme variables
- `assets/tailwind.css` - Generated Tailwind styles
- `input.css` - Tailwind source file

## Development Guidelines

### Adding New Translations
1. Add JSON files to `src/data/` directory
2. Update `translations_index.json` with translation metadata
3. Modify `BibleService::load_books()` and `load_verses()` match statements
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
- Update dependent state when primary selections change

### CSS and Theming
- Use Tailwind utility classes for styling
- Reference theme variables from `assets/main.css`
- Support both dark and light modes
- Maintain responsive design principles

## Platform and Build Configuration

### Dioxus Configuration (`Dioxus.toml`)
- Desktop-first application with cross-platform support
- Bundle configuration for Windows, macOS, and Linux
- Application identifier: `com.studybible.app`

### Cargo Dependencies
- Dioxus 0.7.0-rc.0 with desktop features
- Serde for JSON serialization/deserialization  
- Tokio for async runtime
- Theme crate for additional styling utilities
- Chrono for date/time handling

The application is designed as a comprehensive Bible study tool with room for expansion into bookmarking, highlighting, and other advanced features while maintaining excellent performance through native Rust implementation.

## Android Development Status

### Completed Setup Tasks:
- ✅ Added mobile features to Cargo.toml (`dioxus = { features = ["desktop", "mobile"] }`)
- ✅ Added Android bundle configuration to Dioxus.toml with keystore settings
- ✅ Generated Android keystore file (`studybible-release-key.keystore`)
- ✅ Verified Android SDK, NDK, and toolchains are installed

### Current Android Configuration:
```toml
[bundle.android]
name = "StudyBible"
package_name = "com.studybible.app"
jks_file = "studybible-release-key.keystore"
jks_password = "android123"
key_alias = "studybible"
key_password = "android123"
```

### Outstanding Issues:
- **Java Version Conflict**: Build fails with "Unsupported class file major version 68" 
  - Current: Java 24 (incompatible with Gradle)
  - Solution: Need to use Java 17 permanently for Android builds
  - Java 17 path: `C:\Program Files\Eclipse Adoptium\jdk-17.0.11.9-hotspot`

### Next Steps for Android Testing:
1. **Fix Java Environment**: Set JAVA_HOME to Java 17 permanently
2. **Complete Build**: Run `dx serve --platform android` with proper Java version
3. **Test APK**: Verify app functionality on Android emulator/device
4. **UI Optimization**: Adapt UI for mobile touch interactions

### Android Build Command:
```bash
# With proper Java 17 environment
export JAVA_HOME="C:\Program Files\Eclipse Adoptium\jdk-17.0.11.9-hotspot"
dx serve --platform android
```