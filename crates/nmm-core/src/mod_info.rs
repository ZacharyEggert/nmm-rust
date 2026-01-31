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

    /// Last known version from Nexus Mods (for update checking).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_known_version: Option<String>,

    /// Screenshot/thumbnail image data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<Vec<u8>>,

    /// Whether to warn the user about new versions.
    #[serde(default)]
    pub update_warning_enabled: bool,

    /// Whether to automatically check for updates.
    #[serde(default)]
    pub update_checks_enabled: bool,

    /// Staging area for new load order position during reordering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_load_order: Option<i32>,
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

    /// Set the download ID.
    pub fn with_download_id(mut self, download_id: impl Into<String>) -> Self {
        self.download_id = Some(download_id.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the website URL.
    pub fn with_website(mut self, url: url::Url) -> Self {
        self.website = Some(url);
        self
    }

    /// Set the screenshot data.
    pub fn with_screenshot(mut self, data: Vec<u8>) -> Self {
        self.screenshot = Some(data);
        self
    }

    /// Enable or disable update warnings.
    pub fn with_update_warnings(mut self, enabled: bool) -> Self {
        self.update_warning_enabled = enabled;
        self
    }

    /// Enable or disable automatic update checks.
    pub fn with_update_checks(mut self, enabled: bool) -> Self {
        self.update_checks_enabled = enabled;
        self
    }

    /// Parse the human-readable version string into a semantic version.
    ///
    /// This method attempts to extract a valid semantic version from the
    /// version string, handling common formatting issues like missing components,
    /// extra characters, and leading/trailing dots.
    pub fn parse_machine_version(&mut self) {
        self.machine_version = Self::parse_version(&self.version);
    }

    /// Parse a version string into a semantic version.
    ///
    /// # Arguments
    ///
    /// * `version_str` - The version string to parse
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmm_core::ModInfo;
    ///
    /// assert_eq!(ModInfo::parse_version("1.2.3").unwrap().to_string(), "1.2.3");
    /// assert_eq!(ModInfo::parse_version("v1.2").unwrap().to_string(), "1.2.0");
    /// assert_eq!(ModInfo::parse_version("5").unwrap().to_string(), "5.0.0");
    /// assert_eq!(ModInfo::parse_version(".5").unwrap().to_string(), "0.5.0");
    /// assert!(ModInfo::parse_version("invalid").is_none());
    /// ```
    pub fn parse_version(version_str: &str) -> Option<semver::Version> {
        // Remove non-numeric and non-period characters
        let cleaned: String = version_str
            .chars()
            .filter(|c| c.is_numeric() || *c == '.')
            .collect();

        if cleaned.is_empty() {
            return None;
        }

        // Normalize version string
        let cleaned = cleaned.trim_end_matches('.');
        let cleaned = if cleaned.starts_with('.') {
            format!("0{}", cleaned)
        } else {
            cleaned.to_string()
        };

        // Split by dots and filter out empty parts (handles consecutive dots)
        let parts: Vec<&str> = cleaned.split('.').filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            return None;
        }

        // Ensure at least major.minor.patch format
        let normalized = match parts.len() {
            1 => format!("{}.0.0", parts[0]),
            2 => format!("{}.{}.0", parts[0], parts[1]),
            _ => parts.join("."),
        };

        semver::Version::parse(&normalized).ok()
    }

    /// Check if there's a newer version available.
    ///
    /// Returns `true` if `last_known_version` is set and is greater than
    /// the current `machine_version`.
    pub fn has_update(&self) -> bool {
        if let (Some(current), Some(latest_str)) = (&self.machine_version, &self.last_known_version)
        {
            if let Some(latest) = Self::parse_version(latest_str) {
                return latest > *current;
            }
        }
        false
    }

    /// Check if the user should be notified about an update.
    ///
    /// Returns `true` if update warnings are enabled and a newer version is available.
    pub fn should_notify_update(&self) -> bool {
        self.update_warning_enabled && self.has_update()
    }

    /// Update this ModInfo with values from another ModInfo.
    ///
    /// # Arguments
    ///
    /// * `other` - The ModInfo to copy values from
    /// * `overwrite_all` - If `true`, overwrites all fields. If `false`, only fills in empty/None fields.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nmm_core::ModInfo;
    ///
    /// let mut original = ModInfo::new("A", "a.7z").with_version("1.0");
    /// let update = ModInfo::new("B", "b.7z").with_description("Updated description");
    ///
    /// // Keep existing values, only fill in empty fields
    /// original.update_from(&update, false);
    /// assert_eq!(original.name, "A"); // Kept
    /// assert_eq!(original.description, Some("Updated description".into())); // Filled
    /// ```
    pub fn update_from(&mut self, other: &ModInfo, overwrite_all: bool) {
        macro_rules! update_option {
            ($field:ident) => {
                if overwrite_all || self.$field.is_none() {
                    self.$field = other.$field.clone();
                }
            };
        }

        macro_rules! update_string {
            ($field:ident) => {
                if overwrite_all || self.$field.is_empty() {
                    self.$field = other.$field.clone();
                }
            };
        }

        macro_rules! update_bool {
            ($field:ident) => {
                if overwrite_all {
                    self.$field = other.$field;
                }
            };
        }

        update_option!(id);
        update_option!(download_id);
        update_string!(name);
        update_string!(file_name);
        update_string!(version);
        update_option!(machine_version);
        update_option!(last_known_version);
        update_option!(author);
        update_option!(description);
        update_option!(category_id);
        update_option!(custom_category_id);
        update_option!(website);
        update_option!(download_date);
        update_option!(install_date);
        update_option!(is_endorsed);
        update_option!(screenshot);
        update_bool!(update_warning_enabled);
        update_bool!(update_checks_enabled);
        update_option!(load_order);
        update_option!(new_load_order);
    }
}

