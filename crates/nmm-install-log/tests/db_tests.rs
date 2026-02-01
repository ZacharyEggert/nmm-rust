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

// --- F-008 integration tests ------------------------------------------------

#[test]
fn file_ownership_full_stack() {
    use nmm_core::ORIGINAL_VALUES_KEY;

    let mut log = SqliteInstallLog::open_in_memory().unwrap();

    // Register two mods
    log.add_mod("mod_a", &ModInfo::new("Mod A", "a.7z"))
        .unwrap();
    log.add_mod("mod_b", &ModInfo::new("Mod B", "b.7z"))
        .unwrap();

    let path = "Data/shared_texture.dds";

    // Log original value
    log.log_original_data_file(path).unwrap();

    // mod_a installs the file
    log.add_data_file("mod_a", path).unwrap();

    // mod_b overwrites it
    log.add_data_file("mod_b", path).unwrap();

    // Current owner should be mod_b
    assert_eq!(log.get_current_file_owner(path), Some("mod_b".to_string()));

    // Previous owner should be mod_a
    assert_eq!(log.get_previous_file_owner(path), Some("mod_a".to_string()));

    // Installers should be in chronological order
    let installers = log.get_file_installers(path);
    assert_eq!(installers.len(), 3);
    assert_eq!(installers[0], ORIGINAL_VALUES_KEY);
    assert_eq!(installers[1], "mod_a");
    assert_eq!(installers[2], "mod_b");

    // Both mods should report owning the file
    let mod_a_files = log.get_installed_mod_files("mod_a").unwrap();
    assert!(mod_a_files.contains(&path.to_string()));

    let mod_b_files = log.get_installed_mod_files("mod_b").unwrap();
    assert!(mod_b_files.contains(&path.to_string()));
}

#[test]
fn remove_top_of_stack_reverts_to_previous() {
    let mut log = SqliteInstallLog::open_in_memory().unwrap();

    log.add_mod("mod_a", &ModInfo::new("Mod A", "a.7z"))
        .unwrap();
    log.add_mod("mod_b", &ModInfo::new("Mod B", "b.7z"))
        .unwrap();

    let path = "Data/contested.esp";

    log.add_data_file("mod_a", path).unwrap();
    log.add_data_file("mod_b", path).unwrap();

    // mod_b is the current owner
    assert_eq!(log.get_current_file_owner(path), Some("mod_b".to_string()));

    // Remove mod_b's ownership
    log.remove_data_file("mod_b", path).unwrap();

    // mod_a should now be the current owner
    assert_eq!(log.get_current_file_owner(path), Some("mod_a".to_string()));

    // No previous owner now
    assert_eq!(log.get_previous_file_owner(path), None);
}

#[test]
fn remove_mod_clears_file_ownership() {
    let mut log = SqliteInstallLog::open_in_memory().unwrap();

    log.add_mod("temp_mod", &ModInfo::new("Temp Mod", "temp.7z"))
        .unwrap();

    let path = "Data/temp_file.nif";

    log.add_data_file("temp_mod", path).unwrap();

    // Verify ownership
    assert_eq!(
        log.get_current_file_owner(path),
        Some("temp_mod".to_string())
    );

    // Remove the mod (CASCADE should delete the file_owners row)
    log.remove_mod("temp_mod").unwrap();

    // No owner now
    assert_eq!(log.get_current_file_owner(path), None);
}
