//! Installation log tracking for mods, files, INI edits, and game-specific values.

use crate::{InstallLogError, ModInfo};
use std::hash::{Hash, Hasher};

/// Constant representing the special mod key used to store original file values
/// before any mods modified them.
pub const ORIGINAL_VALUES_KEY: &str = "<<ORIGINAL_VALUES>>";

/// Represents a single INI file edit coordinate (file, section, key).
///
/// Equality and hashing are case-insensitive to match INI file semantics
/// and the `COLLATE NOCASE` behavior in the database schema.
#[derive(Debug, Clone)]
pub struct IniEdit {
    pub file: String,
    pub section: String,
    pub key: String,
}

impl IniEdit {
    /// Creates a new `IniEdit`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nmm_core::IniEdit;
    ///
    /// let edit = IniEdit::new("Skyrim.ini", "Display", "bFull Screen");
    /// ```
    pub fn new(
        file: impl Into<String>,
        section: impl Into<String>,
        key: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            section: section.into(),
            key: key.into(),
        }
    }
}

impl PartialEq for IniEdit {
    fn eq(&self, other: &Self) -> bool {
        self.file.eq_ignore_ascii_case(&other.file)
            && self.section.eq_ignore_ascii_case(&other.section)
            && self.key.eq_ignore_ascii_case(&other.key)
    }
}

impl Eq for IniEdit {}

impl Hash for IniEdit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file.to_ascii_lowercase().hash(state);
        self.section.to_ascii_lowercase().hash(state);
        self.key.to_ascii_lowercase().hash(state);
    }
}

impl PartialOrd for IniEdit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IniEdit {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let file_cmp = self
            .file
            .to_ascii_lowercase()
            .cmp(&other.file.to_ascii_lowercase());
        if file_cmp != std::cmp::Ordering::Equal {
            return file_cmp;
        }
        let section_cmp = self
            .section
            .to_ascii_lowercase()
            .cmp(&other.section.to_ascii_lowercase());
        if section_cmp != std::cmp::Ordering::Equal {
            return section_cmp;
        }
        self.key
            .to_ascii_lowercase()
            .cmp(&other.key.to_ascii_lowercase())
    }
}

impl std::fmt::Display for IniEdit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}].{}", self.file, self.section, self.key)
    }
}

/// Tracks all mod installations, file ownership, INI edits, and game-specific values.
///
/// The install log maintains a stack-based ownership model where multiple mods can
/// modify the same file or setting, with the most recently installed mod taking precedence.
/// When a mod is uninstalled, ownership reverts to the previous mod in the stack.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow use across thread boundaries.
///
/// # Transactions
///
/// The install log supports transactions for atomic multi-step operations. Begin a
/// transaction with [`begin_transaction`](InstallLog::begin_transaction), make changes,
/// then either [`commit_transaction`](InstallLog::commit_transaction) to keep them or
/// [`rollback_transaction`](InstallLog::rollback_transaction) to discard them.
pub trait InstallLog: Send + Sync {
    // -------------------------------------------------------------------------
    // Mod tracking (5 methods)
    // -------------------------------------------------------------------------

