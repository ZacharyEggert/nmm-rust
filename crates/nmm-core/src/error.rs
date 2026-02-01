//! Core error types for NMM.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when working with mods.
#[derive(Debug, Error)]
pub enum ModError {
    /// A file was not found in the mod archive.
    #[error("File not found in mod: {0}")]
    FileNotFound(String),

    /// Failed to read or parse the mod archive.
    #[error("Failed to read archive: {0}")]
    ArchiveError(String),

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur when working with mod formats.
#[derive(Debug, Error)]
pub enum ModFormatError {
    /// The file format is not supported.
    #[error("Unsupported format")]
    UnsupportedFormat,

    /// The archive is corrupted or invalid.
    #[error("Corrupt archive: {0}")]
    CorruptArchive(String),

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur when working with game plugins.
#[derive(Debug, Error)]
pub enum PluginError {
    /// The plugin file is invalid or cannot be parsed.
    #[error("Invalid plugin: {0}")]
    Invalid(String),

    /// The plugin file was not found.
    #[error("Plugin not found: {0}")]
    NotFound(PathBuf),

    /// A required master plugin is missing.
    #[error("Missing master: {0}")]
    MissingMaster(String),

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur when working with game modes.
#[derive(Debug, Error)]
pub enum GameModeError {
    /// The game installation was not found.
    #[error("Game not found at path: {0}")]
    GameNotFound(PathBuf),

    /// The game version is not supported.
    #[error("Unsupported game version: {0}")]
    UnsupportedVersion(String),

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur when working with the install log.
#[derive(Debug, Error)]
pub enum InstallLogError {
    /// The mod identified by the given key is not registered.
    #[error("Mod not found: {0}")]
    ModNotFound(String),

    /// A mod with this key is already registered.
    #[error("Mod already registered: {0}")]
    AlreadyRegistered(String),

    /// The requested ownership entry does not exist.
    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    /// No transaction is currently active.
    #[error("No active transaction")]
    NoActiveTransaction,

    /// A transaction is already active.
    #[error("Transaction already active")]
    TransactionAlreadyActive,

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_error_display() {
        let e = PluginError::Invalid("bad header".into());
        assert_eq!(e.to_string(), "Invalid plugin: bad header");

        let e = PluginError::NotFound(PathBuf::from("Data/mods/missing.esp"));
        assert_eq!(e.to_string(), "Plugin not found: Data/mods/missing.esp");

        let e = PluginError::MissingMaster("Skyrim.esm".into());
        assert_eq!(e.to_string(), "Missing master: Skyrim.esm");
    }

    #[test]
    fn test_plugin_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let plugin_err = PluginError::from(io_err);
        assert!(plugin_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_mod_error_display() {
        let e = ModError::FileNotFound("textures/test.dds".into());
        assert_eq!(e.to_string(), "File not found in mod: textures/test.dds");

        let e = ModError::ArchiveError("truncated".into());
        assert_eq!(e.to_string(), "Failed to read archive: truncated");
    }

    #[test]
    fn test_mod_format_error_display() {
        let e = ModFormatError::UnsupportedFormat;
        assert_eq!(e.to_string(), "Unsupported format");

        let e = ModFormatError::CorruptArchive("bad magic".into());
        assert_eq!(e.to_string(), "Corrupt archive: bad magic");
    }

    #[test]
    fn test_game_mode_error_display() {
        let e = GameModeError::GameNotFound(PathBuf::from("/opt/games/Skyrim"));
        assert_eq!(e.to_string(), "Game not found at path: /opt/games/Skyrim");

        let e = GameModeError::UnsupportedVersion("0.9".into());
        assert_eq!(e.to_string(), "Unsupported game version: 0.9");
    }

    #[test]
    fn test_install_log_error_display() {
        let e = InstallLogError::ModNotFound("mod_123".into());
        assert_eq!(e.to_string(), "Mod not found: mod_123");

        let e = InstallLogError::AlreadyRegistered("mod_456".into());
        assert_eq!(e.to_string(), "Mod already registered: mod_456");

        let e = InstallLogError::EntryNotFound("Data/test.dds".into());
        assert_eq!(e.to_string(), "Entry not found: Data/test.dds");

        let e = InstallLogError::NoActiveTransaction;
        assert_eq!(e.to_string(), "No active transaction");

        let e = InstallLogError::TransactionAlreadyActive;
        assert_eq!(e.to_string(), "Transaction already active");
    }

    #[test]
    fn test_install_log_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "database missing");
        let log_err = InstallLogError::from(io_err);
        assert!(log_err.to_string().contains("IO error"));
        assert!(log_err.to_string().contains("database missing"));
    }
}
