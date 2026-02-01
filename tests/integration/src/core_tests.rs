use nmm_core::{
    FormatConfidence, GameModeDescriptor, GameTheme, IniEdit, InstallLog, InstallLogError, Mod,
    ModFormatError, ModFormatRegistry, ModInfo, PluginError, ORIGINAL_VALUES_KEY,
};
use std::path::{Path, PathBuf};

use crate::{fixtures_dir, symlinks_supported, MockGameDir};

// ---------------------------------------------------------------------------
// Fixture round-trip
// ---------------------------------------------------------------------------

#[test]
fn fixture_mod_info_round_trip() {
    let json_path = fixtures_dir().join("mod_info_sample.json");
    let raw = std::fs::read_to_string(&json_path)
        .unwrap_or_else(|e| panic!("failed to read fixture {json_path:?}: {e}"));

    let original: ModInfo =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("deserialize failed: {e}"));

    // Verify key fields survived the first parse.
    assert_eq!(original.name, "Sample Texture Pack");
    assert_eq!(original.id, Some("12345".into()));
    assert_eq!(original.author, Some("TestAuthor".into()));
    assert!(original.is_endorsed == Some(true));
    assert!(original.update_warning_enabled);
    assert!(original.update_checks_enabled);

    // Serialize → deserialize and assert equality.
    let re_serialized = serde_json::to_string(&original).unwrap();
    let round_tripped: ModInfo = serde_json::from_str(&re_serialized).unwrap();

    assert_eq!(original.name, round_tripped.name);
    assert_eq!(original.id, round_tripped.id);
    assert_eq!(original.download_id, round_tripped.download_id);
    assert_eq!(original.file_name, round_tripped.file_name);
    assert_eq!(original.version, round_tripped.version);
    assert_eq!(original.machine_version, round_tripped.machine_version);
    assert_eq!(original.author, round_tripped.author);
    assert_eq!(original.description, round_tripped.description);
    assert_eq!(original.category_id, round_tripped.category_id);
    assert_eq!(
        original.custom_category_id,
        round_tripped.custom_category_id
    );
    assert_eq!(original.website, round_tripped.website);
    assert_eq!(original.download_date, round_tripped.download_date);
    assert_eq!(original.install_date, round_tripped.install_date);
    assert_eq!(original.is_endorsed, round_tripped.is_endorsed);
    assert_eq!(original.load_order, round_tripped.load_order);
    assert_eq!(original.screenshot, round_tripped.screenshot);
    assert_eq!(
        original.update_warning_enabled,
        round_tripped.update_warning_enabled
    );
    assert_eq!(
        original.update_checks_enabled,
        round_tripped.update_checks_enabled
    );
    assert_eq!(original.new_load_order, round_tripped.new_load_order);
}

// ---------------------------------------------------------------------------
// MockGameDir + GameModeDescriptor
// ---------------------------------------------------------------------------

struct TestGameDescriptor;

impl GameModeDescriptor for TestGameDescriptor {
    fn mode_id(&self) -> &str {
        "TestGame"
    }
    fn name(&self) -> &str {
        "Test Game"
    }
    fn game_executables(&self) -> &[&str] {
        &["MockGame.exe"]
    }
    fn plugin_extensions(&self) -> &[&str] {
        &[".esp", ".esm"]
    }
    fn critical_plugins(&self) -> &[&str] {
        &["Base.esm"]
    }
    fn official_plugins(&self) -> &[&str] {
        &["Base.esm"]
    }
    fn stop_folders(&self) -> &[&str] {
        &["Data"]
    }
    fn theme(&self) -> GameTheme {
        GameTheme {
            primary_color: "#aabbcc".into(),
            icon_path: None,
        }
    }
}

#[test]
fn mock_game_dir_layout() {
    let mock = MockGameDir::new().expect("MockGameDir::new failed");

    // Root and exe exist.
    assert!(mock.path().exists(), "mock game root must exist");
    assert!(
        mock.path().join("MockGame.exe").exists(),
        "MockGame.exe must exist"
    );

    // Data dir exists.
    let data = mock.data_dir();
    assert!(data.exists(), "Data/ must exist");
    assert!(data.is_dir(), "Data/ must be a directory");

    // Descriptor metadata checks.
    let desc = TestGameDescriptor;
    assert_eq!(desc.mode_id(), "TestGame");
    assert_eq!(desc.name(), "Test Game");
    assert_eq!(desc.game_executables(), &["MockGame.exe"]);
    assert_eq!(desc.plugin_extensions(), &[".esp", ".esm"]);
    assert_eq!(desc.critical_plugins(), &["Base.esm"]);
    assert_eq!(desc.stop_folders(), &["Data"]);
    assert_eq!(desc.theme().primary_color, "#aabbcc");
    assert_eq!(desc.max_active_plugins(), 0);
    assert!(desc.required_tool_name().is_none());
}

