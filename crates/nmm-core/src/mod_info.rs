//! Mod metadata and archive abstraction.
//!
//! This module defines the core types for representing mods:
//!
//! - [`ModInfo`] - Metadata about a mod (name, version, author, etc.)
//! - [`Mod`] - Trait for accessing mod archive contents
//! - [`ScriptType`] - Types of installation scripts

use crate::error::ModError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Mod metadata.
///
/// This struct contains all the information about a mod, including
/// identifiers, version information, and metadata from Nexus Mods.
///
/// # Example
///
/// ```rust
/// use nmm_core::ModInfo;
///
/// let mod_info = ModInfo {
///     name: "My Cool Mod".into(),
///     file_name: "MyCoolMod.7z".into(),
///     version: "1.0.0".into(),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModInfo {
    /// Nexus Mods mod ID.
    pub id: Option<String>,

    /// Nexus Mods download ID.
    pub download_id: Option<String>,

    /// Display name of the mod.
    pub name: String,

    /// Archive filename.
    pub file_name: String,

    /// Human-readable version string.
    pub version: String,

    /// Parsed semantic version (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_version: Option<semver::Version>,

    /// Mod author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Mod description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Nexus Mods category ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i32>,

    /// User-assigned custom category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_category_id: Option<i32>,

    /// Mod website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<url::Url>,

    /// When the mod was downloaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_date: Option<DateTime<Utc>>,

    /// When the mod was installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_date: Option<DateTime<Utc>>,

    /// Whether the user has endorsed this mod.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_endorsed: Option<bool>,

    /// Position in mod load order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_order: Option<i32>,
}

impl ModInfo {
    /// Create a new ModInfo with required fields.
    pub fn new(name: impl Into<String>, file_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            file_name: file_name.into(),
            ..Default::default()
        }
    }

    /// Set the version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
}

/// Type of installation script in a mod.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    /// XML-based configuration script (fomod/ModuleConfig.xml).
    XmlScript,

    /// Legacy ModScript format (for Morrowind/Oblivion).
    ModScript,

    /// WebAssembly-based script.
    Wasm,
}

/// Trait for accessing mod archive contents.
///
/// This trait abstracts over different mod archive formats, allowing
/// uniform access to files within the mod.
///
/// # Example
///
/// ```rust,ignore
/// use nmm_core::Mod;
///
/// fn list_textures(mod_archive: &dyn Mod) -> Vec<String> {
///     mod_archive
///         .file_list()
///         .unwrap()
///         .into_iter()
///         .filter(|f| f.ends_with(".dds"))
///         .collect()
/// }
/// ```
pub trait Mod: Send + Sync {
    /// Get mod metadata.
    fn info(&self) -> &ModInfo;

    /// Path to the mod archive file.
    fn archive_path(&self) -> &Path;

    /// Format identifier (e.g., "FOMod", "OMod").
    fn format_id(&self) -> &str;

    /// List all files in the mod archive.
    fn file_list(&self) -> Result<Vec<String>, ModError>;

    /// List files in a specific folder within the archive.
    ///
    /// # Arguments
    ///
    /// * `folder` - Path to the folder within the archive
    /// * `recursive` - Whether to include files in subdirectories
    fn file_list_in_folder(&self, folder: &str, recursive: bool) -> Result<Vec<String>, ModError>;

    /// Read a file from the mod archive into memory.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file within the archive
    ///
    /// # Errors
    ///
    /// Returns `ModError::FileNotFound` if the file doesn't exist in the archive.
    fn read_file(&self, path: &str) -> Result<Vec<u8>, ModError>;

    /// Get a readable stream for a file in the archive.
    ///
    /// This is more memory-efficient than `read_file` for large files.
    fn read_file_stream(&self, path: &str) -> Result<Box<dyn std::io::Read + '_>, ModError>;

    /// Check if the mod has an installation script.
    fn has_script(&self) -> bool;

    /// Get the script content and type if present.
    fn script_content(&self) -> Option<(ScriptType, String)>;

    /// Path to the screenshot within the archive (if any).
    fn screenshot_path(&self) -> Option<&str>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_info_builder() {
        let info = ModInfo::new("Test Mod", "TestMod.7z")
            .with_version("1.0.0")
            .with_author("Test Author");

        assert_eq!(info.name, "Test Mod");
        assert_eq!(info.file_name, "TestMod.7z");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.author, Some("Test Author".into()));
    }

    #[test]
    fn test_mod_info_serialization() {
        let info = ModInfo::new("Test Mod", "TestMod.7z").with_version("1.0.0");

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ModInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, info.name);
        assert_eq!(parsed.version, info.version);
    }
}
