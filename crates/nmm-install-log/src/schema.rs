/// The schema version produced by this crate.  Bump when a migration is added.
pub const CURRENT_VERSION: i64 = 1;

/// The complete DDL for schema version 1.  Every statement is guarded with
/// `IF NOT EXISTS` so the block is safe to re-execute.
const DDL_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_meta (
    key        TEXT PRIMARY KEY,
    int_value  INTEGER,
    text_value TEXT
);

CREATE TABLE IF NOT EXISTS mods (
    mod_key          TEXT PRIMARY KEY,
    archive_path     TEXT    NOT NULL,
    name             TEXT    NOT NULL,
    version          TEXT    NOT NULL DEFAULT '',
    machine_version  TEXT,
    install_date     TEXT
);

CREATE TABLE IF NOT EXISTS file_owners (
    file_path     TEXT    NOT NULL COLLATE NOCASE,
    mod_key       TEXT    NOT NULL,
    install_order INTEGER NOT NULL,
    PRIMARY KEY (file_path, mod_key),
    FOREIGN KEY (mod_key) REFERENCES mods(mod_key) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ini_edits (
    ini_file      TEXT    NOT NULL COLLATE NOCASE,
    section       TEXT    NOT NULL COLLATE NOCASE,
    key           TEXT    NOT NULL COLLATE NOCASE,
    mod_key       TEXT    NOT NULL,
    value         TEXT,
    install_order INTEGER NOT NULL,
    PRIMARY KEY (ini_file, section, key, mod_key),
    FOREIGN KEY (mod_key) REFERENCES mods(mod_key) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS gsv_edits (
    gsv_key       TEXT    NOT NULL COLLATE NOCASE,
    mod_key       TEXT    NOT NULL,
    blob_value    BLOB,
    install_order INTEGER NOT NULL,
    PRIMARY KEY (gsv_key, mod_key),
    FOREIGN KEY (mod_key) REFERENCES mods(mod_key) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_file_owners_by_path
    ON file_owners (file_path, install_order DESC);

CREATE INDEX IF NOT EXISTS idx_file_owners_by_mod
    ON file_owners (mod_key);

CREATE INDEX IF NOT EXISTS idx_ini_edits_by_key
    ON ini_edits (ini_file, section, key, install_order DESC);

CREATE INDEX IF NOT EXISTS idx_ini_edits_by_mod
    ON ini_edits (mod_key);

CREATE INDEX IF NOT EXISTS idx_gsv_edits_by_key
    ON gsv_edits (gsv_key, install_order DESC);

CREATE INDEX IF NOT EXISTS idx_gsv_edits_by_mod
    ON gsv_edits (mod_key);
"#;

/// Seed rows inserted after DDL.  Uses `INSERT OR IGNORE` so the block is
/// idempotent.
const SEED_V1: &str = r#"
INSERT OR IGNORE INTO schema_meta (key, int_value) VALUES ('schema_version', 1);
INSERT OR IGNORE INTO schema_meta (key, int_value) VALUES ('install_order_seq', 0);
"#;

use crate::error::InstallLogError;
use rusqlite::Connection;

/// Read the current schema version from the database.
///
/// Returns `Ok(0)` if `schema_meta` does not exist yet (fresh database).
fn read_version(conn: &Connection) -> Result<i64, InstallLogError> {
    let table_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_meta'",
        [],
        |row| row.get(0),
    )?;

    if table_count == 0 {
        return Ok(0);
    }

    let version: i64 = conn
        .query_row(
            "SELECT int_value FROM schema_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(version)
}

/// Apply the install-log schema to `conn`, creating tables and indices as
/// needed.
///
/// This function is **idempotent**: if the schema is already at
/// [`CURRENT_VERSION`], it returns immediately.  On a fresh database (no
/// `schema_meta` table) it runs the full DDL.  On an older version it
/// applies sequential migrations (none exist yet beyond version 1, but the
/// plumbing is in place).
///
/// The caller is responsible for enabling foreign keys on the connection
/// (`PRAGMA foreign_keys = ON`) before calling this function if FK
/// enforcement is desired.
///
/// # Errors
///
/// Returns [`InstallLogError::UnsupportedSchemaVersion`] if the database
/// already contains a version newer than [`CURRENT_VERSION`].
///
/// Returns [`InstallLogError::Db`] on any SQLite error.
pub fn apply(conn: &Connection) -> Result<(), InstallLogError> {
    let current = read_version(conn)?;

    if current > CURRENT_VERSION {
        return Err(InstallLogError::UnsupportedSchemaVersion {
            found: current,
            max: CURRENT_VERSION,
        });
    }

    if current == CURRENT_VERSION {
        return Ok(());
    }

    // Version 0 -> 1: full initial schema.
    if current < 1 {
        conn.execute_batch(DDL_V1)?;
        conn.execute_batch(SEED_V1)?;
    }

    // Future migrations would be added here as:
    //   if current < 2 { ... }
    //   if current < 3 { ... }

    Ok(())
}