// ---------------------------------------------------------------------------
// FormatConfidence + ModFormatRegistry
// ---------------------------------------------------------------------------

/// A mock format that returns `Match` for `.fomod` paths and `Incompatible`
/// for everything else.
struct FomodMockFormat;

impl nmm_core::ModFormat for FomodMockFormat {
    fn name(&self) -> &str {
        "FOMod"
    }
    fn id(&self) -> &str {
        "FOMod"
    }
    fn extension(&self) -> &str {
        ".fomod"
    }
    fn supports_compression(&self) -> bool {
        false
    }
    fn check_compliance(&self, path: &Path) -> FormatConfidence {
        if path.extension().and_then(|e| e.to_str()) == Some("fomod") {
            FormatConfidence::Match
        } else {
            FormatConfidence::Incompatible
        }
    }
    fn create_mod(
        &self,
        _path: &Path,
        _game_mode: &dyn nmm_core::GameMode,
    ) -> Result<Box<dyn Mod>, ModFormatError> {
        Err(ModFormatError::UnsupportedFormat)
    }
}

/// A mock format that always returns `Compatible` — lower priority than Match.
struct GenericMockFormat;

impl nmm_core::ModFormat for GenericMockFormat {
    fn name(&self) -> &str {
        "Generic"
    }
    fn id(&self) -> &str {
        "Generic"
    }
    fn extension(&self) -> &str {
        ".zip"
    }
    fn supports_compression(&self) -> bool {
        true
    }
    fn check_compliance(&self, _path: &Path) -> FormatConfidence {
        FormatConfidence::Compatible
    }
    fn create_mod(
        &self,
        _path: &Path,
        _game_mode: &dyn nmm_core::GameMode,
    ) -> Result<Box<dyn Mod>, ModFormatError> {
        Err(ModFormatError::UnsupportedFormat)
    }
}

#[test]
fn format_registry_detect_best_match() {
    let mut registry = ModFormatRegistry::new();
    registry.register(Box::new(GenericMockFormat));
    registry.register(Box::new(FomodMockFormat));

    // A .fomod path should resolve to the FomodMockFormat (Match > Compatible).
    let best = registry
        .detect_format(Path::new("mods/my_mod.fomod"))
        .expect("detect_format should find a match");
    assert_eq!(best.id(), "FOMod");
    assert_eq!(best.name(), "FOMod");

    // A .zip path: only GenericMockFormat returns Compatible; FomodMockFormat
    // returns Incompatible, so Generic wins.
    let best = registry
        .detect_format(Path::new("mods/other_mod.zip"))
        .expect("detect_format should find Generic for .zip");
    assert_eq!(best.id(), "Generic");

    // A path that nothing matches (Generic returns Compatible for everything,
    // so it will always match — verify we still get Generic).
    let best = registry
        .detect_format(Path::new("mods/unknown.xyz"))
        .expect("Generic always returns Compatible");
    assert_eq!(best.id(), "Generic");
}

#[test]
fn format_registry_get_by_id() {
    let mut registry = ModFormatRegistry::new();
    registry.register(Box::new(FomodMockFormat));
    registry.register(Box::new(GenericMockFormat));

    assert!(registry.get_format("FOMod").is_some());
    assert!(registry.get_format("Generic").is_some());
    assert!(registry.get_format("NonExistent").is_none());
}

#[test]
fn format_registry_empty_returns_none() {
    let registry = ModFormatRegistry::new();
    assert!(registry
        .detect_format(Path::new("anything.fomod"))
        .is_none());
}

// ---------------------------------------------------------------------------
// PluginError conversions and Display
// ---------------------------------------------------------------------------

#[test]
fn plugin_error_invalid_display() {
    let e = PluginError::Invalid("corrupt header".into());
    assert_eq!(e.to_string(), "Invalid plugin: corrupt header");
}

#[test]
fn plugin_error_not_found_display() {
    let e = PluginError::NotFound(PathBuf::from("Data/plugins/missing.esp"));
    assert_eq!(e.to_string(), "Plugin not found: Data/plugins/missing.esp");
}

#[test]
fn plugin_error_missing_master_display() {
    let e = PluginError::MissingMaster("Skyrim.esm".into());
    assert_eq!(e.to_string(), "Missing master: Skyrim.esm");
}

