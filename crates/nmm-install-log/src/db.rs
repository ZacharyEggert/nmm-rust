//! SQLite-backed implementation of the [`InstallLog`] trait.
//!
//! [`SqliteInstallLog`] owns a single `rusqlite::Connection` and exposes the
//! full [`InstallLog`] interface.  Shared helpers used by every ownership-stack
//! group (mods, files, INI edits, GSV edits) live here so that downstream
//! task implementations (F-007 through F-010) can call them directly.

use std::path::Path;
use std::sync::Mutex;

use nmm_core::{IniEdit, InstallLog, InstallLogError, ModInfo, ORIGINAL_VALUES_KEY};
use rusqlite::Connection;

use crate::schema;

/// SQLite-backed install log.
///
/// Owns a single [`Connection`] wrapped in a [`Mutex`] so that the struct is
/// both `Send` and `Sync` as required by the [`InstallLog`] trait.
/// (`rusqlite::Connection` is `Send` but not `Sync` due to internal
/// `RefCell` usage; the `Mutex` provides the missing `Sync` bound.)
///
/// The `in_transaction` flag tracks whether a SQLite transaction is open and
/// is only mutated through `&mut self` methods, so no additional
/// synchronisation is needed for it.
pub struct SqliteInstallLog {
    conn: Mutex<Connection>,
    in_transaction: bool,
}

// ---------------------------------------------------------------------------
// Constructors & shared helpers
// ---------------------------------------------------------------------------

