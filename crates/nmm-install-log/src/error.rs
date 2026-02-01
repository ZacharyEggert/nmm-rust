use thiserror::Error;

/// Errors that can occur when operating on the install log database.
#[derive(Debug, Error)]
pub enum InstallLogError {
    /// A rusqlite database error.
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    /// The schema version in the database is newer than this binary knows
    /// how to handle.  Migration is not possible.
    #[error("unsupported schema version {found} (max supported: {max})")]
    UnsupportedSchemaVersion { found: i64, max: i64 },
}
