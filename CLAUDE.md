# Nexus Mod Manager Rust - Claude Instructions

## Project Context

This is a cross-platform Rust rewrite of Nexus Mod Manager (NMM), originally a C#/.NET Framework 4.6.1 Windows application for managing video game mods, particularly for Bethesda games (Skyrim, Fallout, etc.).

**Goals**:
- Cross-platform support (Windows and macOS)
- Modern Rust architecture with clean trait abstractions
- Performance improvements over the C# version
- Tauri-based desktop UI with React frontend

**Original Codebase Reference**: `/Users/zachary/code/nmm/Nexus-Mod-Manager/`

## Build & Test Commands

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p nmm-core

# Run all tests
cargo test

# Run tests for specific crate
cargo test -p nmm-install-log

# Run with logging
RUST_LOG=debug cargo test

# Check formatting
cargo fmt --check

# Run clippy lints
cargo clippy --all-targets

# Build release
cargo build --release

# Run CLI
cargo run -p nmm-cli -- --help

# Build Tauri app (from apps/nmm-desktop)
cd apps/nmm-desktop && npm run tauri build
```

## Coding Conventions

### Rust Style

1. **Error Handling**: Use `thiserror` for library errors, `anyhow` for application-level errors

```rust
// In library crates (nmm-core, nmm-vfs, etc.)
#[derive(Debug, thiserror::Error)]
pub enum ModError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// In application code (nmm-cli, nmm-desktop)
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let content = fs::read_to_string("config.toml")
        .context("Failed to read config file")?;
    // ...
}
```

2. **Traits**: Define traits in `nmm-core`, implement in specific crates

```rust
// Good: Trait in nmm-core, implementation in nmm-games
// nmm-core/src/game_mode.rs
pub trait GameMode: Send + Sync { /* ... */ }

// nmm-games/src/skyrim/skyrim_se.rs
impl GameMode for SkyrimSEGameMode { /* ... */ }
```

3. **Async**: Use `tokio` for async operations, prefer async for IO

```rust
pub async fn install_mod(
    vfs: &mut dyn VirtualModActivator,
    mod_archive: &dyn Mod,
) -> Result<(), InstallError> {
    // File operations should be async
    let files = mod_archive.file_list().await?;
    // ...
}
```

4. **Logging**: Use `tracing` with structured logging

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument(skip(vfs), fields(mod_name = %mod_info.name()))]
pub async fn enable_mod(
    vfs: &mut dyn VirtualModActivator,
    mod_info: &dyn Mod,
) -> Result<(), VfsError> {
    info!("Enabling mod");
    // ...
    debug!(file_count = files.len(), "Created links");
}
```

5. **Documentation**: Document public APIs with examples

```rust
/// Creates a virtual file link for a mod file.
///
/// # Arguments
///
/// * `source` - Path to the source file in the mod's virtual install folder
/// * `target` - Path where the link should be created in the game folder
/// * `priority` - Link priority (higher = wins conflicts)
///
/// # Example
///
/// ```rust
/// use nmm_vfs::VirtualModActivator;
///
/// let mut vfs = VirtualModActivator::new(virtual_path, game_mode)?;
/// vfs.add_file_link(&mod_info, source, target, 0)?;
/// ```
///
/// # Errors
///
/// Returns `VfsError::PermissionDenied` if symlinks are not available.
pub fn add_file_link(&mut self, /* ... */) -> Result<(), VfsError>;
```

### File Organization

```
crate/
├── src/
│   ├── lib.rs         # Public exports, module declarations
│   ├── error.rs       # Error types
│   ├── types.rs       # Common types/structs
│   └── feature/
│       ├── mod.rs     # Feature module
│       └── impl.rs    # Implementation details
├── tests/
│   └── integration.rs # Integration tests
└── Cargo.toml
```

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Crates | `nmm-*` | `nmm-core`, `nmm-vfs` |
| Traits | PascalCase, verb/noun | `GameMode`, `VirtualModActivator` |
| Structs | PascalCase | `ModInfo`, `VirtualModLink` |
| Functions | snake_case | `add_file_link`, `get_current_owner` |
| Constants | SCREAMING_SNAKE | `MAX_ACTIVE_PLUGINS` |
| Feature flags | kebab-case | `tauri`, `async-std` |

## Architectural Decisions

### 1. SQLite for Install Log (Instead of XML)

**Rationale**: The C# version uses XML files which are slow for large installations and prone to corruption. SQLite provides:
- ACID transactions for reliable rollback
- Fast queries for ownership lookups
- Concurrent read access

**Migration**: Provide XML import for existing NMM users.

### 2. Traits for Game Modes (Instead of Class Hierarchy)

**Rationale**: Rust doesn't have class inheritance. Traits provide:
- Composition over inheritance
- Clear contracts for game mode implementers
- Easy mocking for tests

```rust
// C# version had: GameModeBase -> GamebryoGameModeBase -> SkyrimSEGameMode
// Rust version uses: GameMode trait + GamebryoMixin helper functions

