# Nexus Mod Manager Rust Implementation Plan

## Overview

This document outlines the plan to rewrite Nexus Mod Manager from C#/.NET Framework 4.6.1 to Rust, with cross-platform support for Windows and macOS.

## [Project Structure](./project_structure.md)

```
nexus-mod-manager-rs/
├── Cargo.toml                        # Workspace root
├── crates/
│   ├── nmm-core/                     # Core domain types and traits
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mod_info.rs           # IModInfo equivalent
│   │   │   ├── game_mode.rs          # IGameMode/IGameModeDescriptor
│   │   │   ├── mod_format.rs         # IModFormat equivalent
│   │   │   ├── error.rs              # Error types
│   │   │   └── prelude.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-vfs/                      # Virtual file system
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── activator.rs          # IVirtualModActivator equivalent
│   │   │   ├── link.rs               # Symlink/hardlink operations
│   │   │   ├── priority.rs           # Priority resolution
│   │   │   └── config.rs             # VirtualModConfig.xml handling
│   │   └── Cargo.toml
│   │
│   ├── nmm-install-log/              # Installation logging
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── log.rs                # IInstallLog equivalent
│   │   │   ├── transaction.rs        # Transaction support
│   │   │   ├── file_ownership.rs     # File ownership tracking
│   │   │   ├── ini_edits.rs          # INI edit tracking
│   │   │   └── migrations.rs         # Log format migrations
│   │   └── Cargo.toml
│   │
│   ├── nmm-archive/                  # Archive/mod format handling
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── formats/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── fomod.rs          # FOMod format
│   │   │   │   ├── omod.rs           # OMod format
│   │   │   │   └── archive.rs        # Generic archive (7z, zip, rar)
│   │   │   ├── extractor.rs
│   │   │   └── compressor.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-scripting/                # Script execution
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── xml_script/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── parser.rs         # XML script parser
│   │   │   │   ├── executor.rs       # Script executor
│   │   │   │   ├── conditions.rs     # Condition evaluation
│   │   │   │   └── ui.rs             # UI abstraction for options
│   │   │   ├── mod_script.rs         # Legacy ModScript interpreter
│   │   │   └── wasm/                 # WASM scripting (replaces C#)
│   │   │       ├── mod.rs
│   │   │       └── runtime.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-plugin-manager/           # Game plugin management
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── plugin.rs             # Plugin representation
│   │   │   ├── order.rs              # Load order management
│   │   │   ├── validator.rs          # Order validation
│   │   │   ├── discovery.rs          # Plugin discovery
│   │   │   ├── sorter.rs             # LOOT-based sorting
│   │   │   └── formats/
│   │   │       ├── mod.rs
│   │   │       ├── bethesda.rs       # .esp/.esm/.esl handling
│   │   │       └── plugins_txt.rs    # plugins.txt parser
│   │   └── Cargo.toml
│   │
│   ├── nmm-game-modes/               # Game mode plugin system
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── registry.rs           # Game mode registry
│   │   │   ├── detection.rs          # Game installation detection
│   │   │   └── gamebryo/             # Gamebryo engine base
│   │   │       ├── mod.rs
│   │   │       ├── base.rs           # GamebryoGameModeBase equivalent
│   │   │       ├── ini.rs            # INI file handling
│   │   │       └── settings.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-games/                    # Built-in game implementations
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── skyrim/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── skyrim_le.rs
│   │   │   │   ├── skyrim_se.rs
│   │   │   │   ├── skyrim_vr.rs
│   │   │   │   └── skyrim_gog.rs
│   │   │   ├── fallout/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── fallout3.rs
│   │   │   │   ├── fallout_nv.rs
│   │   │   │   ├── fallout4.rs
│   │   │   │   └── fallout4_vr.rs
│   │   │   ├── oblivion/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── oblivion.rs
│   │   │   │   └── oblivion_remastered.rs
│   │   │   ├── starfield.rs
│   │   │   ├── morrowind.rs
│   │   │   ├── witcher3.rs
│   │   │   ├── cyberpunk2077.rs
│   │   │   ├── baldurs_gate3.rs
│   │   │   └── [other games...]
│   │   └── Cargo.toml
│   │
│   ├── nmm-nexus-api/                # Nexus Mods API client
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs
│   │   │   ├── auth.rs               # API key / SSO
│   │   │   ├── endpoints/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── mods.rs
│   │   │   │   ├── files.rs
│   │   │   │   └── user.rs
│   │   │   └── models.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-profiles/                 # Profile management
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── profile.rs
│   │   │   ├── export.rs
│   │   │   └── import.rs
│   │   └── Cargo.toml
│   │
│   └── nmm-cli/                      # CLI application
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/
│       │   │   ├── mod.rs
│       │   │   ├── install.rs
│       │   │   ├── activate.rs
│       │   │   ├── profile.rs
│       │   │   └── list.rs
│       │   └── config.rs
│       └── Cargo.toml
│
├── apps/
│   └── nmm-desktop/                  # Tauri desktop application
│       ├── src-tauri/
│       │   ├── src/
│       │   │   ├── main.rs
│       │   │   ├── commands.rs       # Tauri commands
│       │   │   └── state.rs          # Application state
│       │   ├── Cargo.toml
│       │   └── tauri.conf.json
│       ├── src/                      # React frontend
│       │   ├── App.tsx
│       │   ├── components/
│       │   │   ├── ModList.tsx
│       │   │   ├── PluginOrder.tsx
│       │   │   ├── ProfileSelector.tsx
│       │   │   └── InstallWizard.tsx
│       │   └── hooks/
│       ├── package.json
│       └── vite.config.ts
│
└── tests/
    └── integration/
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            ├── install_tests.rs
            ├── profile_tests.rs
            └── fixtures/
```

