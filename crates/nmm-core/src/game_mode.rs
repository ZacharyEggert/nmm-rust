//! Game mode abstraction.
//!
//! This module defines the traits for game modes:
//!
//! - [`GameModeDescriptor`] - Static metadata about a game
//! - [`GameMode`] - Runtime game mode with installation path
//! - [`GameTheme`] - UI theming for the game

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// UI theme for a game mode.
#[derive(Debug, Clone, Default)]
pub struct GameTheme {
    /// Primary UI color (hex format, e.g., "#4a90d9").
    pub primary_color: String,

    /// Path to the game icon.
    pub icon_path: Option<PathBuf>,
}

/// Static metadata about a game mode.
///
/// This trait provides information that doesn't depend on a specific
/// game installation, such as the game's name and supported file types.
///
/// # Example
///
/// ```rust
/// use nmm_core::{GameModeDescriptor, GameTheme};
///
/// struct SkyrimSEDescriptor;
///
/// impl GameModeDescriptor for SkyrimSEDescriptor {
///     fn mode_id(&self) -> &str { "SkyrimSE" }
///     fn name(&self) -> &str { "Skyrim Special Edition" }
///     fn game_executables(&self) -> &[&str] { &["SkyrimSE.exe"] }
///     fn plugin_extensions(&self) -> &[&str] { &[".esp", ".esm", ".esl"] }
///     fn critical_plugins(&self) -> &[&str] {
///         &["Skyrim.esm", "Update.esm", "Dawnguard.esm", "HearthFires.esm", "Dragonborn.esm"]
///     }
///     fn official_plugins(&self) -> &[&str] {
///         &["Skyrim.esm", "Update.esm", "Dawnguard.esm", "HearthFires.esm", "Dragonborn.esm"]
///     }
///     fn stop_folders(&self) -> &[&str] {
///         &["Data", "Textures", "Meshes", "Sound", "Scripts", "Interface", "SKSE"]
///     }
///     fn theme(&self) -> GameTheme {
///         GameTheme { primary_color: "#2b5797".into(), icon_path: None }
///     }
/// }
/// ```
pub trait GameModeDescriptor: Send + Sync {
    /// Unique identifier for this game mode.
    ///
    /// This should be a short, filesystem-safe string like "SkyrimSE" or "Fallout4".
    fn mode_id(&self) -> &str;

    /// Human-readable display name.
    ///
    /// For example, "Skyrim Special Edition" or "Fallout 4".
    fn name(&self) -> &str;

    /// Possible executable names for the game.
    ///
    /// Used for game detection and version checking.
    fn game_executables(&self) -> &[&str];

    /// File extensions for game plugins.
    ///
    /// For Bethesda games, this would be `[".esp", ".esm", ".esl"]`.
    /// Return an empty slice for games without a plugin system.
    fn plugin_extensions(&self) -> &[&str];

    /// Critical plugins that cannot be disabled or reordered.
    ///
    /// These are typically the base game master files.
    fn critical_plugins(&self) -> &[&str];

    /// Official DLC/expansion plugins in their correct load order.
    fn official_plugins(&self) -> &[&str];

    /// Folders that indicate the root of mod content in archives.
    ///
    /// When extracting mods, NMM looks for these folders to determine
    /// where the actual mod content starts.
    fn stop_folders(&self) -> &[&str];

    /// UI theme for this game mode.
    fn theme(&self) -> GameTheme;

    /// Maximum number of active plugins (0 = unlimited).
    ///
    /// For Bethesda games, this is typically 255.
    fn max_active_plugins(&self) -> u32 {
        0
    }

    /// Required external tool name (e.g., "SKSE", "F4SE").
    fn required_tool_name(&self) -> Option<&str> {
        None
    }
}

/// Plugin factory trait for games that use plugins.
pub trait PluginFactory: Send + Sync {
    /// Create a plugin from a file path.
    fn create_plugin(&self, path: &Path) -> Result<Plugin, crate::error::ModError>;

    /// Check if a file is a valid plugin.
    fn is_plugin(&self, path: &Path) -> bool;
}

/// Plugin order validator.
pub trait PluginOrderValidator: Send + Sync {
    /// Validate a plugin order.
    fn validate(&self, plugins: &[Plugin]) -> bool;

    /// Correct an invalid order (modifies in place).
    fn correct_order(&self, plugins: &mut Vec<Plugin>);
}

