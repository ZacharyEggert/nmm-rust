## Implementation Phases

### Phase 1: Core Foundation (Months 1-3)

**Goals**: Establish project structure, core traits, and essential infrastructure.

#### Month 1: Project Setup & Core Types

- [ ] Initialize Cargo workspace
- [ ] Set up CI/CD (GitHub Actions)
  - Build matrix: Windows, macOS, Linux
  - Test coverage reporting
  - Release automation
- [ ] Implement `nmm-core` crate
  - `ModInfo` struct with serde support
  - `GameModeDescriptor` and `GameMode` traits
  - `ModFormat` trait and `FormatConfidence` enum
  - Error types with thiserror
- [ ] Set up integration test framework

#### Month 2: Install Log with SQLite

- [ ] Design SQLite schema for install log
  ```sql
  CREATE TABLE mods (
      key TEXT PRIMARY KEY,
      path TEXT NOT NULL,
      name TEXT NOT NULL,
      version TEXT,
      machine_version TEXT,
      install_date TEXT
  );

  CREATE TABLE file_owners (
      path TEXT NOT NULL,
      mod_key TEXT NOT NULL,
      install_order INTEGER NOT NULL,
      PRIMARY KEY (path, mod_key),
      FOREIGN KEY (mod_key) REFERENCES mods(key)
  );

  CREATE TABLE ini_edits (
      file TEXT NOT NULL,
      section TEXT NOT NULL,
      key TEXT NOT NULL,
      mod_key TEXT NOT NULL,
      value TEXT NOT NULL,
      install_order INTEGER NOT NULL,
      PRIMARY KEY (file, section, key, mod_key),
      FOREIGN KEY (mod_key) REFERENCES mods(key)
  );
  ```
- [ ] Implement `InstallLog` trait
- [ ] Add transaction support with SQLite transactions
- [ ] Write XML migration tool (import from C# NMM)
- [ ] Unit tests for ownership tracking

#### Month 3: Virtual File System

- [ ] Implement symlink/hardlink abstraction
  ```rust
  pub fn create_link(source: &Path, target: &Path, link_type: LinkType) -> Result<(), VfsError>;

  pub enum LinkType {
      Symlink,
      Hardlink,
      Copy,  // Fallback when links not available
  }
  ```
- [ ] Windows symlink handling (Developer Mode detection)
- [ ] macOS symlink handling
- [ ] Multi-drive hardlink support
- [ ] Priority resolution algorithm
- [ ] VirtualModConfig.xml parser/writer
- [ ] Integration tests with temp directories

### Phase 2: Game Support (Months 4-5)

**Goals**: Implement game mode infrastructure and priority game support.

#### Month 4: Game Mode Infrastructure

- [ ] Implement `nmm-game-modes` crate
  - Game detection (registry on Windows, paths on macOS)
  - Game mode registry
- [ ] Implement `nmm-games` crate structure
- [ ] `GamebryoGameModeBase` equivalent
  - INI file handling
  - Plugin directory management
  - Path adjustment for legacy mods
- [ ] Steam library path detection

#### Month 5: Priority Games

- [ ] Skyrim Special Edition
  - Plugin management (.esp, .esm, .esl)
  - plugins.txt and loadorder.txt handling
  - SKSE detection
- [ ] Fallout 4
  - Similar to Skyrim SE
  - F4SE detection
- [ ] Starfield
  - Updated plugin format handling
  - SFSE detection
- [ ] Basic plugin sorting (LOOT masterlist parsing)

### Phase 3: Archive & Scripting (Months 6-7)

**Goals**: Mod archive handling and installation script execution.

#### Month 6: Archive Handling

- [ ] Implement `nmm-archive` crate
- [ ] FOMod format support
  - fomod/info.xml parsing
  - fomod/ModuleConfig.xml detection
- [ ] OMod format support (legacy)
- [ ] Generic archive extraction (7z, zip, rar)
- [ ] Archive structure detection (stop folders)
- [ ] Mod root detection algorithm

#### Month 7: Scripting Engine

- [ ] Implement `nmm-scripting` crate
- [ ] XmlScript parser (versions 1.0-5.0)
  - Condition evaluation
  - File pattern matching
  - UI abstraction for options
- [ ] XmlScript executor
- [ ] ModScript interpreter (legacy, for Morrowind/Oblivion)
- [ ] WASM scripting foundation (wasmtime integration)
- [ ] Script sandboxing

### Phase 4: Desktop Application (Months 8-10)

**Goals**: Build cross-platform desktop UI with Tauri.

#### Month 8: Tauri Setup & Core UI

- [ ] Initialize Tauri project
- [ ] React frontend structure
- [ ] Tauri commands for:
  - Game mode selection
  - Mod list management
  - Install/uninstall operations
- [ ] Basic mod list view
- [ ] Mod details panel

#### Month 9: Advanced UI Features

- [ ] Plugin ordering interface (drag-and-drop)
- [ ] Profile selector and management
- [ ] Installation wizard (XmlScript UI)
- [ ] Settings panel
- [ ] Conflict detection and resolution UI

#### Month 10: Integration & Polish

- [ ] nxm:// protocol handler
  - Windows URL protocol registration
  - macOS URL scheme handling
- [ ] Nexus Mods API integration
  - Mod search
  - Download queue
  - Update checking
- [ ] Theme support (light/dark, game-specific)
- [ ] Keyboard shortcuts
- [ ] Accessibility improvements

### Phase 5: Polish & Migration (Months 11-12)

**Goals**: Migration tools, testing, and release preparation.

#### Month 11: Migration & Testing

- [ ] NMM installation importer
  - InstallLog.xml migration
  - VirtualModConfig.xml migration
  - Profile migration
- [ ] Vortex installation importer (optional)
- [ ] Comprehensive integration tests
- [ ] Performance benchmarking
- [ ] Memory usage optimization

#### Month 12: Documentation & Release

- [ ] User documentation
- [ ] Developer documentation (crate docs)
- [ ] Game mode development guide
- [ ] Beta testing program
- [ ] Release automation
- [ ] Installer/package creation
  - Windows MSI/MSIX
  - macOS DMG
  - Linux AppImage/Flatpak
