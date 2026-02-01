//! SQLite-backed implementation of the [`InstallLog`] trait.
//!
//! [`SqliteInstallLog`] owns a single `rusqlite::Connection` and exposes the
//! full [`InstallLog`] interface.  Shared helpers used by every ownership-stack
//! group (mods, files, INI edits, GSV edits) live here so that downstream
//! task implementations (F-007 through F-010) can call them directly.

use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use nmm_core::{IniEdit, InstallLog, InstallLogError, ModInfo, ORIGINAL_VALUES_KEY};
use rusqlite::{Connection, OptionalExtension};

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

    fn add_mod(&mut self, mod_key: &str, info: &ModInfo) -> Result<(), InstallLogError> {
        if self.mod_exists(mod_key)? {
            return Err(InstallLogError::AlreadyRegistered(mod_key.to_owned()));
        }
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO mods (mod_key, archive_path, name, version, machine_version, install_date) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                mod_key,
                info.file_name,
                info.name,
                info.version,
                info.machine_version.as_ref().map(|v| v.to_string()),
                info.install_date.map(|dt| dt.to_rfc3339()),
            ],
        )
        .map_err(Self::db_err)?;
        Ok(())
    }

    fn replace_mod(&mut self, mod_key: &str, info: &ModInfo) -> Result<(), InstallLogError> {
        if mod_key == ORIGINAL_VALUES_KEY {
            return Err(InstallLogError::ModNotFound(mod_key.to_owned()));
        }
        if !self.mod_exists(mod_key)? {
            return Err(InstallLogError::ModNotFound(mod_key.to_owned()));
        }
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE mods SET archive_path = ?1, name = ?2, version = ?3, \
             machine_version = ?4, install_date = ?5 WHERE mod_key = ?6",
            rusqlite::params![
                info.file_name,
                info.name,
                info.version,
                info.machine_version.as_ref().map(|v| v.to_string()),
                info.install_date.map(|dt| dt.to_rfc3339()),
                mod_key,
            ],
        )
        .map_err(Self::db_err)?;
        Ok(())
    }

    fn remove_mod(&mut self, mod_key: &str) -> Result<(), InstallLogError> {
        if mod_key == ORIGINAL_VALUES_KEY {
            return Err(InstallLogError::ModNotFound(mod_key.to_owned()));
        }
        if !self.mod_exists(mod_key)? {
            return Err(InstallLogError::ModNotFound(mod_key.to_owned()));
        }
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM mods WHERE mod_key = ?1", [mod_key])
            .map_err(Self::db_err)?;
        Ok(())
    }

    fn get_mod(&self, mod_key: &str) -> Option<ModInfo> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT archive_path, name, version, machine_version, install_date \
             FROM mods WHERE mod_key = ?1",
            [mod_key],
            |row| {
                let file_name: String = row.get(0)?;
                let name: String = row.get(1)?;
                let version: String = row.get(2)?;
                let machine_version = row
                    .get::<_, Option<String>>(3)?
                    .and_then(|s| ModInfo::parse_version(&s));
                let install_date = row
                    .get::<_, Option<String>>(4)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));
                Ok(ModInfo {
                    file_name,
                    name,
                    version,
                    machine_version,
                    install_date,
                    ..Default::default()
                })
            },
        )
        .optional()
        .ok()
        .flatten()
    }

    fn active_mods(&self) -> Vec<ModInfo> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT archive_path, name, version, machine_version, install_date FROM mods WHERE mod_key != ?1",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let rows = stmt.query_map([ORIGINAL_VALUES_KEY], |row| {
            let file_name: String = row.get(0)?;
            let name: String = row.get(1)?;
            let version: String = row.get(2)?;
            let machine_version = row
                .get::<_, Option<String>>(3)?
                .and_then(|s| ModInfo::parse_version(&s));
            let install_date = row
                .get::<_, Option<String>>(4)?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));
            Ok(ModInfo {
                file_name,
                name,
                version,
                machine_version,
                install_date,
                ..Default::default()
            })
        });
        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => vec![],
        }
    }

    // --- File ownership (F-008) ----------------------------------------------

    fn add_data_file(&mut self, mod_key: &str, file_path: &str) -> Result<(), InstallLogError> {
        self.require_mod(mod_key)?;
        let order = self.next_install_order()?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO file_owners (file_path, mod_key, install_order) VALUES (?1, ?2, ?3)",
            rusqlite::params![file_path, mod_key, order],
        )
        .map_err(Self::db_err)?;
        Ok(())
    }

    fn remove_data_file(&mut self, mod_key: &str, file_path: &str) -> Result<(), InstallLogError> {
        self.require_mod(mod_key)?;
        let conn = self.conn.lock().unwrap();
        let deleted = conn
            .execute(
                "DELETE FROM file_owners WHERE file_path = ?1 AND mod_key = ?2",
                rusqlite::params![file_path, mod_key],
            )
            .map_err(Self::db_err)?;
        if deleted == 0 {
            return Err(InstallLogError::EntryNotFound(file_path.to_owned()));
        }
        Ok(())
    }

    fn get_current_file_owner(&self, file_path: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT mod_key FROM file_owners WHERE file_path = ?1 ORDER BY install_order DESC LIMIT 1",
            [file_path],
            |row| row.get(0),
        )
        .optional()
        .ok()
        .flatten()
    }

    fn get_previous_file_owner(&self, file_path: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT mod_key FROM file_owners WHERE file_path = ?1 ORDER BY install_order DESC LIMIT 1 OFFSET 1",
            [file_path],
            |row| row.get(0),
        )
        .optional()
        .ok()
        .flatten()
    }

    fn log_original_data_file(&mut self, file_path: &str) -> Result<(), InstallLogError> {
        let order = self.next_install_order()?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO file_owners (file_path, mod_key, install_order) VALUES (?1, ?2, ?3)",
            rusqlite::params![file_path, ORIGINAL_VALUES_KEY, order],
        )
        .map_err(Self::db_err)?;
        Ok(())
    }

    fn get_installed_mod_files(&self, mod_key: &str) -> Result<Vec<String>, InstallLogError> {
        self.require_mod(mod_key)?;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT file_path FROM file_owners WHERE mod_key = ?1")
            .map_err(Self::db_err)?;
        let rows = stmt
            .query_map([mod_key], |row| row.get(0))
            .map_err(Self::db_err)?;
        rows.collect::<Result<Vec<String>, _>>()
            .map_err(Self::db_err)
    }

    fn get_file_installers(&self, file_path: &str) -> Vec<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT mod_key FROM file_owners WHERE file_path = ?1 ORDER BY install_order ASC",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let result: Result<Vec<String>, _> = stmt
            .query_map([file_path], |row| row.get(0))
            .and_then(|rows| rows.collect());
        result.unwrap_or_else(|_| vec![])
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

    fn remove_ini_edit(&mut self, _mod_key: &str, _edit: &IniEdit) -> Result<(), InstallLogError> {
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
        todo!(
            "F-009: log_original_ini_value — INSERT with ORIGINAL_VALUES_KEY (note: FK constraint)"
        )
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
        todo!(
            "F-009: log_original_gsv_value — INSERT with ORIGINAL_VALUES_KEY (note: FK constraint)"
        )
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

    // --- add_mod -------------------------------------------------------------

    #[test]
    fn add_mod_inserts_row() {
        let mut log = fresh_log();
        let info = ModInfo {
            file_name: "TestMod.7z".into(),
            name: "Test Mod".into(),
            version: "1.2.3".into(),
            machine_version: Some(semver::Version::parse("1.2.3").unwrap()),
            install_date: Some(
                DateTime::parse_from_rfc3339("2024-06-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ..Default::default()
        };
        log.add_mod("mod_001", &info).unwrap();

        let conn = log.conn.lock().unwrap();
        let (key, archive, name, ver, mv, id): (String, String, String, String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT mod_key, archive_path, name, version, machine_version, install_date FROM mods WHERE mod_key = 'mod_001'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .unwrap();

        assert_eq!(key, "mod_001");
        assert_eq!(archive, "TestMod.7z");
        assert_eq!(name, "Test Mod");
        assert_eq!(ver, "1.2.3");
        assert_eq!(mv.as_deref(), Some("1.2.3"));
        assert!(id.is_some());
        assert!(id.unwrap().contains("2024-06-15"));
    }

    #[test]
    fn add_mod_returns_already_registered() {
        let mut log = fresh_log();
        let info = ModInfo::new("Mod A", "a.7z");
        log.add_mod("dup_key", &info).unwrap();
        match log.add_mod("dup_key", &info) {
            Err(InstallLogError::AlreadyRegistered(key)) => assert_eq!(key, "dup_key"),
            other => panic!("expected AlreadyRegistered, got {:?}", other),
        }
    }

    #[test]
    fn add_mod_with_null_optionals() {
        let mut log = fresh_log();
        let info = ModInfo {
            file_name: "Null.7z".into(),
            name: "Null Mod".into(),
            version: "0.1".into(),
            machine_version: None,
            install_date: None,
            ..Default::default()
        };
        log.add_mod("null_mod", &info).unwrap();

        let conn = log.conn.lock().unwrap();
        let (mv, id): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT machine_version, install_date FROM mods WHERE mod_key = 'null_mod'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert!(mv.is_none(), "machine_version must be NULL");
        assert!(id.is_none(), "install_date must be NULL");
    }

    // --- replace_mod ---------------------------------------------------------

    #[test]
    fn replace_mod_updates_all_columns() {
        let mut log = fresh_log();
        log.conn.lock().unwrap().execute(
            "INSERT INTO mods (mod_key, archive_path, name, version) VALUES ('rep_mod', 'old.7z', 'Old', '0.1')",
            [],
        ).unwrap();

        let new_info = ModInfo {
            file_name: "new.7z".into(),
            name: "New Name".into(),
            version: "2.0.0".into(),
            machine_version: Some(semver::Version::parse("2.0.0").unwrap()),
            install_date: Some(
                DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ..Default::default()
        };
        log.replace_mod("rep_mod", &new_info).unwrap();

        let conn = log.conn.lock().unwrap();
        let (archive, name, ver, mv, id): (String, String, String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT archive_path, name, version, machine_version, install_date FROM mods WHERE mod_key = 'rep_mod'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();
        assert_eq!(archive, "new.7z");
        assert_eq!(name, "New Name");
        assert_eq!(ver, "2.0.0");
        assert_eq!(mv.as_deref(), Some("2.0.0"));
        assert!(id.is_some());
    }

    #[test]
    fn replace_mod_returns_mod_not_found() {
        let mut log = fresh_log();
        let info = ModInfo::new("Ghost", "ghost.7z");
        match log.replace_mod("ghost_key", &info) {
            Err(InstallLogError::ModNotFound(key)) => assert_eq!(key, "ghost_key"),
            other => panic!("expected ModNotFound, got {:?}", other),
        }
    }

    #[test]
    fn replace_mod_rejects_original_values_key() {
        let mut log = fresh_log();
        let info = ModInfo::new("OV", "ov.7z");
        match log.replace_mod(ORIGINAL_VALUES_KEY, &info) {
            Err(InstallLogError::ModNotFound(key)) => assert_eq!(key, ORIGINAL_VALUES_KEY),
            other => panic!(
                "expected ModNotFound for ORIGINAL_VALUES_KEY, got {:?}",
                other
            ),
        }
    }

    // --- remove_mod ----------------------------------------------------------

    #[test]
    fn remove_mod_deletes_row() {
        let mut log = fresh_log();
        log.conn
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO mods (mod_key, archive_path, name) VALUES ('del_mod', 'd.7z', 'Del')",
                [],
            )
            .unwrap();
        assert!(log.mod_exists("del_mod").unwrap());
        log.remove_mod("del_mod").unwrap();
        assert!(!log.mod_exists("del_mod").unwrap());
    }

    #[test]
    fn remove_mod_returns_mod_not_found() {
        let mut log = fresh_log();
        match log.remove_mod("no_such_mod") {
            Err(InstallLogError::ModNotFound(key)) => assert_eq!(key, "no_such_mod"),
            other => panic!("expected ModNotFound, got {:?}", other),
        }
    }

    #[test]
    fn remove_mod_rejects_original_values_key() {
        let mut log = fresh_log();
        match log.remove_mod(ORIGINAL_VALUES_KEY) {
            Err(InstallLogError::ModNotFound(key)) => assert_eq!(key, ORIGINAL_VALUES_KEY),
            other => panic!(
                "expected ModNotFound for ORIGINAL_VALUES_KEY, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn remove_mod_cascades_child_rows() {
        let mut log = fresh_log();
        {
            let conn = log.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO mods (mod_key, archive_path, name) VALUES ('cascade_mod', 'c.7z', 'Cascade')",
                [],
            ).unwrap();
            conn.execute(
                "INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/test.dds', 'cascade_mod', 1)",
                [],
            ).unwrap();
        }

        log.remove_mod("cascade_mod").unwrap();

        let conn = log.conn.lock().unwrap();
        let child_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM file_owners WHERE mod_key = 'cascade_mod'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(child_count, 0, "CASCADE must delete child file_owners rows");
    }

    // --- get_mod -------------------------------------------------------------

    #[test]
    fn get_mod_returns_none_for_missing() {
        let log = fresh_log();
        assert!(log.get_mod("nonexistent").is_none());
    }

    #[test]
    fn get_mod_returns_populated_mod_info() {
        let mut log = fresh_log();
        let info = ModInfo {
            file_name: "Full.7z".into(),
            name: "Full Mod".into(),
            version: "3.1.4".into(),
            machine_version: Some(semver::Version::parse("3.1.4").unwrap()),
            install_date: Some(
                DateTime::parse_from_rfc3339("2024-12-25T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ..Default::default()
        };
        log.add_mod("full_mod", &info).unwrap();

        let got = log.get_mod("full_mod").unwrap();
        assert_eq!(got.file_name, "Full.7z");
        assert_eq!(got.name, "Full Mod");
        assert_eq!(got.version, "3.1.4");
        assert_eq!(
            got.machine_version,
            Some(semver::Version::parse("3.1.4").unwrap())
        );
        assert!(got.install_date.is_some());
        assert_eq!(
            got.install_date.unwrap().format("%Y-%m-%d").to_string(),
            "2024-12-25"
        );
    }

    #[test]
    fn get_mod_handles_null_optionals() {
        let mut log = fresh_log();
        let info = ModInfo {
            file_name: "Sparse.7z".into(),
            name: "Sparse".into(),
            version: "1.0".into(),
            machine_version: None,
            install_date: None,
            ..Default::default()
        };
        log.add_mod("sparse_mod", &info).unwrap();

        let got = log.get_mod("sparse_mod").unwrap();
        assert!(got.machine_version.is_none());
        assert!(got.install_date.is_none());
    }

    #[test]
    fn get_mod_unpersisted_fields_are_default() {
        let mut log = fresh_log();
        let info = ModInfo {
            file_name: "Def.7z".into(),
            name: "Def Mod".into(),
            version: "1.0".into(),
            author: Some("Someone".into()),
            id: Some("12345".into()),
            screenshot: Some(vec![0xFF, 0xD8]),
            ..Default::default()
        };
        log.add_mod("def_mod", &info).unwrap();

        let got = log.get_mod("def_mod").unwrap();
        assert!(got.author.is_none());
        assert!(got.id.is_none());
        assert!(got.screenshot.is_none());
        assert!(got.description.is_none());
        assert!(got.download_id.is_none());
    }

    // --- active_mods ---------------------------------------------------------

    #[test]
    fn active_mods_empty_when_no_mods() {
        let log = fresh_log();
        assert!(log.active_mods().is_empty());
    }

    #[test]
    fn active_mods_returns_all_mods() {
        let mut log = fresh_log();
        log.add_mod("m1", &ModInfo::new("Alpha", "a.7z")).unwrap();
        log.add_mod("m2", &ModInfo::new("Beta", "b.7z")).unwrap();
        log.add_mod("m3", &ModInfo::new("Gamma", "g.7z")).unwrap();

        let mods = log.active_mods();
        assert_eq!(mods.len(), 3);

        let names: std::collections::HashSet<&str> = mods.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains("Alpha"));
        assert!(names.contains("Beta"));
        assert!(names.contains("Gamma"));
    }

    #[test]
    fn active_mods_roundtrip_matches_get_mod() {
        let mut log = fresh_log();
        let info = ModInfo {
            file_name: "RT.7z".into(),
            name: "Roundtrip".into(),
            version: "2.5.1".into(),
            machine_version: Some(semver::Version::parse("2.5.1").unwrap()),
            install_date: Some(
                DateTime::parse_from_rfc3339("2024-03-10T08:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ..Default::default()
        };
        log.add_mod("rt_mod", &info).unwrap();

        let from_active = &log.active_mods()[0];
        let from_get = log.get_mod("rt_mod").unwrap();

        assert_eq!(from_active.file_name, from_get.file_name);
        assert_eq!(from_active.name, from_get.name);
        assert_eq!(from_active.version, from_get.version);
        assert_eq!(from_active.machine_version, from_get.machine_version);
        assert_eq!(from_active.install_date, from_get.install_date);
    }

    // --- add_data_file -------------------------------------------------------

    #[test]
    fn add_data_file_inserts_row() {
        let mut log = fresh_log();
        log.conn.lock().unwrap().execute(
            "INSERT INTO mods (mod_key, archive_path, name) VALUES ('test_mod', 'test.7z', 'Test')",
            [],
        ).unwrap();
        log.add_data_file("test_mod", "Data/test.dds").unwrap();

        let conn = log.conn.lock().unwrap();
        let (path, key, order): (String, String, i64) = conn
            .query_row(
                "SELECT file_path, mod_key, install_order FROM file_owners WHERE mod_key = 'test_mod'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(path, "Data/test.dds");
        assert_eq!(key, "test_mod");
        assert!(order > 0);
    }

    #[test]
    fn add_data_file_rejects_unknown_mod() {
        let mut log = fresh_log();
        match log.add_data_file("unknown_mod", "Data/test.dds") {
            Err(InstallLogError::ModNotFound(key)) => assert_eq!(key, "unknown_mod"),
            other => panic!("expected ModNotFound, got {:?}", other),
        }
    }

    #[test]
    fn add_data_file_assigns_monotonic_install_order() {
        let mut log = fresh_log();
        log.conn.lock().unwrap().execute(
            "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mono_mod', 'm.7z', 'Mono')",
            [],
        ).unwrap();
        log.add_data_file("mono_mod", "Data/file1.dds").unwrap();
        log.add_data_file("mono_mod", "Data/file2.dds").unwrap();

        let conn = log.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT install_order FROM file_owners WHERE mod_key = 'mono_mod' ORDER BY install_order").unwrap();
        let orders: Vec<i64> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(orders.len(), 2);
        assert!(
            orders[1] > orders[0],
            "install_order must be monotonically increasing"
        );
    }

    // --- remove_data_file ----------------------------------------------------

    #[test]
    fn remove_data_file_deletes_row() {
        let mut log = fresh_log();
        log.conn.lock().unwrap().execute_batch(
            "INSERT INTO mods (mod_key, archive_path, name) VALUES ('del_file_mod', 'd.7z', 'Del');
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/removeme.dds', 'del_file_mod', 1);"
        ).unwrap();

        log.remove_data_file("del_file_mod", "Data/removeme.dds")
            .unwrap();

        let conn = log.conn.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM file_owners WHERE file_path = 'Data/removeme.dds'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn remove_data_file_rejects_unknown_mod() {
        let mut log = fresh_log();
        match log.remove_data_file("ghost_mod", "Data/file.dds") {
            Err(InstallLogError::ModNotFound(key)) => assert_eq!(key, "ghost_mod"),
            other => panic!("expected ModNotFound, got {:?}", other),
        }
    }

    #[test]
    fn remove_data_file_rejects_missing_entry() {
        let mut log = fresh_log();
        log.conn.lock().unwrap().execute(
            "INSERT INTO mods (mod_key, archive_path, name) VALUES ('exists_mod', 'e.7z', 'Exists')",
            [],
        ).unwrap();
        match log.remove_data_file("exists_mod", "Data/missing.dds") {
            Err(InstallLogError::EntryNotFound(path)) => assert_eq!(path, "Data/missing.dds"),
            other => panic!("expected EntryNotFound, got {:?}", other),
        }
    }

    // --- get_current_file_owner ----------------------------------------------

    #[test]
    fn get_current_file_owner_none_when_empty() {
        let log = fresh_log();
        assert!(log.get_current_file_owner("Data/none.dds").is_none());
    }

    #[test]
    fn get_current_file_owner_returns_latest() {
        let log = fresh_log();
        log.conn.lock().unwrap().execute_batch(
            "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod_a', 'a.7z', 'A');
             INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod_b', 'b.7z', 'B');
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/shared.dds', 'mod_a', 1);
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/shared.dds', 'mod_b', 2);"
        ).unwrap();

        let owner = log.get_current_file_owner("Data/shared.dds");
        assert_eq!(owner, Some("mod_b".to_string()));
    }

    // --- get_previous_file_owner ---------------------------------------------

    #[test]
    fn get_previous_file_owner_none_with_single_owner() {
        let log = fresh_log();
        log.conn.lock().unwrap().execute_batch(
            "INSERT INTO mods (mod_key, archive_path, name) VALUES ('solo_mod', 's.7z', 'Solo');
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/solo.dds', 'solo_mod', 1);"
        ).unwrap();

        assert!(log.get_previous_file_owner("Data/solo.dds").is_none());
    }

    #[test]
    fn get_previous_file_owner_returns_second() {
        let log = fresh_log();
        log.conn.lock().unwrap().execute_batch(
            "INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod_x', 'x.7z', 'X');
             INSERT INTO mods (mod_key, archive_path, name) VALUES ('mod_y', 'y.7z', 'Y');
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/prev.dds', 'mod_x', 1);
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/prev.dds', 'mod_y', 2);"
        ).unwrap();

        let prev = log.get_previous_file_owner("Data/prev.dds");
        assert_eq!(prev, Some("mod_x".to_string()));
    }

    // --- log_original_data_file ----------------------------------------------

    #[test]
    fn log_original_data_file_inserts_sentinel() {
        let mut log = fresh_log();
        log.log_original_data_file("Data/original.dds").unwrap();

        let conn = log.conn.lock().unwrap();
        let (path, key): (String, String) = conn
            .query_row(
                "SELECT file_path, mod_key FROM file_owners WHERE file_path = 'Data/original.dds'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(path, "Data/original.dds");
        assert_eq!(key, ORIGINAL_VALUES_KEY);
    }

    // --- get_installed_mod_files ---------------------------------------------

    #[test]
    fn get_installed_mod_files_returns_all() {
        let log = fresh_log();
        log.conn.lock().unwrap().execute_batch(
            "INSERT INTO mods (mod_key, archive_path, name) VALUES ('files_mod', 'f.7z', 'Files');
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/file1.dds', 'files_mod', 1);
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/file2.dds', 'files_mod', 2);"
        ).unwrap();

        let files = log.get_installed_mod_files("files_mod").unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"Data/file1.dds".to_string()));
        assert!(files.contains(&"Data/file2.dds".to_string()));
    }

    #[test]
    fn get_installed_mod_files_rejects_unknown_mod() {
        let log = fresh_log();
        match log.get_installed_mod_files("unknown") {
            Err(InstallLogError::ModNotFound(key)) => assert_eq!(key, "unknown"),
            other => panic!("expected ModNotFound, got {:?}", other),
        }
    }

    #[test]
    fn get_installed_mod_files_works_for_original_values_key() {
        let mut log = fresh_log();
        log.log_original_data_file("Data/orig.dds").unwrap();

        let files = log.get_installed_mod_files(ORIGINAL_VALUES_KEY).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "Data/orig.dds");
    }

    // --- get_file_installers -------------------------------------------------

    #[test]
    fn get_file_installers_empty_for_unknown_file() {
        let log = fresh_log();
        let installers = log.get_file_installers("Data/unknown.dds");
        assert!(installers.is_empty());
    }

    #[test]
    fn get_file_installers_returns_asc_order() {
        let log = fresh_log();
        log.conn.lock().unwrap().execute_batch(
            "INSERT INTO mods (mod_key, archive_path, name) VALUES ('first_mod', '1.7z', 'First');
             INSERT INTO mods (mod_key, archive_path, name) VALUES ('second_mod', '2.7z', 'Second');
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/multi.dds', 'first_mod', 1);
             INSERT INTO file_owners (file_path, mod_key, install_order) VALUES ('Data/multi.dds', 'second_mod', 2);"
        ).unwrap();

        let installers = log.get_file_installers("Data/multi.dds");
        assert_eq!(installers.len(), 2);
        assert_eq!(installers[0], "first_mod");
        assert_eq!(installers[1], "second_mod");
    }

    // --- active_mods excludes sentinel ---------------------------------------

    #[test]
    fn active_mods_excludes_original_values_sentinel() {
        let log = fresh_log();
        // Sentinel is seeded by schema, so active_mods should be empty
        assert!(log.active_mods().is_empty());
    }
}