/// Load order manager.
pub trait LoadOrderManager: Send + Sync {
    /// Get the current load order.
    fn get_load_order(&self) -> Result<Vec<Plugin>, crate::error::ModError>;

    /// Set the load order.
    fn set_load_order(&mut self, plugins: &[Plugin]) -> Result<(), crate::error::ModError>;

    /// Activate a plugin.
    fn activate(&mut self, plugin: &Plugin) -> Result<(), crate::error::ModError>;

    /// Deactivate a plugin.
    fn deactivate(&mut self, plugin: &Plugin) -> Result<(), crate::error::ModError>;

    /// Get currently active plugins.
    fn active_plugins(&self) -> Vec<&Plugin>;
}

/// Game plugin representation.
#[derive(Debug, Clone)]
pub struct Plugin {
    /// Full path to the plugin file.
    pub path: PathBuf,

    /// Plugin filename.
    pub filename: String,

    /// Whether this is a master file (.esm).
    pub is_master: bool,

    /// Whether this is a light plugin (.esl).
    pub is_light: bool,

    /// Required master plugins.
    pub masters: Vec<String>,

    /// Plugin description (from file header).
    pub description: Option<String>,

    /// Plugin author (from file header).
    pub author: Option<String>,
}

/// Runtime game mode.
///
/// Extends [`GameModeDescriptor`] with runtime information about a specific
/// game installation.
pub trait GameMode: GameModeDescriptor {
    /// Path where the game is installed.
    fn installation_path(&self) -> &Path;

    /// Secondary installation path (if applicable).
    ///
    /// Some games have multiple installation directories.
    fn secondary_installation_path(&self) -> Option<&Path> {
        None
    }

    /// Directory where game plugins are stored.
    ///
    /// For Bethesda games, this is typically `<game>/Data`.
    fn plugin_directory(&self) -> PathBuf;

    /// Whether this game uses a plugin system.
    fn uses_plugins(&self) -> bool;

    /// Whether plugin auto-sorting is supported.
    fn supports_plugin_auto_sorting(&self) -> bool {
        false
    }

    /// Get the plugin factory for this game.
    fn plugin_factory(&self) -> Option<Box<dyn PluginFactory>>;

    /// Get the plugin order validator.
    fn plugin_order_validator(&self) -> Option<Box<dyn PluginOrderValidator>>;

    /// Get the load order manager.
    fn load_order_manager(&self) -> Option<Box<dyn LoadOrderManager>>;

    /// Check if a plugin is critical (cannot be disabled/reordered).
    fn is_critical_plugin(&self, plugin_name: &str) -> bool {
        self.critical_plugins()
            .iter()
            .any(|p| p.eq_ignore_ascii_case(plugin_name))
    }

    /// Adjust path for mod format compatibility.
    ///
    /// This handles legacy mods that assume different installation paths.
    /// For example, older FOMods for Bethesda games might not include
    /// the "Data" prefix.
    fn adjust_mod_path(&self, _format_id: &str, path: &str, _ignore_if_present: bool) -> String {
        path.to_string()
    }

    /// File extensions that require hardlinks instead of symlinks.
    ///
    /// Some file types (like Bethesda plugins) don't work correctly
    /// through symlinks.
    fn hardlink_required_extensions(&self) -> HashSet<&str> {
        HashSet::new()
    }

    /// Get paths to game settings/INI files.
    fn settings_files(&self) -> Vec<PathBuf> {
        vec![]
    }

    /// Get the installed game version.
    fn game_version(&self) -> Option<semver::Version> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockGameDescriptor;

    impl GameModeDescriptor for MockGameDescriptor {
        fn mode_id(&self) -> &str {
            "MockGame"
        }
        fn name(&self) -> &str {
            "Mock Game"
        }
        fn game_executables(&self) -> &[&str] {
            &["MockGame.exe"]
        }
        fn plugin_extensions(&self) -> &[&str] {
            &[]
        }
        fn critical_plugins(&self) -> &[&str] {
            &[]
        }
        fn official_plugins(&self) -> &[&str] {
            &[]
        }
        fn stop_folders(&self) -> &[&str] {
            &["Data"]
        }
        fn theme(&self) -> GameTheme {
            GameTheme::default()
        }
    }

    #[test]
    fn test_descriptor_defaults() {
        let desc = MockGameDescriptor;
        assert_eq!(desc.max_active_plugins(), 0);
        assert!(desc.required_tool_name().is_none());
    }
}