impl std::fmt::Display for ModInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;

        if !self.version.is_empty() {
            write!(f, " v{}", self.version)?;
        }

        if let Some(author) = &self.author {
            write!(f, " by {}", author)?;
        }

        Ok(())
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

    #[test]
    fn test_new_fields_defaults() {
        let info = ModInfo::default();
        assert!(info.last_known_version.is_none());
        assert!(info.screenshot.is_none());
        assert!(!info.update_warning_enabled);
        assert!(!info.update_checks_enabled);
        assert!(info.new_load_order.is_none());
    }

    #[test]
    fn test_builder_pattern_new_methods() {
        let info = ModInfo::new("Test", "test.7z")
            .with_download_id("12345")
            .with_description("A test mod")
            .with_website(url::Url::parse("https://example.com").unwrap())
            .with_screenshot(vec![1, 2, 3, 4])
            .with_update_warnings(true)
            .with_update_checks(true);

        assert_eq!(info.download_id, Some("12345".into()));
        assert_eq!(info.description, Some("A test mod".into()));
        assert!(info.website.is_some());
        assert_eq!(info.screenshot, Some(vec![1, 2, 3, 4]));
        assert!(info.update_warning_enabled);
        assert!(info.update_checks_enabled);
    }

    #[test]
    fn test_version_parsing_valid() {
        assert_eq!(
            ModInfo::parse_version("1.2.3").unwrap().to_string(),
            "1.2.3"
        );
        assert_eq!(ModInfo::parse_version("1.2").unwrap().to_string(), "1.2.0");
        assert_eq!(ModInfo::parse_version("5").unwrap().to_string(), "5.0.0");
        assert_eq!(
            ModInfo::parse_version("2.10.5").unwrap().to_string(),
            "2.10.5"
        );
    }

    #[test]
    fn test_version_parsing_with_prefix() {
        assert_eq!(
            ModInfo::parse_version("v1.2.3").unwrap().to_string(),
            "1.2.3"
        );
        assert_eq!(ModInfo::parse_version("V1.5").unwrap().to_string(), "1.5.0");
        assert_eq!(
            ModInfo::parse_version("version 2.0.1").unwrap().to_string(),
            "2.0.1"
        );
    }

    #[test]
    fn test_version_parsing_edge_cases() {
        assert_eq!(ModInfo::parse_version(".5").unwrap().to_string(), "0.5.0");
        assert_eq!(ModInfo::parse_version("1.2.").unwrap().to_string(), "1.2.0");
        assert_eq!(ModInfo::parse_version("1..2").unwrap().to_string(), "1.2.0");
    }

    #[test]
    fn test_version_parsing_invalid() {
        assert!(ModInfo::parse_version("invalid").is_none());
        assert!(ModInfo::parse_version("").is_none());
        assert!(ModInfo::parse_version("abc").is_none());
        assert!(ModInfo::parse_version("...").is_none());
    }

    #[test]
    fn test_parse_machine_version() {
        let mut info = ModInfo::new("Test", "test.7z").with_version("v1.5.2");
        info.parse_machine_version();
        assert_eq!(info.machine_version.unwrap().to_string(), "1.5.2");
    }

    #[test]
    fn test_has_update_true() {
        let mut info = ModInfo::new("Test", "test.7z").with_version("1.0.0");
        info.parse_machine_version();
        info.last_known_version = Some("1.5.0".into());
        assert!(info.has_update());
    }

    #[test]
    fn test_has_update_false_same_version() {
        let mut info = ModInfo::new("Test", "test.7z").with_version("1.5.0");
        info.parse_machine_version();
        info.last_known_version = Some("1.5.0".into());
        assert!(!info.has_update());
    }

    #[test]
    fn test_has_update_false_older_version() {
        let mut info = ModInfo::new("Test", "test.7z").with_version("2.0.0");
        info.parse_machine_version();
        info.last_known_version = Some("1.5.0".into());
        assert!(!info.has_update());
    }

    #[test]
    fn test_has_update_no_machine_version() {
        let mut info = ModInfo::new("Test", "test.7z").with_version("invalid");
        info.last_known_version = Some("1.5.0".into());
        assert!(!info.has_update());
    }

    #[test]
    fn test_should_notify_update() {
        let mut info = ModInfo::new("Test", "test.7z")
            .with_version("1.0.0")
            .with_update_warnings(true);
        info.parse_machine_version();
        info.last_known_version = Some("1.5.0".into());
        assert!(info.should_notify_update());
    }

    #[test]
    fn test_should_notify_update_disabled() {
        let mut info = ModInfo::new("Test", "test.7z")
            .with_version("1.0.0")
            .with_update_warnings(false);
        info.parse_machine_version();
        info.last_known_version = Some("1.5.0".into());
        assert!(!info.should_notify_update());
    }

    #[test]
    fn test_update_from_overwrite_all() {
        let mut original = ModInfo::new("A", "a.7z").with_version("1.0");
        let update = ModInfo::new("B", "b.7z").with_version("2.0");
        original.update_from(&update, true);
        assert_eq!(original.name, "B");
        assert_eq!(original.file_name, "b.7z");
        assert_eq!(original.version, "2.0");
    }

    #[test]
    fn test_update_from_keep_existing() {
        let mut original = ModInfo::new("A", "a.7z").with_version("1.0");
        let update = ModInfo::new("B", "b.7z").with_description("Updated description");
        original.update_from(&update, false);
        assert_eq!(original.name, "A"); // Kept
        assert_eq!(original.version, "1.0"); // Kept
        assert_eq!(original.description, Some("Updated description".into())); // Filled
    }

    #[test]
    fn test_update_from_fill_empty_fields() {
        let mut original = ModInfo::new("Test", "test.7z");
        let update = ModInfo::new("Test", "test.7z")
            .with_author("Author")
            .with_description("Description")
            .with_download_id("123");
        original.update_from(&update, false);
        assert_eq!(original.author, Some("Author".into()));
        assert_eq!(original.description, Some("Description".into()));
        assert_eq!(original.download_id, Some("123".into()));
    }

    #[test]
    fn test_update_from_bool_fields() {
        let mut original = ModInfo::new("A", "a.7z")
            .with_update_warnings(false)
            .with_update_checks(false);
        let update = ModInfo::new("B", "b.7z")
            .with_update_warnings(true)
            .with_update_checks(true);

        // Bool fields should not update when overwrite_all is false
        original.update_from(&update, false);
        assert!(!original.update_warning_enabled);
        assert!(!original.update_checks_enabled);

        // Bool fields should update when overwrite_all is true
        original.update_from(&update, true);
        assert!(original.update_warning_enabled);
        assert!(original.update_checks_enabled);
    }

    #[test]
    fn test_display_trait() {
        let info = ModInfo::new("Test Mod", "test.7z");
        assert_eq!(format!("{}", info), "Test Mod");

        let info_with_version = info.with_version("1.2.3");
        assert_eq!(format!("{}", info_with_version), "Test Mod v1.2.3");

        let info_with_author = info_with_version.with_author("John Doe");
        assert_eq!(
            format!("{}", info_with_author),
            "Test Mod v1.2.3 by John Doe"
        );
    }

    #[test]
    fn test_screenshot_serialization() {
        let screenshot_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // Fake JPEG header
        let info = ModInfo::new("Test", "test.7z").with_screenshot(screenshot_data.clone());

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ModInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.screenshot, Some(screenshot_data));
    }

    #[test]
    fn test_serialization_backward_compatibility() {
        // Test that old JSON without new fields can still be deserialized
        let old_json = r#"{
            "name": "Old Mod",
            "file_name": "old.7z",
            "version": "1.0.0"
        }"#;

        let info: ModInfo = serde_json::from_str(old_json).unwrap();
        assert_eq!(info.name, "Old Mod");
        assert_eq!(info.file_name, "old.7z");
        assert_eq!(info.version, "1.0.0");
        assert!(info.last_known_version.is_none());
        assert!(!info.update_warning_enabled);
        assert!(!info.update_checks_enabled);
    }

    #[test]
    fn test_load_order_independence() {
        let mut info1 = ModInfo::new("Test", "test.7z");
        info1.load_order = Some(5);
        info1.new_load_order = Some(10);

        let mut info2 = ModInfo::new("Test", "test.7z");
        info2.load_order = Some(3);
        info2.new_load_order = Some(7);

        // Load order fields are independent
        assert_ne!(info1.load_order, info2.load_order);
        assert_ne!(info1.new_load_order, info2.new_load_order);
    }
}