## [Core Trait Definitions](./core_trait_definitions.md)

### Game Mode Abstraction

```rust
// nmm-core/src/game_mode.rs

use std::path::{Path, PathBuf};
use std::collections::HashSet;

/// Static game metadata (equivalent to IGameModeDescriptor)
pub trait GameModeDescriptor: Send + Sync {
    /// Unique identifier for this game mode (e.g., "SkyrimSE")
    fn mode_id(&self) -> &str;

    /// Display name (e.g., "Skyrim Special Edition")
    fn name(&self) -> &str;

    /// Possible game executable names
    fn game_executables(&self) -> &[&str];

    /// Plugin file extensions (e.g., [".esp", ".esm", ".esl"])
    fn plugin_extensions(&self) -> &[&str];

    /// Critical plugins that cannot be disabled or reordered
    fn critical_plugins(&self) -> &[&str];

    /// Official DLC plugins in load order
    fn official_plugins(&self) -> &[&str];

    /// Folders that indicate mod root in archives
    fn stop_folders(&self) -> &[&str];

    /// Theme/branding for UI
    fn theme(&self) -> GameTheme;

    /// Maximum active plugins (0 = unlimited)
    fn max_active_plugins(&self) -> u32 {
        0
    }
}

/// Runtime game mode (equivalent to IGameMode)
pub trait GameMode: GameModeDescriptor {
    /// Path where game is installed
    fn installation_path(&self) -> &Path;

    /// Secondary installation path (if applicable)
    fn secondary_installation_path(&self) -> Option<&Path> {
        None
    }

    /// Directory where plugins are stored
    fn plugin_directory(&self) -> PathBuf;

    /// Whether this game uses plugins
    fn uses_plugins(&self) -> bool;

    /// Whether plugin auto-sorting is supported
    fn supports_plugin_auto_sorting(&self) -> bool {
        false
    }

    /// Get the plugin factory for this game
    fn plugin_factory(&self) -> Option<Box<dyn PluginFactory>>;

    /// Get the plugin order validator
    fn plugin_order_validator(&self) -> Option<Box<dyn PluginOrderValidator>>;

    /// Get the load order manager
    fn load_order_manager(&self) -> Option<Box<dyn LoadOrderManager>>;

    /// Check if a plugin is critical (cannot be disabled/reordered)
    fn is_critical_plugin(&self, plugin_name: &str) -> bool {
        self.critical_plugins()
            .iter()
            .any(|p| p.eq_ignore_ascii_case(plugin_name))
    }

    /// Adjust path for mod format compatibility (legacy FOMod support)
    fn adjust_mod_path(&self, format_id: &str, path: &str, ignore_if_present: bool) -> String {
        path.to_string()
    }

    /// File types that require hardlinks instead of symlinks
    fn hardlink_required_extensions(&self) -> HashSet<&str> {
        HashSet::new()
    }

    /// Settings/INI files for this game
    fn settings_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}

/// Game theme for UI customization
#[derive(Debug, Clone)]
pub struct GameTheme {
    pub primary_color: String,
    pub icon_path: Option<PathBuf>,
}
```

