## Cross-Platform Considerations

### Windows

```rust
#[cfg(target_os = "windows")]
mod windows {
    use windows::Win32::Storage::FileSystem::{
        CreateSymbolicLinkW, CreateHardLinkW, SYMBOLIC_LINK_FLAG_DIRECTORY,
    };

    pub fn create_symlink(source: &Path, target: &Path) -> Result<(), VfsError> {
        // Check if Developer Mode is enabled
        if !is_developer_mode_enabled() {
            return Err(VfsError::PermissionDenied(
                "Developer Mode required for symlinks".into()
            ));
        }
        // Create symlink using Windows API
        unsafe {
            CreateSymbolicLinkW(/* ... */);
        }
    }

    pub fn is_developer_mode_enabled() -> bool {
        // Check registry key
        let key = winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock")
            .ok();
        key.and_then(|k| k.get_value::<u32, _>("AllowDevelopmentWithoutDevLicense").ok())
            .map(|v| v == 1)
            .unwrap_or(false)
    }
}
```

### macOS

```rust
#[cfg(target_os = "macos")]
mod macos {
    use std::os::unix::fs::symlink;

    pub fn create_symlink(source: &Path, target: &Path) -> Result<(), VfsError> {
        // Standard POSIX symlinks work without special permissions
        symlink(source, target).map_err(VfsError::from)
    }

    pub fn detect_steam_library() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let steam_path = home.join("Library/Application Support/Steam/steamapps/common");
        if steam_path.exists() {
            Some(steam_path)
        } else {
            None
        }
    }
}
```

### Game Detection Strategy

```rust
pub fn detect_game_installation(game_id: &str) -> Option<PathBuf> {
    // Platform-specific detection
    #[cfg(target_os = "windows")]
    {
        // 1. Check Windows Registry (most reliable)
        if let Some(path) = detect_from_registry(game_id) {
            return Some(path);
        }
    }

    // 2. Check Steam library paths (cross-platform)
    if let Some(path) = detect_from_steam(game_id) {
        return Some(path);
    }

    // 3. Check GOG Galaxy paths
    if let Some(path) = detect_from_gog(game_id) {
        return Some(path);
    }

    // 4. Check common installation directories
    detect_from_common_paths(game_id)
}
```
