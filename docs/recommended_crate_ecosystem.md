## Recommended Crate Ecosystem

### Core Dependencies

| Purpose | Crate | Notes |
|---------|-------|-------|
| Async Runtime | `tokio` | Full features for async operations |
| Error Handling | `thiserror`, `anyhow` | thiserror for library errors, anyhow for application |
| Serialization | `serde`, `serde_json` | JSON config files |
| XML | `quick-xml`, `serde-xml-rs` | VirtualModConfig.xml, InstallLog.xml, XmlScript |
| Logging | `tracing`, `tracing-subscriber` | Structured logging with async support |
| CLI | `clap` | Command-line argument parsing |

### Archive Handling

| Purpose | Crate | Notes |
|---------|-------|-------|
| ZIP | `zip` | Standard zip archives |
| 7z | `sevenz-rust` | 7-Zip format (most common for mods) |
| RAR | `unrar` | RAR archives (requires libunrar) |
| Compression | `flate2`, `lz4`, `zstd` | Various compression algorithms |

### Database & Storage

| Purpose | Crate | Notes |
|---------|-------|-------|
| SQLite | `rusqlite` | Install log storage (upgrade from XML) |
| Migrations | `refinery` | Database schema migrations |
| KV Store | `sled` | Fast embedded database for caching |

### File System

| Purpose | Crate | Notes |
|---------|-------|-------|
| Path Utils | `pathdiff`, `normpath` | Path manipulation |
| File Watching | `notify` | Watch for file changes |
| Temp Files | `tempfile` | Temporary file handling |
| Directory Walking | `walkdir`, `ignore` | Recursive directory traversal |

### Platform-Specific

| Purpose | Crate | Notes |
|---------|-------|-------|
| Windows APIs | `windows` | Official Microsoft Windows crate |
| macOS APIs | `core-foundation` | macOS framework bindings |
| Registry | `winreg` | Windows registry access |
| Symlinks | `symlink` | Cross-platform symlink creation |

### UI Framework

| Purpose | Crate | Notes |
|---------|-------|-------|
| Desktop UI | `tauri` | Cross-platform desktop with web frontend |
| Frontend | React + TypeScript | Modern, familiar web stack |
| Build | Vite | Fast frontend bundling |

### Scripting

| Purpose | Crate | Notes |
|---------|-------|-------|
| WASM Runtime | `wasmtime` | Run WASM scripts (replaces C# scripts) |
| Lua | `mlua` | Optional Lua scripting support |

### Networking

| Purpose | Crate | Notes |
|---------|-------|-------|
| HTTP Client | `reqwest` | Async HTTP client for Nexus API |
| URL Handling | `url` | URL parsing and manipulation |

### Testing

| Purpose | Crate | Notes |
|---------|-------|-------|
| Assertions | `assert_fs`, `predicates` | File system testing |
| Mocking | `mockall` | Mock trait implementations |
| Property Testing | `proptest` | Property-based testing |