pub trait GameMode: GameModeDescriptor + Send + Sync {
    fn installation_path(&self) -> &Path;
    fn uses_plugins(&self) -> bool;
    // ...
}

// Shared Gamebryo functionality as helper functions
mod gamebryo {
    pub fn hardlink_required_extensions() -> HashSet<&'static str> {
        [".esp", ".esm", ".esl", ".bsa"].into_iter().collect()
    }

    pub fn adjust_fomod_path(path: &str) -> String {
        if !path.starts_with("Data/") && !path.starts_with("Data\\") {
            format!("Data/{}", path)
        } else {
            path.to_string()
        }
    }
}
```

### 3. Tauri for Desktop UI (Instead of WinForms)

**Rationale**: WinForms is Windows-only. Tauri provides:
- Cross-platform (Windows, macOS, Linux)
- Modern web-based UI (React)
- Rust backend integration
- Small bundle size

### 4. WASM Scripting (Instead of C# Scripts)

**Rationale**: C# scripting requires Mono/CoreCLR. WASM provides:
- Cross-platform execution
- Security sandboxing
- Support for multiple source languages (Rust, C, AssemblyScript)
- Deterministic execution

### 5. Symlink Fallback Strategy

**Decision**: Try symlinks first, fall back to hardlinks, then copies.

```rust
pub enum LinkStrategy {
    Symlink,     // Preferred (Windows Developer Mode, Unix)
    Hardlink,    // Fallback (same filesystem required)
    Copy,        // Last resort (always works, uses disk space)
}