### Mod Abstraction

```rust
// nmm-core/src/mod_info.rs

use std::path::Path;
use chrono::{DateTime, Utc};

/// Mod metadata (equivalent to IModInfo)
#[derive(Debug, Clone)]
pub struct ModInfo {
    pub id: Option<String>,
    pub download_id: Option<String>,
    pub name: String,
    pub file_name: String,
    pub version: String,
    pub machine_version: Option<semver::Version>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub website: Option<url::Url>,
    pub download_date: Option<DateTime<Utc>>,
    pub install_date: Option<DateTime<Utc>>,
    pub is_endorsed: Option<bool>,
    pub load_order: Option<i32>,
}

/// Mod archive abstraction (equivalent to IMod)
pub trait Mod: Send + Sync {
    /// Get mod metadata
    fn info(&self) -> &ModInfo;

    /// Path to the mod archive
    fn archive_path(&self) -> &Path;

    /// Format identifier (e.g., "FOMod", "OMod")
    fn format_id(&self) -> &str;

    /// List all files in the mod
    fn file_list(&self) -> Result<Vec<String>, ModError>;

    /// List files in a specific folder
    fn file_list_in_folder(&self, folder: &str, recursive: bool) -> Result<Vec<String>, ModError>;

    /// Read a file from the mod archive
    fn read_file(&self, path: &str) -> Result<Vec<u8>, ModError>;

    /// Get a readable stream for a file
    fn read_file_stream(&self, path: &str) -> Result<Box<dyn std::io::Read>, ModError>;

    /// Check if mod has an installation script
    fn has_script(&self) -> bool;

    /// Get the script content if present
    fn script_content(&self) -> Option<(ScriptType, String)>;

    /// Path to screenshot within archive
    fn screenshot_path(&self) -> Option<&str>;
}

#[derive(Debug, Clone)]
pub enum ScriptType {
    XmlScript,
    ModScript,
    Wasm,
}

#[derive(Debug, thiserror::Error)]
pub enum ModError {
    #[error("File not found in mod: {0}")]
    FileNotFound(String),

    #[error("Failed to read archive: {0}")]
    ArchiveError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Mod Format Abstraction

```rust
// nmm-core/src/mod_format.rs

use std::path::Path;

/// Confidence level for format detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatConfidence {
    Incompatible = 0,
    Convertible = 1,
    Compatible = 2,
    Match = 3,
}

/// Mod archive format handler (equivalent to IModFormat)
pub trait ModFormat: Send + Sync {
    /// Format name for display
    fn name(&self) -> &str;

    /// Unique format identifier
    fn id(&self) -> &str;

    /// File extension (e.g., ".fomod", ".omod")
    fn extension(&self) -> &str;

    /// Whether this format supports creating mods
    fn supports_compression(&self) -> bool;

    /// Check if a file matches this format
    fn check_compliance(&self, path: &Path) -> FormatConfidence;

