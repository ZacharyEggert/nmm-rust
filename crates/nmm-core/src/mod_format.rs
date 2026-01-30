//! Mod archive format handling.
//!
//! This module defines the [`ModFormat`] trait for handling different
//! mod archive formats like FOMod, OMod, and generic archives.

use crate::error::ModFormatError;
use crate::game_mode::GameMode;
use crate::mod_info::Mod;
use std::path::Path;

/// Confidence level for format detection.
///
/// When detecting what format a mod archive is in, different formats
/// can claim different levels of confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatConfidence {
    /// The file is incompatible with this format.
    Incompatible = 0,

    /// The file can be converted to this format.
    Convertible = 1,

    /// The file is compatible with this format.
    Compatible = 2,

    /// The file definitively matches this format.
    Match = 3,
}

impl FormatConfidence {
    /// Check if the file is at least compatible with the format.
    pub fn is_usable(&self) -> bool {
        *self >= FormatConfidence::Compatible
    }
}

/// Mod archive format handler.
///
/// Implementations of this trait know how to detect and work with
/// specific mod archive formats.
///
/// # Example
///
/// ```rust,ignore
/// use nmm_core::{ModFormat, FormatConfidence};
///
/// struct FomodFormat;
///
/// impl ModFormat for FomodFormat {
///     fn name(&self) -> &str { "FOMod" }
///     fn id(&self) -> &str { "FOMod" }
///     fn extension(&self) -> &str { ".fomod" }
///     fn supports_compression(&self) -> bool { false }
///
///     fn check_compliance(&self, path: &Path) -> FormatConfidence {
///         // Check for fomod folder or specific markers
///         if has_fomod_info(path) {
///             FormatConfidence::Match
///         } else {
///             FormatConfidence::Compatible
///         }
///     }
///
///     fn create_mod(
///         &self,
///         path: &Path,
///         game_mode: &dyn GameMode,
///     ) -> Result<Box<dyn Mod>, ModFormatError> {
///         // Create FOMod instance
///         Ok(Box::new(Fomod::new(path, game_mode)?))
///     }
/// }
/// ```
pub trait ModFormat: Send + Sync {
    /// Human-readable format name.
    fn name(&self) -> &str;

    /// Unique format identifier.
    ///
    /// Used for format matching and persistence.
    fn id(&self) -> &str;

    /// File extension typically used for this format.
    fn extension(&self) -> &str;

    /// Whether this format supports creating/compressing mods.
    fn supports_compression(&self) -> bool;

    /// Check how well a file matches this format.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the mod archive file
    ///
    /// # Returns
    ///
    /// A [`FormatConfidence`] indicating how well the file matches.
    fn check_compliance(&self, path: &Path) -> FormatConfidence;

    /// Create a [`Mod`] instance from an archive file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the mod archive file
    /// * `game_mode` - The game mode for context
    ///
    /// # Errors
    ///
    /// Returns [`ModFormatError`] if the file cannot be read or parsed.
    fn create_mod(
        &self,
        path: &Path,
        game_mode: &dyn GameMode,
    ) -> Result<Box<dyn Mod>, ModFormatError>;
}

/// Registry of mod formats.
///
/// Used to detect the format of mod archives and create appropriate
/// handlers.
pub struct ModFormatRegistry {
    formats: Vec<Box<dyn ModFormat>>,
}

impl Default for ModFormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModFormatRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
        }
    }

    /// Register a format handler.
    pub fn register(&mut self, format: Box<dyn ModFormat>) {
        self.formats.push(format);
    }

    /// Detect the best matching format for a file.
    ///
    /// Returns the format with the highest confidence level.
    pub fn detect_format(&self, path: &Path) -> Option<&dyn ModFormat> {
        self.formats
            .iter()
            .map(|f| (f.as_ref(), f.check_compliance(path)))
            .filter(|(_, c)| c.is_usable())
            .max_by_key(|(_, c)| *c)
            .map(|(f, _)| f)
    }

    /// Get a format by ID.
    pub fn get_format(&self, id: &str) -> Option<&dyn ModFormat> {
        self.formats.iter().find(|f| f.id() == id).map(|f| f.as_ref())
    }

    /// Get all registered formats.
    pub fn formats(&self) -> &[Box<dyn ModFormat>] {
        &self.formats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_confidence_ordering() {
        assert!(FormatConfidence::Match > FormatConfidence::Compatible);
        assert!(FormatConfidence::Compatible > FormatConfidence::Convertible);
        assert!(FormatConfidence::Convertible > FormatConfidence::Incompatible);
    }

    #[test]
    fn test_format_confidence_usable() {
        assert!(FormatConfidence::Match.is_usable());
        assert!(FormatConfidence::Compatible.is_usable());
        assert!(!FormatConfidence::Convertible.is_usable());
        assert!(!FormatConfidence::Incompatible.is_usable());
    }
}