pub fn create_link(source: &Path, target: &Path) -> Result<LinkStrategy, VfsError> {
    // Try symlink first
    if try_symlink(source, target).is_ok() {
        return Ok(LinkStrategy::Symlink);
    }

    // Try hardlink if on same filesystem
    if same_filesystem(source, target) {
        if try_hardlink(source, target).is_ok() {
            return Ok(LinkStrategy::Hardlink);
        }
    }

    // Fall back to copy
    fs::copy(source, target)?;
    Ok(LinkStrategy::Copy)
}
```

## Critical Source References

When implementing features, reference these C# files for behavior details:

### Core Interfaces

| Rust Trait | C# Interface | File Path |
|------------|--------------|-----------|
| `GameModeDescriptor` | `IGameModeDescriptor` | `ModManager.Interface/Games/IGameModeDescriptor.cs` |
| `GameMode` | `IGameMode` | `ModManager.Interface/Games/IGameMode.cs` |
| `Mod` | `IMod` | `ModManager.Interface/Mods/IMod.cs` |
| `ModInfo` | `IModInfo` | `ModManager.Interface/Mods/IModInfo.cs` |
| `ModFormat` | `IModFormat` | `ModManager.Interface/Mods/IModFormat.cs` |
| `InstallLog` | `IInstallLog` | `ModManager.Interface/ModManagement/InstallationLog/IInstallLog.cs` |
| `VirtualModActivator` | `IVirtualModActivator` | `ModManager.Interface/ModManagement/VirtualModActivator/IVirtualModActivator.cs` |

### Implementations

| Feature | C# Implementation | Notes |
|---------|-------------------|-------|
| Install Log | `NexusClient/ModManagement/InstallationLog/InstallLog.cs` | File ownership stack, transaction support |
| Virtual Activator | `NexusClient/ModManagement/VirtualModActivator/VirtualModActivator.cs` | Symlink/hardlink creation, multi-HD support |
| Plugin Manager | `NexusClient/PluginManagement/PluginManager.cs` | Plugin ordering, activation |
| Gamebryo Base | `Game Modes/GamebryoBase/GamebryoGameModeBase.cs` | Bethesda game shared functionality |
| XmlScript | `Script Types/XmlScript/XmlScriptType.cs` | Script parsing and execution |
| Program Entry | `NexusClient/Program.cs` | Application bootstrap |

### File Formats

| Format | Example Location | Notes |
|--------|------------------|-------|
| InstallLog.xml | Stored in game mode folder | Version 0.5.0.0 |
| VirtualModConfig.xml | Stored in VirtualInstall folder | Version 0.3.0.0 |
| plugins.txt | Game's local AppData | Bethesda plugin list |
| XmlScript 5.0 | Inside mod archives at `fomod/ModuleConfig.xml` | Schema: XmlScript5.0.xsd |

## Implementing New Game Modes

### Step 1: Create Game Mode File

```rust
// nmm-games/src/my_game.rs

use nmm_core::{GameMode, GameModeDescriptor, GameTheme};
use std::path::{Path, PathBuf};

pub struct MyGameGameMode {
    installation_path: PathBuf,
}

impl MyGameGameMode {
    pub fn new(installation_path: &Path) -> Result<Self, GameModeError> {
        // Verify game installation
        let exe_path = installation_path.join("MyGame.exe");
        if !exe_path.exists() {
            return Err(GameModeError::GameNotFound);
        }

        Ok(Self {
            installation_path: installation_path.to_path_buf(),
        })
    }
}

impl GameModeDescriptor for MyGameGameMode {
    fn mode_id(&self) -> &str {
        "MyGame"
    }

    fn name(&self) -> &str {
        "My Game"
    }

    fn game_executables(&self) -> &[&str] {
        &["MyGame.exe"]
    }

    fn plugin_extensions(&self) -> &[&str] {
        &[]  // No plugin system
    }

    fn critical_plugins(&self) -> &[&str] {
        &[]
    }

    fn official_plugins(&self) -> &[&str] {
        &[]
    }

    fn stop_folders(&self) -> &[&str] {
        &["Data", "Mods"]  // Folders that indicate mod root
    }

    fn theme(&self) -> GameTheme {
        GameTheme {
            primary_color: "#4a90d9".into(),
            icon_path: None,
        }
    }
}

impl GameMode for MyGameGameMode {
    fn installation_path(&self) -> &Path {
        &self.installation_path
    }

    fn plugin_directory(&self) -> PathBuf {
        self.installation_path.join("Data")
    }

    fn uses_plugins(&self) -> bool {
        false
    }

    fn plugin_factory(&self) -> Option<Box<dyn PluginFactory>> {
        None
    }

    fn plugin_order_validator(&self) -> Option<Box<dyn PluginOrderValidator>> {
        None
    }

    fn load_order_manager(&self) -> Option<Box<dyn LoadOrderManager>> {
        None
    }
}
```

### Step 2: Add Game Detection

```rust
// nmm-games/src/detection.rs

pub fn detect_my_game() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // Check registry
        if let Some(path) = check_registry_key(
            r"SOFTWARE\MyCompany\MyGame",
            "InstallPath"
        ) {
            if path.join("MyGame.exe").exists() {
                return Some(path);
            }
        }
    }

    // Check Steam
    if let Some(steam) = detect_steam_library() {
        let steam_path = steam.join("MyGame");
        if steam_path.join("MyGame.exe").exists() {
            return Some(steam_path);
        }
    }

    None
}
```

### Step 3: Register in Module

```rust
// nmm-games/src/lib.rs

