## Testing Strategy

### Unit Tests

Each crate has comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_ownership_tracking() {
        let mut log = InstallLog::new_in_memory().unwrap();

        // Install mod A
        let mod_a = create_test_mod("ModA");
        let key_a = log.add_active_mod(&mod_a).unwrap();
        log.add_data_file(&key_a, Path::new("data/texture.dds")).unwrap();

        // Install mod B (same file)
        let mod_b = create_test_mod("ModB");
        let key_b = log.add_active_mod(&mod_b).unwrap();
        log.add_data_file(&key_b, Path::new("data/texture.dds")).unwrap();

        // B is current owner, A is previous
        assert_eq!(log.get_current_file_owner(Path::new("data/texture.dds")).unwrap().name, "ModB");
        assert_eq!(log.get_previous_file_owner(Path::new("data/texture.dds")).unwrap().name, "ModA");

        // Remove B, A becomes owner again
        log.remove_data_file(&key_b, Path::new("data/texture.dds")).unwrap();
        assert_eq!(log.get_current_file_owner(Path::new("data/texture.dds")).unwrap().name, "ModA");
    }
}
```

### Integration Tests

```rust
// tests/integration/src/install_tests.rs

#[tokio::test]
async fn test_full_mod_installation_flow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let game_dir = temp_dir.path().join("game");
    let mods_dir = temp_dir.path().join("mods");
    let virtual_dir = temp_dir.path().join("virtual");

    // Set up mock game installation
    setup_mock_skyrim(&game_dir);

    // Initialize components
    let game_mode = SkyrimSEGameMode::new(&game_dir).unwrap();
    let mut vfs = VirtualModActivator::new(&virtual_dir, &game_mode).unwrap();
    let mut install_log = InstallLog::new(temp_dir.path().join("InstallLog.db")).unwrap();

    // Install a test mod
    let mod_path = create_test_fomod(&mods_dir, "TestMod", &["textures/test.dds"]);
    let test_mod = FomodFormat.create_mod(&mod_path, &game_mode).unwrap();

    // Run installation
    install_mod(&mut vfs, &mut install_log, &test_mod, &game_dir).await.unwrap();

    // Verify
    assert!(game_dir.join("Data/textures/test.dds").exists());
    assert!(install_log.get_current_file_owner(Path::new("Data/textures/test.dds")).is_some());
}
```

### Fixture Mods

Create test fixtures for common mod structures:

```
tests/fixtures/
├── simple_mod/
│   └── Data/
│       └── textures/
│           └── test.dds
├── fomod_with_script/
│   ├── fomod/
│   │   ├── info.xml
│   │   └── ModuleConfig.xml
│   └── Data/
│       ├── option_a/
│       └── option_b/
└── legacy_fomod/
    └── textures/
        └── test.dds  # Missing Data folder (legacy format)
```
