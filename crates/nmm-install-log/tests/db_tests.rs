use nmm_install_log::SqliteInstallLog;

#[test]
fn in_memory_log_constructs() {
    let _log = SqliteInstallLog::open_in_memory().expect("open_in_memory must succeed");
}

#[test]
fn file_based_log_constructs() {
    let dir = tempfile::tempdir().expect("tempdir failed");
    let db_path = dir.path().join("test_install_log.db");
    let _log = SqliteInstallLog::new(&db_path).expect("new must succeed");
    assert!(db_path.exists(), "database file must be created");
}

#[test]
fn file_based_log_is_reentrant() {
    let dir = tempfile::tempdir().expect("tempdir failed");
    let db_path = dir.path().join("reentrant.db");
    {
        let _log = SqliteInstallLog::new(&db_path).expect("first open");
    }
    // Second open must not fail — schema::apply is idempotent.
    let _log = SqliteInstallLog::new(&db_path).expect("second open must succeed");
}

#[test]
fn sqlite_install_log_is_trait_object_safe() {
    let log = SqliteInstallLog::open_in_memory().unwrap();
    fn _takes_trait_object(_: &dyn nmm_core::InstallLog) {}
    _takes_trait_object(&log);
}