mod my_game;
pub use my_game::MyGameGameMode;

// Add to game registry
pub fn all_game_modes() -> Vec<Box<dyn GameModeDescriptor>> {
    vec![
        // ... existing games
        Box::new(MyGameDescriptor),
    ]
}
```

### For Bethesda/Gamebryo Games

Use the Gamebryo helpers:

```rust
// nmm-games/src/skyrim/skyrim_se.rs

use nmm_game_modes::gamebryo::{self, GamebryoPluginFactory, GamebryoOrderValidator};

impl GameMode for SkyrimSEGameMode {
    fn uses_plugins(&self) -> bool {
        true
    }

    fn plugin_factory(&self) -> Option<Box<dyn PluginFactory>> {
        Some(Box::new(GamebryoPluginFactory::new(
            self.plugin_directory(),
            vec!["esp", "esm", "esl"],
        )))
    }

    fn plugin_order_validator(&self) -> Option<Box<dyn PluginOrderValidator>> {
        Some(Box::new(GamebryoOrderValidator::new(
            self.critical_plugins(),
        )))
    }

    fn hardlink_required_extensions(&self) -> HashSet<&str> {
        gamebryo::hardlink_required_extensions()
    }

    fn adjust_mod_path(&self, format_id: &str, path: &str, ignore_if_present: bool) -> String {
        if format_id == "FOMod" || format_id == "OMod" {
            gamebryo::adjust_fomod_path(path, ignore_if_present)
        } else {
            path.to_string()
        }
    }
}
```

## Cross-Platform Development Notes

### Windows-Specific Code

```rust
#[cfg(target_os = "windows")]
mod windows {
    use windows::Win32::Storage::FileSystem::*;

    pub fn create_symlink(source: &Path, target: &Path, is_dir: bool) -> io::Result<()> {
        let flags = if is_dir {
            SYMBOLIC_LINK_FLAG_DIRECTORY
        } else {
            SYMBOLIC_LINK_FLAGS(0)
        };

        // Convert paths to wide strings
        let source_wide: Vec<u16> = source.as_os_str().encode_wide().chain(once(0)).collect();
        let target_wide: Vec<u16> = target.as_os_str().encode_wide().chain(once(0)).collect();

        unsafe {
            CreateSymbolicLinkW(
                PCWSTR::from_raw(target_wide.as_ptr()),
                PCWSTR::from_raw(source_wide.as_ptr()),
                flags,
            )
        }
        .ok()
    }
}
```

### macOS-Specific Code

```rust
#[cfg(target_os = "macos")]
mod macos {
    use std::os::unix::fs::symlink;

    pub fn create_symlink(source: &Path, target: &Path, _is_dir: bool) -> io::Result<()> {
        // Unix symlinks work the same for files and directories
        symlink(source, target)
    }

    pub fn get_steam_library() -> Option<PathBuf> {
        dirs::home_dir()
            .map(|h| h.join("Library/Application Support/Steam/steamapps/common"))
            .filter(|p| p.exists())
    }
}
```

### Platform Abstraction

```rust
// nmm-vfs/src/link.rs

pub fn create_symlink(source: &Path, target: &Path, is_dir: bool) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        windows::create_symlink(source, target, is_dir)
    }

    #[cfg(target_os = "macos")]
    {
        macos::create_symlink(source, target, is_dir)
    }

    #[cfg(target_os = "linux")]
    {
        std::os::unix::fs::symlink(source, target)
    }
}
```

## Testing Patterns

### Unit Testing with Mocks

```rust
use mockall::automock;

#[automock]
pub trait InstallLog {
    fn add_data_file(&mut self, mod_key: &str, path: &Path) -> Result<(), InstallLogError>;
    fn get_current_file_owner(&self, path: &Path) -> Option<&ModInfo>;
}