    /// Registers a new mod in the install log.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Unique identifier for the mod
    /// * `info` - Mod metadata
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::AlreadyRegistered`] if a mod with this key already exists
    /// * [`InstallLogError::Io`] if database access fails
    fn add_mod(&mut self, mod_key: &str, info: &ModInfo) -> Result<(), InstallLogError>;

    /// Updates an existing mod's metadata.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod to update
    /// * `info` - New mod metadata
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod doesn't exist
    /// * [`InstallLogError::Io`] if database access fails
    fn replace_mod(&mut self, mod_key: &str, info: &ModInfo) -> Result<(), InstallLogError>;

    /// Removes a mod and all its ownership records from the install log.
    ///
    /// This does not delete the mod's files from disk; it only removes tracking information.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod to remove
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod doesn't exist
    /// * [`InstallLogError::Io`] if database access fails
    fn remove_mod(&mut self, mod_key: &str) -> Result<(), InstallLogError>;

    /// Retrieves metadata for a specific mod.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod to look up
    ///
    /// # Returns
    ///
    /// `Some(ModInfo)` if found, `None` otherwise.
    fn get_mod(&self, mod_key: &str) -> Option<ModInfo>;

    /// Returns all registered mods.
    ///
    /// # Returns
    ///
    /// A vector of all mod metadata in the install log.
    fn active_mods(&self) -> Vec<ModInfo>;

    // -------------------------------------------------------------------------
    // File ownership (7 methods)
    // -------------------------------------------------------------------------

    /// Records that a mod installed a file.
    ///
    /// If another mod already owns this file, the new mod takes precedence (top of stack).
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod that installed the file
    /// * `file_path` - Path to the file (relative to game data directory)
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::Io`] if database access fails
    fn add_data_file(&mut self, mod_key: &str, file_path: &str) -> Result<(), InstallLogError>;

    /// Removes a file ownership record for a mod.
    ///
    /// If other mods also installed this file, ownership reverts to the previous mod.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    /// * `file_path` - Path to the file
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::EntryNotFound`] if the mod didn't install this file
    /// * [`InstallLogError::Io`] if database access fails
    fn remove_data_file(&mut self, mod_key: &str, file_path: &str) -> Result<(), InstallLogError>;

    /// Returns the mod key of the current owner of a file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file
    ///
    /// # Returns
    ///
    /// `Some(mod_key)` if a mod owns this file, `None` if no mod has installed it.
    fn get_current_file_owner(&self, file_path: &str) -> Option<String>;

    /// Returns the mod key of the previous owner of a file (second in the stack).
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file
    ///
    /// # Returns
    ///
    /// `Some(mod_key)` if at least two mods have installed this file, `None` otherwise.
    fn get_previous_file_owner(&self, file_path: &str) -> Option<String>;

    /// Records that a file existed in the game directory before any mods modified it.
    ///
    /// Uses the special [`ORIGINAL_VALUES_KEY`] as the mod key.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::Io`] if database access fails
    fn log_original_data_file(&mut self, file_path: &str) -> Result<(), InstallLogError>;

    /// Returns all files installed by a specific mod.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    ///
    /// # Returns
    ///
    /// A vector of file paths, or an error if the mod doesn't exist.
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::Io`] if database access fails
    fn get_installed_mod_files(&self, mod_key: &str) -> Result<Vec<String>, InstallLogError>;

    /// Returns all mod keys that have installed a specific file, in installation order.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file
    ///
    /// # Returns
    ///
    /// A vector of mod keys, ordered from oldest to newest installer. Returns an empty
    /// vector if no mods have installed this file.
    fn get_file_installers(&self, file_path: &str) -> Vec<String>;

    // -------------------------------------------------------------------------
    // INI edits (8 methods)
    // -------------------------------------------------------------------------

    /// Records that a mod modified an INI setting.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    /// * `edit` - The INI coordinate (file, section, key)
    /// * `value` - The new value set by the mod
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::Io`] if database access fails
    fn add_ini_edit(
        &mut self,
        mod_key: &str,
        edit: &IniEdit,
        value: &str,
    ) -> Result<(), InstallLogError>;

    /// Updates an existing INI edit's value.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    /// * `edit` - The INI coordinate
    /// * `value` - The new value
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::EntryNotFound`] if the mod didn't make this edit
    /// * [`InstallLogError::Io`] if database access fails
    fn replace_ini_edit(
        &mut self,
        mod_key: &str,
        edit: &IniEdit,
        value: &str,
    ) -> Result<(), InstallLogError>;

    /// Removes an INI edit ownership record.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    /// * `edit` - The INI coordinate
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::EntryNotFound`] if the mod didn't make this edit
    /// * [`InstallLogError::Io`] if database access fails
    fn remove_ini_edit(&mut self, mod_key: &str, edit: &IniEdit) -> Result<(), InstallLogError>;

    /// Returns the mod key of the current owner of an INI setting.
    ///
    /// # Arguments
    ///
    /// * `edit` - The INI coordinate
    ///
    /// # Returns
    ///
    /// `Some(mod_key)` if a mod owns this setting, `None` otherwise.
    fn get_current_ini_edit_owner(&self, edit: &IniEdit) -> Option<String>;

    /// Returns the previous value of an INI setting (second in the stack).
    ///
    /// # Arguments
    ///
    /// * `edit` - The INI coordinate
    ///
    /// # Returns
    ///
    /// `Some(value)` if at least two mods have modified this setting, `None` otherwise.
    fn get_previous_ini_value(&self, edit: &IniEdit) -> Option<String>;

    /// Records the original value of an INI setting before any mods modified it.
    ///
    /// Uses the special [`ORIGINAL_VALUES_KEY`] as the mod key.
    ///
    /// # Arguments
    ///
    /// * `edit` - The INI coordinate
    /// * `value` - The original value
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::Io`] if database access fails
    fn log_original_ini_value(
        &mut self,
        edit: &IniEdit,
        value: &str,
    ) -> Result<(), InstallLogError>;

    /// Returns all INI edits made by a specific mod.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    ///
    /// # Returns
    ///
    /// A vector of INI coordinates, or an error if the mod doesn't exist.
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::Io`] if database access fails
    fn get_installed_ini_edits(&self, mod_key: &str) -> Result<Vec<IniEdit>, InstallLogError>;

    /// Returns all mod keys that have modified an INI setting, in modification order.
    ///
    /// # Arguments
    ///
    /// * `edit` - The INI coordinate
    ///
    /// # Returns
    ///
    /// A vector of mod keys, ordered from oldest to newest. Returns an empty vector
    /// if no mods have modified this setting.
    fn get_ini_edit_installers(&self, edit: &IniEdit) -> Vec<String>;

    // -------------------------------------------------------------------------
    // Game-specific values (8 methods)
    // -------------------------------------------------------------------------

    /// Records that a mod modified a game-specific value (e.g., registry key, binary file).
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    /// * `gsv_key` - Key identifying the game-specific value
    /// * `value` - The binary value data
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::Io`] if database access fails
    fn add_gsv_edit(
        &mut self,
        mod_key: &str,
        gsv_key: &str,
        value: &[u8],
    ) -> Result<(), InstallLogError>;

    /// Updates an existing game-specific value.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    /// * `gsv_key` - Key identifying the value
    /// * `value` - The new binary value data
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::EntryNotFound`] if the mod didn't make this edit
    /// * [`InstallLogError::Io`] if database access fails
    fn replace_gsv_edit(
        &mut self,
        mod_key: &str,
        gsv_key: &str,
        value: &[u8],
    ) -> Result<(), InstallLogError>;

    /// Removes a game-specific value ownership record.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    /// * `gsv_key` - Key identifying the value
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::EntryNotFound`] if the mod didn't make this edit
    /// * [`InstallLogError::Io`] if database access fails
    fn remove_gsv_edit(&mut self, mod_key: &str, gsv_key: &str) -> Result<(), InstallLogError>;

    /// Returns the mod key of the current owner of a game-specific value.
    ///
    /// # Arguments
    ///
    /// * `gsv_key` - Key identifying the value
    ///
    /// # Returns
    ///
    /// `Some(mod_key)` if a mod owns this value, `None` otherwise.
    fn get_current_gsv_edit_owner(&self, gsv_key: &str) -> Option<String>;

    /// Returns the previous value of a game-specific setting (second in the stack).
    ///
    /// # Arguments
    ///
    /// * `gsv_key` - Key identifying the value
    ///
    /// # Returns
    ///
    /// `Some(value)` if at least two mods have modified this value, `None` otherwise.
    fn get_previous_gsv_value(&self, gsv_key: &str) -> Option<Vec<u8>>;

    /// Records the original value of a game-specific setting before any mods modified it.
    ///
    /// Uses the special [`ORIGINAL_VALUES_KEY`] as the mod key.
    ///
    /// # Arguments
    ///
    /// * `gsv_key` - Key identifying the value
    /// * `value` - The original binary value data
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::Io`] if database access fails
    fn log_original_gsv_value(
        &mut self,
        gsv_key: &str,
        value: &[u8],
    ) -> Result<(), InstallLogError>;

    /// Returns all game-specific value keys modified by a specific mod.
    ///
    /// # Arguments
    ///
    /// * `mod_key` - Identifier of the mod
    ///
    /// # Returns
    ///
    /// A vector of GSV keys, or an error if the mod doesn't exist.
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::ModNotFound`] if the mod isn't registered
    /// * [`InstallLogError::Io`] if database access fails
    fn get_installed_gsv_edits(&self, mod_key: &str) -> Result<Vec<String>, InstallLogError>;

    /// Returns all mod keys that have modified a game-specific value, in modification order.
    ///
    /// # Arguments
    ///
    /// * `gsv_key` - Key identifying the value
    ///
    /// # Returns
    ///
    /// A vector of mod keys, ordered from oldest to newest. Returns an empty vector
    /// if no mods have modified this value.
    fn get_gsv_edit_installers(&self, gsv_key: &str) -> Vec<String>;

    // -------------------------------------------------------------------------
    // Transactions (3 methods)
    // -------------------------------------------------------------------------

    /// Begins a new transaction.
    ///
    /// All subsequent operations are staged until either
    /// [`commit_transaction`](InstallLog::commit_transaction) or
    /// [`rollback_transaction`](InstallLog::rollback_transaction) is called.
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::TransactionAlreadyActive`] if a transaction is already in progress
    /// * [`InstallLogError::Io`] if database access fails
    fn begin_transaction(&mut self) -> Result<(), InstallLogError>;

    /// Commits the current transaction, making all staged changes permanent.
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::NoActiveTransaction`] if no transaction is active
    /// * [`InstallLogError::Io`] if database access fails
    fn commit_transaction(&mut self) -> Result<(), InstallLogError>;

    /// Rolls back the current transaction, discarding all staged changes.
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::NoActiveTransaction`] if no transaction is active
    /// * [`InstallLogError::Io`] if database access fails
    fn rollback_transaction(&mut self) -> Result<(), InstallLogError>;

    // -------------------------------------------------------------------------
    // Backup (1 method)
    // -------------------------------------------------------------------------

    /// Creates a backup of the install log.
    ///
    /// The implementation determines the backup location and naming scheme.
    ///
    /// # Errors
    ///
    /// * [`InstallLogError::Io`] if backup creation fails
    fn backup(&self) -> Result<(), InstallLogError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ini_edit_case_insensitive_equality() {
        let edit1 = IniEdit::new("Skyrim.ini", "Display", "bFullScreen");
        let edit2 = IniEdit::new("skyrim.ini", "display", "bfullscreen");
        assert_eq!(edit1, edit2, "IniEdit equality must be case-insensitive");
    }

    #[test]
    fn ini_edit_different_files_not_equal() {
        let edit1 = IniEdit::new("Skyrim.ini", "Display", "bFullScreen");
        let edit2 = IniEdit::new("SkyrimPrefs.ini", "Display", "bFullScreen");
        assert_ne!(
            edit1, edit2,
            "IniEdit with different files must not be equal"
        );
    }

    #[test]
    fn ini_edit_hash_consistency() {
        let edit1 = IniEdit::new("Skyrim.ini", "Display", "bFullScreen");
        let edit2 = IniEdit::new("skyrim.ini", "DISPLAY", "bfullscreen");

        let mut set = HashSet::new();
        set.insert(edit1);
        set.insert(edit2);

        assert_eq!(
            set.len(),
            1,
            "Case-variant IniEdits must hash to the same value"
        );
    }

    #[test]
    fn ini_edit_ord_ordering() {
        let mut edits = [
            IniEdit::new("SkyrimPrefs.ini", "Display", "iSize W"),
            IniEdit::new("Skyrim.ini", "General", "sLanguage"),
            IniEdit::new("Skyrim.ini", "Display", "bFullScreen"),
        ];

        edits.sort();

        // Expected order: Skyrim.ini Display bFullScreen < Skyrim.ini General sLanguage < SkyrimPrefs.ini
        assert_eq!(edits[0].file.to_ascii_lowercase(), "skyrim.ini");
        assert_eq!(edits[0].section.to_ascii_lowercase(), "display");

        assert_eq!(edits[1].file.to_ascii_lowercase(), "skyrim.ini");
        assert_eq!(edits[1].section.to_ascii_lowercase(), "general");

        assert_eq!(edits[2].file.to_ascii_lowercase(), "skyrimprefs.ini");
    }

    #[test]
    fn ini_edit_display() {
        let edit = IniEdit::new("Skyrim.ini", "Display", "bFullScreen");
        let display = format!("{}", edit);
        assert_eq!(display, "Skyrim.ini[Display].bFullScreen");
    }

    #[test]
    fn install_log_is_object_safe() {
        // Compile-time check: if InstallLog is object-safe, this function type-checks.
        fn _assert(_: &dyn InstallLog) {}
    }
}
