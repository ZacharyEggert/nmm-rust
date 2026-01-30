## Core Trait Definitions

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
