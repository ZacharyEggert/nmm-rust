use chrono::{DateTime, Utc};
use nmm_core::{InstallLog, ModInfo};
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

// --- F-007 integration tests ------------------------------------------------

#[test]
fn add_then_get_roundtrip() {
    let mut log = SqliteInstallLog::open_in_memory().unwrap();
    let info = ModInfo {
        file_name: "Integration.7z".into(),
        name: "Integration Mod".into(),
        version: "4.2.0".into(),
        machine_version: Some(semver::Version::parse("4.2.0").unwrap()),
        install_date: Some(
            DateTime::parse_from_rfc3339("2024-08-20T15:45:00Z")
                .unwrap()
                .with_timezone(&Utc),
        ),
        ..Default::default()
    };

    log.add_mod("integ_mod", &info).unwrap();
    let got = log.get_mod("integ_mod").unwrap();

    assert_eq!(got.file_name, "Integration.7z");
    assert_eq!(got.name, "Integration Mod");
    assert_eq!(got.version, "4.2.0");
    assert_eq!(
        got.machine_version,
        Some(semver::Version::parse("4.2.0").unwrap())
    );
    assert!(got.install_date.is_some());
    assert_eq!(
        got.install_date.unwrap().format("%Y-%m-%d").to_string(),
        "2024-08-20"
    );
}

#[test]
fn add_replace_get_roundtrip() {
    let mut log = SqliteInstallLog::open_in_memory().unwrap();
    let original = ModInfo {
        file_name: "Original.7z".into(),
        name: "Original".into(),
        version: "1.0.0".into(),
        machine_version: Some(semver::Version::parse("1.0.0").unwrap()),
        install_date: Some(
            DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        ),
        ..Default::default()
    };
    log.add_mod("replace_me", &original).unwrap();

    let updated = ModInfo {
        file_name: "Updated.7z".into(),
        name: "Updated Name".into(),
        version: "2.1.0".into(),
        machine_version: Some(semver::Version::parse("2.1.0").unwrap()),
        install_date: Some(
            DateTime::parse_from_rfc3339("2025-06-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        ),
        ..Default::default()
    };
    log.replace_mod("replace_me", &updated).unwrap();

    let got = log.get_mod("replace_me").unwrap();
    assert_eq!(got.file_name, "Updated.7z");
    assert_eq!(got.name, "Updated Name");
    assert_eq!(got.version, "2.1.0");
    assert_eq!(
        got.machine_version,
        Some(semver::Version::parse("2.1.0").unwrap())
    );
    assert_eq!(
        got.install_date.unwrap().format("%Y-%m-%d").to_string(),
        "2025-06-01"
    );
}

#[test]
fn add_remove_get_roundtrip() {
    let mut log = SqliteInstallLog::open_in_memory().unwrap();
    let info = ModInfo::new("Removable", "removable.7z");

    log.add_mod("remove_me", &info).unwrap();
    assert!(log.get_mod("remove_me").is_some());

    log.remove_mod("remove_me").unwrap();
    assert!(log.get_mod("remove_me").is_none());
}