#[test]
fn test_installation_logs_files() {
    let mut mock_log = MockInstallLog::new();

    mock_log
        .expect_add_data_file()
        .withf(|key, path| key == "mod_001" && path == Path::new("Data/test.dds"))
        .times(1)
        .returning(|_, _| Ok(()));

    install_file(&mut mock_log, "mod_001", Path::new("Data/test.dds")).unwrap();
}
```

### Integration Testing with Temp Directories

```rust
use tempfile::TempDir;
use assert_fs::prelude::*;

#[tokio::test]
async fn test_mod_installation_creates_links() {
    let temp = TempDir::new().unwrap();
    let game_dir = temp.path().join("game");
    let mods_dir = temp.path().join("mods");
    let virtual_dir = temp.path().join("virtual");

    // Create mock game structure
    fs::create_dir_all(game_dir.join("Data")).unwrap();

    // Create mock mod
    let mod_archive = mods_dir.join("TestMod.7z");
    create_test_archive(&mod_archive, &["Data/textures/test.dds"]);

    // Run installation
    let game_mode = MockGameMode::new(&game_dir);
    let mut vfs = VirtualModActivator::new(&virtual_dir, &game_mode).unwrap();
    let test_mod = load_mod(&mod_archive, &game_mode).unwrap();

    vfs.enable_mod(&test_mod).await.unwrap();

    // Verify link created
    assert!(game_dir.join("Data/textures/test.dds").exists());
}
```

## Common Issues & Solutions

### Issue: Symlinks fail on Windows

**Cause**: Developer Mode not enabled or insufficient privileges.

**Solution**:
```rust
fn check_symlink_capability() -> bool {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("target");
    let link = temp.path().join("link");

    fs::write(&target, "test").unwrap();
    symlink_file(&target, &link).is_ok()
}
```

### Issue: Long paths on Windows

**Cause**: Windows 260 character path limit.

**Solution**:
```rust
#[cfg(target_os = "windows")]
fn normalize_long_path(path: &Path) -> PathBuf {
    // Use \\?\ prefix for long paths
    let path_str = path.to_string_lossy();
    if path_str.len() > 260 && !path_str.starts_with(r"\\?\") {
        PathBuf::from(format!(r"\\?\{}", path_str))
    } else {
        path.to_path_buf()
    }
}
```

### Issue: Case sensitivity differences

**Cause**: Windows is case-insensitive, Unix is case-sensitive.

**Solution**:
```rust
// Always use case-insensitive comparison for file tracking
use unicase::UniCase;

type FileOwnerMap = HashMap<UniCase<String>, Vec<ModKey>>;
```

### Issue: File locked by game

**Cause**: Game is running with file handles open.

**Solution**:
```rust
fn wait_for_file_unlock(path: &Path, timeout: Duration) -> Result<(), FileError> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        match fs::OpenOptions::new().write(true).open(path) {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e.into()),
        }
    }
    Err(FileError::Locked(path.to_path_buf()))
}
```

## Debugging Tips

### Enable Debug Logging

```bash
# Set log level
RUST_LOG=nmm_vfs=debug,nmm_install_log=trace cargo run

# Log to file
RUST_LOG=debug cargo run 2>&1 | tee debug.log
```

### Inspect SQLite Database

```bash
# Open install log database
sqlite3 ~/.local/share/nmm/SkyrimSE/InstallLog.db

# View active mods
SELECT * FROM mods;

# View file ownership
SELECT path, mod_key, install_order FROM file_owners ORDER BY path;
```

### Test with Mock Game

Create a minimal mock game installation for testing:

```bash
mkdir -p /tmp/mock_skyrim/Data
touch /tmp/mock_skyrim/SkyrimSE.exe
touch /tmp/mock_skyrim/Data/Skyrim.esm

cargo test -p nmm-games -- --test-threads=1
```