impl SqliteInstallLog {
    /// Open (or create) a file-based install log at `path`.
    ///
    /// Foreign keys are enabled and the schema is applied before returning.
    pub fn new(path: &Path) -> Result<Self, crate::error::InstallLogError> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// Open an in-memory database.  Useful for tests.
    pub fn open_in_memory() -> Result<Self, crate::error::InstallLogError> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn)
    }

    /// Shared initialisation: enable foreign keys, apply schema, then wrap
    /// the connection in a `Mutex`.
    fn init(conn: Connection) -> Result<Self, crate::error::InstallLogError> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        schema::apply(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            in_transaction: false,
        })
    }

    /// Atomically increment the `install_order_seq` counter and return the
    /// new value.  The row is seeded by [`schema::apply`] so this is always
    /// an UPDATE, never an INSERT.
    pub(crate) fn next_install_order(&self) -> Result<i64, InstallLogError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE schema_meta SET int_value = int_value + 1 WHERE key = 'install_order_seq'",
            [],
        )
        .map_err(Self::db_err)?;

        let seq: i64 = conn
            .query_row(
                "SELECT int_value FROM schema_meta WHERE key = 'install_order_seq'",
                [],
                |row| row.get(0),
            )
            .map_err(Self::db_err)?;

        Ok(seq)
    }

    /// Return `true` if `mod_key` has a row in the `mods` table.
    pub(crate) fn mod_exists(&self, mod_key: &str) -> Result<bool, InstallLogError> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM mods WHERE mod_key = ?1",
                [mod_key],
                |row| row.get(0),
            )
            .map_err(Self::db_err)?;
        Ok(count > 0)
    }

    /// Assert that `mod_key` is either registered in `mods` or is the special
    /// [`ORIGINAL_VALUES_KEY`] sentinel.  Returns `ModNotFound` otherwise.
    pub(crate) fn require_mod(&self, mod_key: &str) -> Result<(), InstallLogError> {
        if mod_key == ORIGINAL_VALUES_KEY {
            return Ok(());
        }
        if self.mod_exists(mod_key)? {
            return Ok(());
        }
        Err(InstallLogError::ModNotFound(mod_key.to_owned()))
    }

    /// Convert a `rusqlite::Error` into [`InstallLogError::Io`].
    ///
    /// The trait contract only provides `Io(io::Error)` for database failures,
    /// so we wrap the rusqlite error message in an `io::Error` with kind `Other`.
    fn db_err(e: rusqlite::Error) -> InstallLogError {
        InstallLogError::Io(std::io::Error::other(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// InstallLog trait — skeleton stubs for downstream tasks
// ---------------------------------------------------------------------------

impl InstallLog for SqliteInstallLog {
    // --- Mod tracking (F-007) ------------------------------------------------

    fn add_mod(&mut self, _mod_key: &str, _info: &ModInfo) -> Result<(), InstallLogError> {
        todo!("F-007: add_mod — INSERT into mods")
    }

    fn replace_mod(&mut self, _mod_key: &str, _info: &ModInfo) -> Result<(), InstallLogError> {
        todo!("F-007: replace_mod — UPDATE mods row")
    }

    fn remove_mod(&mut self, _mod_key: &str) -> Result<(), InstallLogError> {
        todo!("F-007: remove_mod — DELETE from mods (CASCADE removes children)")
    }

    fn get_mod(&self, _mod_key: &str) -> Option<ModInfo> {
        todo!("F-007: get_mod — SELECT from mods, map to ModInfo")
    }

    fn active_mods(&self) -> Vec<ModInfo> {
        todo!("F-007: active_mods — SELECT all from mods")
    }

    // --- File ownership (F-008) ----------------------------------------------

    fn add_data_file(&mut self, _mod_key: &str, _file_path: &str) -> Result<(), InstallLogError> {
        todo!("F-008: add_data_file — require_mod + next_install_order + INSERT file_owners")
    }

    fn remove_data_file(
        &mut self,
        _mod_key: &str,
        _file_path: &str,
    ) -> Result<(), InstallLogError> {
        todo!("F-008: remove_data_file — require_mod + DELETE from file_owners")
    }

    fn get_current_file_owner(&self, _file_path: &str) -> Option<String> {
        todo!("F-008: get_current_file_owner — SELECT mod_key ORDER BY install_order DESC LIMIT 1")
    }

    fn get_previous_file_owner(&self, _file_path: &str) -> Option<String> {
        todo!("F-008: get_previous_file_owner — ... LIMIT 1 OFFSET 1")
    }

    fn log_original_data_file(&mut self, _file_path: &str) -> Result<(), InstallLogError> {
        todo!("F-008: log_original_data_file — INSERT with ORIGINAL_VALUES_KEY (note: FK constraint)")
    }

    fn get_installed_mod_files(&self, _mod_key: &str) -> Result<Vec<String>, InstallLogError> {
        todo!("F-008: get_installed_mod_files — require_mod + SELECT file_path WHERE mod_key")
    }

    fn get_file_installers(&self, _file_path: &str) -> Vec<String> {
        todo!("F-008: get_file_installers — SELECT mod_key ORDER BY install_order ASC")
    }

    // --- INI edits (F-009) ---------------------------------------------------

    fn add_ini_edit(
        &mut self,
        _mod_key: &str,
        _edit: &IniEdit,
        _value: &str,
    ) -> Result<(), InstallLogError> {
        todo!("F-009: add_ini_edit — require_mod + next_install_order + INSERT ini_edits")
    }

    fn replace_ini_edit(
        &mut self,
        _mod_key: &str,
        _edit: &IniEdit,
        _value: &str,
    ) -> Result<(), InstallLogError> {
        todo!("F-009: replace_ini_edit — UPDATE ini_edits value")
    }

    fn remove_ini_edit(
        &mut self,
        _mod_key: &str,
        _edit: &IniEdit,
    ) -> Result<(), InstallLogError> {
        todo!("F-009: remove_ini_edit — DELETE from ini_edits")
    }

    fn get_current_ini_edit_owner(&self, _edit: &IniEdit) -> Option<String> {
        todo!(
            "F-009: get_current_ini_edit_owner — SELECT mod_key ORDER BY install_order DESC LIMIT 1"
        )
    }

    fn get_previous_ini_value(&self, _edit: &IniEdit) -> Option<String> {
        todo!("F-009: get_previous_ini_value — SELECT value ... LIMIT 1 OFFSET 1")
    }

    fn log_original_ini_value(
        &mut self,
        _edit: &IniEdit,
        _value: &str,
    ) -> Result<(), InstallLogError> {
        todo!("F-009: log_original_ini_value — INSERT with ORIGINAL_VALUES_KEY (note: FK constraint)")
    }

    fn get_installed_ini_edits(&self, _mod_key: &str) -> Result<Vec<IniEdit>, InstallLogError> {
        todo!("F-009: get_installed_ini_edits — require_mod + SELECT ini_file/section/key WHERE mod_key")
    }

    fn get_ini_edit_installers(&self, _edit: &IniEdit) -> Vec<String> {
        todo!("F-009: get_ini_edit_installers — SELECT mod_key ORDER BY install_order ASC")
    }

    // --- Game-specific values (F-009) ----------------------------------------

    fn add_gsv_edit(
        &mut self,
        _mod_key: &str,
        _gsv_key: &str,
        _value: &[u8],
    ) -> Result<(), InstallLogError> {
        todo!("F-009: add_gsv_edit — require_mod + next_install_order + INSERT gsv_edits")
    }

    fn replace_gsv_edit(
        &mut self,
        _mod_key: &str,
        _gsv_key: &str,
        _value: &[u8],
    ) -> Result<(), InstallLogError> {
        todo!("F-009: replace_gsv_edit — UPDATE gsv_edits blob_value")
    }

    fn remove_gsv_edit(&mut self, _mod_key: &str, _gsv_key: &str) -> Result<(), InstallLogError> {
        todo!("F-009: remove_gsv_edit — DELETE from gsv_edits")
    }

    fn get_current_gsv_edit_owner(&self, _gsv_key: &str) -> Option<String> {
        todo!("F-009: get_current_gsv_edit_owner — SELECT mod_key ORDER BY install_order DESC LIMIT 1")
    }

    fn get_previous_gsv_value(&self, _gsv_key: &str) -> Option<Vec<u8>> {
        todo!("F-009: get_previous_gsv_value — SELECT blob_value ... LIMIT 1 OFFSET 1")
    }

    fn log_original_gsv_value(
        &mut self,
        _gsv_key: &str,
        _value: &[u8],
    ) -> Result<(), InstallLogError> {
        todo!("F-009: log_original_gsv_value — INSERT with ORIGINAL_VALUES_KEY (note: FK constraint)")
    }

    fn get_installed_gsv_edits(&self, _mod_key: &str) -> Result<Vec<String>, InstallLogError> {
        todo!("F-009: get_installed_gsv_edits — require_mod + SELECT gsv_key WHERE mod_key")
    }

    fn get_gsv_edit_installers(&self, _gsv_key: &str) -> Vec<String> {
        todo!("F-009: get_gsv_edit_installers — SELECT mod_key ORDER BY install_order ASC")
    }

    // --- Transactions (F-010) ------------------------------------------------

    fn begin_transaction(&mut self) -> Result<(), InstallLogError> {
        todo!("F-010: begin_transaction — guard on in_transaction, then BEGIN")
    }

    fn commit_transaction(&mut self) -> Result<(), InstallLogError> {
        todo!("F-010: commit_transaction — guard on in_transaction, then COMMIT")
    }

    fn rollback_transaction(&mut self) -> Result<(), InstallLogError> {
        todo!("F-010: rollback_transaction — guard on in_transaction, then ROLLBACK")
    }

    // --- Backup (F-010) ------------------------------------------------------

    fn backup(&self) -> Result<(), InstallLogError> {
        todo!("F-010: backup — rusqlite backup API to sibling file")
    }
}

// ---------------------------------------------------------------------------
// Unit tests — exercises private/pub(crate) helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_log() -> SqliteInstallLog {
        SqliteInstallLog::open_in_memory().expect("open_in_memory failed")
    }

    // --- Constructors --------------------------------------------------------

    #[test]
    fn open_in_memory_succeeds() {
        let _log = fresh_log();
    }

    #[test]
    fn new_creates_file_db() {
        let dir = tempfile::tempdir().expect("tempdir failed");
        let path = dir.path().join("test.db");
        let _log = SqliteInstallLog::new(&path).expect("new failed");
        assert!(path.exists(), "database file must be created");
    }

    // --- next_install_order --------------------------------------------------

    #[test]
    fn next_install_order_starts_at_one() {
        let log = fresh_log();
        assert_eq!(log.next_install_order().unwrap(), 1);
    }

    #[test]
    fn next_install_order_increments_monotonically() {
        let log = fresh_log();
        assert_eq!(log.next_install_order().unwrap(), 1);
        assert_eq!(log.next_install_order().unwrap(), 2);
        assert_eq!(log.next_install_order().unwrap(), 3);
    }

    // --- mod_exists ----------------------------------------------------------

    #[test]
    fn mod_exists_false_when_empty() {
        let log = fresh_log();
        assert!(!log.mod_exists("anything").unwrap());
    }

    #[test]
    fn mod_exists_true_after_insert() {
        let log = fresh_log();
        log.conn
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO mods (mod_key, archive_path, name) VALUES ('test_mod', 'test.7z', 'Test')",
                [],
            )
            .unwrap();
        assert!(log.mod_exists("test_mod").unwrap());
    }

    // --- require_mod ---------------------------------------------------------

    #[test]
    fn require_mod_allows_original_values_key() {
        let log = fresh_log();
        assert!(log.require_mod(ORIGINAL_VALUES_KEY).is_ok());
    }

    #[test]
    fn require_mod_returns_mod_not_found_for_missing() {
        let log = fresh_log();
        match log.require_mod("missing") {
            Err(InstallLogError::ModNotFound(key)) => assert_eq!(key, "missing"),
            other => panic!("expected ModNotFound, got {:?}", other),
        }
    }

    #[test]
    fn require_mod_passes_for_registered_mod() {
        let log = fresh_log();
        log.conn
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO mods (mod_key, archive_path, name) VALUES ('real_mod', 'r.7z', 'Real')",
                [],
            )
            .unwrap();
        assert!(log.require_mod("real_mod").is_ok());
    }

    // --- db_err --------------------------------------------------------------

    #[test]
    fn db_err_produces_io_error() {
        let rusqlite_err = rusqlite::Error::QueryReturnedNoRows;
        let converted = SqliteInstallLog::db_err(rusqlite_err);
        match converted {
            InstallLogError::Io(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::Other);
                assert!(io_err.to_string().contains("no rows"));
            }
            _ => panic!("expected Io variant"),
        }
    }

    // --- initial state -------------------------------------------------------

    #[test]
    fn in_transaction_initially_false() {
        let log = fresh_log();
        assert!(!log.in_transaction);
    }
}