#[test]
fn plugin_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let plugin_err = PluginError::from(io_err);
    assert!(
        plugin_err.to_string().contains("IO error"),
        "expected IO error wrapper, got: {plugin_err}"
    );
    assert!(
        plugin_err.to_string().contains("access denied"),
        "expected inner message preserved, got: {plugin_err}"
    );
}

// ---------------------------------------------------------------------------
// Symlink-conditional test
// ---------------------------------------------------------------------------

#[test]
fn symlink_inside_mock_game_dir() {
    if !symlinks_supported() {
        eprintln!("SKIPPED: symlinks not supported on this platform/configuration");
        return;
    }

    let mock = MockGameDir::new().expect("MockGameDir::new failed");

    // Create a real file inside Data/.
    let source = mock.data_dir().join("original.txt");
    std::fs::write(&source, b"hello symlink").expect("write source file");

    // Create a symlink next to it.
    let link = mock.data_dir().join("link.txt");

    #[cfg(target_os = "windows")]
    {
        std::os::windows::fs::symlink_file(&source, &link).expect("symlink_file failed on Windows");
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(&source, &link).expect("symlink failed on Unix");
    }

    // The symlink must exist and resolve to the same content.
    assert!(link.exists(), "symlink must exist");
    let content = std::fs::read_to_string(&link).expect("read through symlink");
    assert_eq!(content, "hello symlink");
}

// ---------------------------------------------------------------------------
// InstallLog trait and IniEdit integration tests
// ---------------------------------------------------------------------------

#[test]
fn install_log_trait_importable() {
    // Verify that the trait, struct, error, and constant are all accessible from nmm_core.
    let edit = IniEdit::new("Test.ini", "Section", "Key");
    assert_eq!(edit.file, "Test.ini");
    assert_eq!(edit.section, "Section");
    assert_eq!(edit.key, "Key");

    // Verify we can name the trait object type.
    fn _takes_install_log(_log: &dyn InstallLog) {}

    // Verify all error variants are constructible.
    let _e1 = InstallLogError::ModNotFound("test".into());
    let _e2 = InstallLogError::AlreadyRegistered("test".into());
    let _e3 = InstallLogError::EntryNotFound("test".into());
    let _e4 = InstallLogError::NoActiveTransaction;
    let _e5 = InstallLogError::TransactionAlreadyActive;

    // Verify constant is accessible.
    assert_eq!(ORIGINAL_VALUES_KEY, "<<ORIGINAL_VALUES>>");
}

#[test]
fn ini_edit_case_insensitive_round_trip() {
    use std::collections::HashSet;

    let edit1 = IniEdit::new("Skyrim.ini", "Display", "bFullScreen");
    let edit2 = IniEdit::new("SKYRIM.INI", "DISPLAY", "BFULLSCREEN");

    // Equality must be case-insensitive.
    assert_eq!(
        edit1, edit2,
        "IniEdit equality must ignore ASCII case differences"
    );

    // HashSet must treat them as the same item.
    let mut set = HashSet::new();
    set.insert(edit1);
    set.insert(edit2);

    assert_eq!(
        set.len(),
        1,
        "HashSet must deduplicate case-variant IniEdits"
    );
}

#[test]
fn install_log_error_variants_display() {
    let e = InstallLogError::ModNotFound("my_mod_123".into());
    let display = e.to_string();
    assert!(!display.is_empty(), "Error display must not be empty");
    assert!(
        display.contains("my_mod_123"),
        "Error display must include context"
    );

    let e = InstallLogError::AlreadyRegistered("duplicate_mod".into());
    let display = e.to_string();
    assert!(!display.is_empty());
    assert!(display.contains("duplicate_mod"));

    let e = InstallLogError::EntryNotFound("Data/test.dds".into());
    let display = e.to_string();
    assert!(!display.is_empty());
    assert!(display.contains("Data/test.dds"));

    let e = InstallLogError::NoActiveTransaction;
    assert_eq!(
        e.to_string(),
        "No active transaction",
        "Specific message expected"
    );

    let e = InstallLogError::TransactionAlreadyActive;
    assert_eq!(
        e.to_string(),
        "Transaction already active",
        "Specific message expected"
    );

    // Test From<io::Error>.
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let log_err = InstallLogError::from(io_err);
    let display = log_err.to_string();
    assert!(
        display.contains("IO error"),
        "From<io::Error> must wrap properly"
    );
    assert!(
        display.contains("access denied"),
        "Inner message must be preserved"
    );
}
