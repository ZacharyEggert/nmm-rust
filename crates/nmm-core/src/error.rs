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