    /// Create a Mod instance from this format
    fn create_mod(
        &self,
        path: &Path,
        game_mode: &dyn GameMode,
    ) -> Result<Box<dyn Mod>, ModFormatError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ModFormatError {
    #[error("Unsupported format")]
    UnsupportedFormat,

    #[error("Corrupt archive: {0}")]
    CorruptArchive(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Virtual File System

```rust
// nmm-vfs/src/activator.rs

use std::path::{Path, PathBuf};
use nmm_core::{Mod, ModInfo};

/// File link in the virtual file system
#[derive(Debug, Clone)]
pub struct VirtualModLink {
    pub mod_info: VirtualModInfo,
    pub real_path: PathBuf,      // Path in VirtualInstall folder
    pub virtual_path: PathBuf,   // Path relative to game folder
    pub priority: i32,
    pub active: bool,
}

/// Mod information for virtual links
#[derive(Debug, Clone)]
pub struct VirtualModInfo {
    pub mod_id: Option<String>,
    pub download_id: Option<String>,
    pub mod_name: String,
    pub mod_file_name: String,
    pub mod_file_path: PathBuf,
    pub file_version: String,
}

/// Virtual file system activator (equivalent to IVirtualModActivator)
pub trait VirtualModActivator: Send + Sync {
    /// Initialize the activator
    fn initialize(&mut self) -> Result<(), VfsError>;

    /// Check if initialized
    fn is_initialized(&self) -> bool;

    /// Virtual install path
    fn virtual_path(&self) -> &Path;

    /// Get all virtual links
    fn links(&self) -> &[VirtualModLink];

    /// Get all virtual mods
    fn mods(&self) -> &[VirtualModInfo];

    /// Add a file link for a mod
    fn add_file_link(
        &mut self,
        mod_info: &dyn Mod,
        source_path: &Path,
        dest_path: &Path,
        priority: i32,
    ) -> Result<(), VfsError>;

    /// Remove a file link
    fn remove_file_link(&mut self, virtual_path: &Path, mod_info: &dyn Mod) -> Result<(), VfsError>;

    /// Enable a mod (create all its links)
    fn enable_mod(&mut self, mod_info: &dyn Mod) -> Result<(), VfsError>;

    /// Disable a mod (remove all its links)
    fn disable_mod(&mut self, mod_info: &dyn Mod) -> Result<(), VfsError>;

    /// Get the current owner of a file
    fn get_file_owner(&self, path: &Path) -> Option<&VirtualModInfo>;

    /// Update link priorities
    fn update_priorities(&mut self, links: &[VirtualModLink]) -> Result<(), VfsError>;

    /// Purge all links
    fn purge_links(&mut self) -> Result<(), VfsError>;

    /// Save configuration
    fn save(&self) -> Result<(), VfsError>;

    /// Load configuration
    fn load(&mut self, path: &Path) -> Result<(), VfsError>;
}

#[derive(Debug, thiserror::Error)]
pub enum VfsError {
    #[error("Not initialized")]
    NotInitialized,

    #[error("Link creation failed: {0}")]
    LinkFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

### Installation Log

```rust
// nmm-install-log/src/log.rs

use std::path::Path;
use nmm_core::{Mod, ModInfo};

/// INI edit record
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IniEdit {
    pub file: String,
    pub section: String,
    pub key: String,
}

/// Installation log (equivalent to IInstallLog)
pub trait InstallLog: Send + Sync {
    /// Add a mod to the active mods list
    fn add_active_mod(&mut self, mod_info: &dyn Mod) -> Result<String, InstallLogError>;

    /// Remove a mod from active mods
    fn remove_mod(&mut self, mod_info: &dyn Mod) -> Result<(), InstallLogError>;

    /// Get the key for a mod
    fn get_mod_key(&self, mod_info: &dyn Mod) -> Option<String>;

    /// Get all active mods
    fn active_mods(&self) -> &[ModInfo];

    // File tracking
    fn add_data_file(&mut self, mod_key: &str, path: &Path) -> Result<(), InstallLogError>;
    fn remove_data_file(&mut self, mod_key: &str, path: &Path) -> Result<(), InstallLogError>;
    fn get_current_file_owner(&self, path: &Path) -> Option<&ModInfo>;
    fn get_previous_file_owner(&self, path: &Path) -> Option<&ModInfo>;
    fn get_file_installers(&self, path: &Path) -> Vec<&ModInfo>;
    fn get_installed_files(&self, mod_key: &str) -> Vec<&Path>;

    // INI tracking
    fn add_ini_edit(
        &mut self,
        mod_key: &str,
        file: &str,
        section: &str,
        key: &str,
        value: &str,
    ) -> Result<(), InstallLogError>;
    fn remove_ini_edit(
        &mut self,
        mod_key: &str,
        file: &str,
        section: &str,
        key: &str,
    ) -> Result<(), InstallLogError>;
    fn get_current_ini_owner(&self, file: &str, section: &str, key: &str) -> Option<&ModInfo>;
    fn get_previous_ini_value(&self, file: &str, section: &str, key: &str) -> Option<&str>;

    // Persistence
    fn save(&self) -> Result<(), InstallLogError>;
    fn backup(&self) -> Result<(), InstallLogError>;
}

#[derive(Debug, thiserror::Error)]
pub enum InstallLogError {
    #[error("Mod not found: {0}")]
    ModNotFound(String),

    #[error("Duplicate mod key")]
    DuplicateKey,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(String),
}
```

### Plugin Management

```rust
// nmm-plugin-manager/src/plugin.rs

use std::path::{Path, PathBuf};

/// Game plugin representation
#[derive(Debug, Clone)]
pub struct Plugin {
    pub path: PathBuf,
    pub filename: String,
    pub is_master: bool,          // .esm
    pub is_light: bool,           // .esl
    pub masters: Vec<String>,     // Required master plugins
    pub description: Option<String>,
    pub author: Option<String>,
}

/// Plugin factory (equivalent to IPluginFactory)
pub trait PluginFactory: Send + Sync {
    /// Create a plugin from a file path
    fn create_plugin(&self, path: &Path) -> Result<Plugin, PluginError>;

    /// Check if a file is a valid plugin
    fn is_plugin(&self, path: &Path) -> bool;
}

/// Plugin order validator
pub trait PluginOrderValidator: Send + Sync {
    /// Validate a plugin order
    fn validate(&self, plugins: &[Plugin]) -> bool;

    /// Correct an invalid order
    fn correct_order(&self, plugins: &mut Vec<Plugin>);
}

/// Plugin order manager
pub trait LoadOrderManager: Send + Sync {
    /// Get the current load order
    fn get_load_order(&self) -> Result<Vec<Plugin>, PluginError>;

    /// Set the load order
    fn set_load_order(&mut self, plugins: &[Plugin]) -> Result<(), PluginError>;

    /// Activate a plugin
    fn activate(&mut self, plugin: &Plugin) -> Result<(), PluginError>;

    /// Deactivate a plugin
    fn deactivate(&mut self, plugin: &Plugin) -> Result<(), PluginError>;

    /// Get active plugins
    fn active_plugins(&self) -> Vec<&Plugin>;
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Invalid plugin: {0}")]
    Invalid(String),

    #[error("Plugin not found: {0}")]
    NotFound(PathBuf),

    #[error("Missing master: {0}")]
    MissingMaster(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## [Recommended Crate Ecosystem](./recommended_crate_ecosystem.md)

### Core Dependencies

| Purpose | Crate | Notes |
|---------|-------|-------|
| Async Runtime | `tokio` | Full features for async operations |
| Error Handling | `thiserror`, `anyhow` | thiserror for library errors, anyhow for application |
| Serialization | `serde`, `serde_json` | JSON config files |
| XML | `quick-xml`, `serde-xml-rs` | VirtualModConfig.xml, InstallLog.xml, XmlScript |
| Logging | `tracing`, `tracing-subscriber` | Structured logging with async support |
| CLI | `clap` | Command-line argument parsing |

### Archive Handling

| Purpose | Crate | Notes |
|---------|-------|-------|
| ZIP | `zip` | Standard zip archives |
| 7z | `sevenz-rust` | 7-Zip format (most common for mods) |
| RAR | `unrar` | RAR archives (requires libunrar) |
| Compression | `flate2`, `lz4`, `zstd` | Various compression algorithms |

### Database & Storage

| Purpose | Crate | Notes |
|---------|-------|-------|
| SQLite | `rusqlite` | Install log storage (upgrade from XML) |
| Migrations | `refinery` | Database schema migrations |
| KV Store | `sled` | Fast embedded database for caching |

### File System

| Purpose | Crate | Notes |
|---------|-------|-------|
| Path Utils | `pathdiff`, `normpath` | Path manipulation |
| File Watching | `notify` | Watch for file changes |
| Temp Files | `tempfile` | Temporary file handling |
| Directory Walking | `walkdir`, `ignore` | Recursive directory traversal |

### Platform-Specific

| Purpose | Crate | Notes |
|---------|-------|-------|
| Windows APIs | `windows` | Official Microsoft Windows crate |
| macOS APIs | `core-foundation` | macOS framework bindings |
| Registry | `winreg` | Windows registry access |
| Symlinks | `symlink` | Cross-platform symlink creation |

### UI Framework

| Purpose | Crate | Notes |
|---------|-------|-------|
| Desktop UI | `tauri` | Cross-platform desktop with web frontend |
| Frontend | React + TypeScript | Modern, familiar web stack |
| Build | Vite | Fast frontend bundling |

### Scripting

| Purpose | Crate | Notes |
|---------|-------|-------|
| WASM Runtime | `wasmtime` | Run WASM scripts (replaces C# scripts) |
| Lua | `mlua` | Optional Lua scripting support |

### Networking

| Purpose | Crate | Notes |
|---------|-------|-------|
| HTTP Client | `reqwest` | Async HTTP client for Nexus API |
| URL Handling | `url` | URL parsing and manipulation |

### Testing

| Purpose | Crate | Notes |
|---------|-------|-------|
| Assertions | `assert_fs`, `predicates` | File system testing |
| Mocking | `mockall` | Mock trait implementations |
| Property Testing | `proptest` | Property-based testing |

## [Implementation Phases](./implementation_phases.md)

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

## [Cross-Platform Considerations](./cross-platform_considerations.md)

### Windows

```rust
#[cfg(target_os = "windows")]
mod windows {
    use windows::Win32::Storage::FileSystem::{
        CreateSymbolicLinkW, CreateHardLinkW, SYMBOLIC_LINK_FLAG_DIRECTORY,
    };

    pub fn create_symlink(source: &Path, target: &Path) -> Result<(), VfsError> {
        // Check if Developer Mode is enabled
        if !is_developer_mode_enabled() {
            return Err(VfsError::PermissionDenied(
                "Developer Mode required for symlinks".into()
            ));
        }
        // Create symlink using Windows API
        unsafe {
            CreateSymbolicLinkW(/* ... */);
        }
    }

    pub fn is_developer_mode_enabled() -> bool {
        // Check registry key
        let key = winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock")
            .ok();
        key.and_then(|k| k.get_value::<u32, _>("AllowDevelopmentWithoutDevLicense").ok())
            .map(|v| v == 1)
            .unwrap_or(false)
    }
}
```

### macOS

```rust
#[cfg(target_os = "macos")]
mod macos {
    use std::os::unix::fs::symlink;

    pub fn create_symlink(source: &Path, target: &Path) -> Result<(), VfsError> {
        // Standard POSIX symlinks work without special permissions
        symlink(source, target).map_err(VfsError::from)
    }

    pub fn detect_steam_library() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let steam_path = home.join("Library/Application Support/Steam/steamapps/common");
        if steam_path.exists() {
            Some(steam_path)
        } else {
            None
        }
    }
}
```

### Game Detection Strategy

```rust
pub fn detect_game_installation(game_id: &str) -> Option<PathBuf> {
    // Platform-specific detection
    #[cfg(target_os = "windows")]
    {
        // 1. Check Windows Registry (most reliable)
        if let Some(path) = detect_from_registry(game_id) {
            return Some(path);
        }
    }

    // 2. Check Steam library paths (cross-platform)
    if let Some(path) = detect_from_steam(game_id) {
        return Some(path);
    }

    // 3. Check GOG Galaxy paths
    if let Some(path) = detect_from_gog(game_id) {
        return Some(path);
    }

    // 4. Check common installation directories
    detect_from_common_paths(game_id)
}
```

## [Testing Strategy](./testing_strategy.md)

### Unit Tests

Each crate has comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_ownership_tracking() {
        let mut log = InstallLog::new_in_memory().unwrap();

        // Install mod A
        let mod_a = create_test_mod("ModA");
        let key_a = log.add_active_mod(&mod_a).unwrap();
        log.add_data_file(&key_a, Path::new("data/texture.dds")).unwrap();

        // Install mod B (same file)
        let mod_b = create_test_mod("ModB");
        let key_b = log.add_active_mod(&mod_b).unwrap();
        log.add_data_file(&key_b, Path::new("data/texture.dds")).unwrap();

        // B is current owner, A is previous
        assert_eq!(log.get_current_file_owner(Path::new("data/texture.dds")).unwrap().name, "ModB");
        assert_eq!(log.get_previous_file_owner(Path::new("data/texture.dds")).unwrap().name, "ModA");

        // Remove B, A becomes owner again
        log.remove_data_file(&key_b, Path::new("data/texture.dds")).unwrap();
        assert_eq!(log.get_current_file_owner(Path::new("data/texture.dds")).unwrap().name, "ModA");
    }
}
```

### Integration Tests

```rust
// tests/integration/src/install_tests.rs

#[tokio::test]
async fn test_full_mod_installation_flow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let game_dir = temp_dir.path().join("game");
    let mods_dir = temp_dir.path().join("mods");
    let virtual_dir = temp_dir.path().join("virtual");

    // Set up mock game installation
    setup_mock_skyrim(&game_dir);

    // Initialize components
    let game_mode = SkyrimSEGameMode::new(&game_dir).unwrap();
    let mut vfs = VirtualModActivator::new(&virtual_dir, &game_mode).unwrap();
    let mut install_log = InstallLog::new(temp_dir.path().join("InstallLog.db")).unwrap();

    // Install a test mod
    let mod_path = create_test_fomod(&mods_dir, "TestMod", &["textures/test.dds"]);
    let test_mod = FomodFormat.create_mod(&mod_path, &game_mode).unwrap();

    // Run installation
    install_mod(&mut vfs, &mut install_log, &test_mod, &game_dir).await.unwrap();

    // Verify
    assert!(game_dir.join("Data/textures/test.dds").exists());
    assert!(install_log.get_current_file_owner(Path::new("Data/textures/test.dds")).is_some());
}
```

### Fixture Mods

Create test fixtures for common mod structures:

```
tests/fixtures/
├── simple_mod/
│   └── Data/
│       └── textures/
│           └── test.dds
├── fomod_with_script/
│   ├── fomod/
│   │   ├── info.xml
│   │   └── ModuleConfig.xml
│   └── Data/
│       ├── option_a/
│       └── option_b/
└── legacy_fomod/
    └── textures/
        └── test.dds  # Missing Data folder (legacy format)
```

## [Performance Considerations](./performance_considerations.md)

### File Operations

- Use memory-mapped files for large archives
- Batch symlink operations
- Async file I/O with tokio

### Database

- Use SQLite WAL mode for concurrent reads
- Prepare statements for repeated queries
- Batch inserts with transactions

### UI

- Virtual scrolling for large mod lists
- Lazy loading of mod metadata
- Background indexing

## [Security Considerations](./security_considerations.md)

### Script Sandboxing

```rust
// WASM scripts run in a sandboxed environment
pub struct ScriptSandbox {
    engine: wasmtime::Engine,
    linker: wasmtime::Linker<ScriptState>,
}

impl ScriptSandbox {
    pub fn new() -> Self {
        let mut config = wasmtime::Config::new();
        // Limit memory and CPU
        config.max_wasm_stack(1024 * 1024);  // 1MB stack
        config.memory_guaranteed_dense_image_size(64 * 1024 * 1024);  // 64MB memory

        let engine = wasmtime::Engine::new(&config).unwrap();
        let mut linker = wasmtime::Linker::new(&engine);

        // Only expose safe APIs
        linker.func_wrap("env", "read_file", |path: &str| -> Vec<u8> {
            // Validate path is within mod archive
            // ...
        });

        Self { engine, linker }
    }
}
```

### Path Validation

```rust
pub fn validate_path(base: &Path, relative: &Path) -> Result<PathBuf, SecurityError> {
    let resolved = base.join(relative).canonicalize()?;

    // Ensure path doesn't escape base directory
    if !resolved.starts_with(base) {
        return Err(SecurityError::PathTraversal);
    }

    // Check for dangerous file names
    let file_name = resolved.file_name().unwrap_or_default();
    if DANGEROUS_FILES.contains(&file_name.to_string_lossy().as_ref()) {
        return Err(SecurityError::DangerousFile);
    }

    Ok(resolved)
}
```

## [Future Enhancements](./future_enhancements.md)

### Phase 6+ (Future)

- **Cloud Sync**: Profile and mod list sync via Nexus account
- **Mod Collections**: Support for Nexus Collections format
- **Linux Support**: Full Linux support with Steam Proton integration
- **External Tool Integration**: Launch through SKSE/F4SE/SFSE
- **Conflict Visualization**: Tree-view of file conflicts
- **Automated Testing**: Test mods in isolated game instances
