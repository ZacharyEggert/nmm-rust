#[cfg(test)]
mod core_tests;

use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A temporary mock game directory that persists for the lifetime of the test.
///
/// Creates a minimal game installation layout:
///
/// ```text
/// <root>/
/// ├── MockGame.exe
/// └── Data/
/// ```
pub struct MockGameDir {
    _temp: TempDir,
    root: PathBuf,
}

impl MockGameDir {
    /// Create a new mock game directory with a dummy executable and Data folder.
    pub fn new() -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().to_path_buf();

        std::fs::write(root.join("MockGame.exe"), b"")?;
        std::fs::create_dir_all(root.join("Data"))?;

        Ok(Self { _temp: temp, root })
    }

    /// Path to the mock game root directory.
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Path to the mock game's Data subdirectory.
    pub fn data_dir(&self) -> PathBuf {
        self.root.join("Data")
    }
}

/// Return the path to the `fixtures/` directory bundled with this test crate.
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Probe whether the current platform supports creating symlinks without
/// elevated privileges.  On Windows this requires Developer Mode; on Unix it
/// is unconditionally available.
pub fn symlinks_supported() -> bool {
    let temp = match TempDir::new() {
        Ok(t) => t,
        Err(_) => return false,
    };

    let target = temp.path().join("target");
    let link = temp.path().join("link");

    if std::fs::write(&target, b"probe").is_err() {
        return false;
    }

    #[cfg(target_os = "windows")]
    {
        std::os::windows::fs::symlink_file(&target, &link).is_ok()
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(&target, &link).is_ok()
    }
}
