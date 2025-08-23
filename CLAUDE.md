# StudyBible Dioxus Implementation - Todo List

## Project Overview
Converting the existing StudyBible project from React/Tauri to a native Dioxus (Rust) application. The original project is a comprehensive Bible study application with features like multi-translation support, highlighting, bookmarks, search, and parallel view.

## Original Project Analysis
- **Framework**: React 19.1 + TypeScript + Tauri v2
- **Features**: Multi-translation Bible reading, text highlighting, bookmarks, search, parallel view, zoom controls, dark/light theme
- **Data**: KJV, NIV, NKJV, Tamil Bible translations in JSON format
- **UI**: Glass morphism design with Tailwind CSS
- **Architecture**: SQLite backend with Rust, React frontend

## Target Implementation
- **Framework**: Dioxus 0.6+ (Rust native)
- **Platform**: Desktop-first with cross-platform support
- **Features**: All original features recreated in Dioxus
- **Performance**: Native Rust performance benefits

## Implementation Todo List

### Phase 1: Setup and Foundation
- [ ] Install Dioxus CLI (cargo binstall dioxus-cli@0.7.0-rc.0 --force)
- [ ] Create new Dioxus project structure for StudyBible
- [ ] Copy bible translations data from original project to new Dioxus project
- [ ] Set up project dependencies and Cargo.toml configuration

### Phase 2: Core Data Structures
- [ ] Create core data types and structures (Translation, Book, Verse, etc.)
- [ ] Implement Bible data loading and management system
- [ ] Create main app component with routing and state management

### Phase 3: Basic UI Components
- [ ] Implement Bible reader UI components (verse display, chapter navigation)
- [ ] Add sidebar component with book selection and navigation
- [ ] Create settings modal for preferences and customization

### Phase 4: Advanced Features
- [ ] Implement search functionality across translations
- [ ] Add bookmarking system for saving favorite verses
- [ ] Implement text highlighting feature with color options
- [ ] Add parallel view for side-by-side translation comparison

### Phase 5: User Experience
- [ ] Implement zoom controls and accessibility features
- [ ] Add responsive design and mobile support
- [ ] Style the application with modern UI design

### Phase 6: Testing and Polish
- [ ] Test desktop application functionality
- [ ] Test cross-platform compatibility
- [ ] Add error handling and loading states

## Key Files to Reference from Original Project
- `src/types/bible.ts` - Core data type definitions
- `src/App.tsx` - Main application structure and state management
- `src/lib/enhancedBibleApi.ts` - Bible data API
- `src/components/` - All UI components
- `public/data/` - Bible translation JSON files

## Dioxus Installation Commands
```bash
# Install Dioxus CLI
cargo binstall dioxus-cli@0.7.0-rc.0 --force

# Alternative installation
cargo install --git https://github.com/DioxusLabs/dioxus dioxus-cli --locked

# Create new project
dx new study-bible

# Run project
dx serve
```

## Key Dioxus Concepts to Implement
- **Components**: Using `#[component]` and `rsx!` macro
- **State Management**: Using `use_signal()` for reactive state
- **Hooks**: Similar to React hooks but Rust-native
- **Styling**: CSS-in-Rust or external stylesheets
- **Desktop Platform**: Native desktop app without web wrapper

## Next Steps
1. Install Dioxus CLI when internet connection improves
2. Create project structure
3. Start with basic Bible reader functionality
4. Incrementally add features following the todo list

## Notes
- Original project has excellent glass morphism UI that should be recreated
- Focus on desktop experience first, then expand to other platforms
- Maintain all functionality from original React version
- Take advantage of Rust's performance and safety benefits