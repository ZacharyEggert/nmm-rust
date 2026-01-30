## Project Structure

```
nexus-mod-manager-rs/
├── Cargo.toml                        # Workspace root
├── crates/
│   ├── nmm-core/                     # Core domain types and traits
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mod_info.rs           # IModInfo equivalent
│   │   │   ├── game_mode.rs          # IGameMode/IGameModeDescriptor
│   │   │   ├── mod_format.rs         # IModFormat equivalent
│   │   │   ├── error.rs              # Error types
│   │   │   └── prelude.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-vfs/                      # Virtual file system
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── activator.rs          # IVirtualModActivator equivalent
│   │   │   ├── link.rs               # Symlink/hardlink operations
│   │   │   ├── priority.rs           # Priority resolution
│   │   │   └── config.rs             # VirtualModConfig.xml handling
│   │   └── Cargo.toml
│   │
│   ├── nmm-install-log/              # Installation logging
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── log.rs                # IInstallLog equivalent
│   │   │   ├── transaction.rs        # Transaction support
│   │   │   ├── file_ownership.rs     # File ownership tracking
│   │   │   ├── ini_edits.rs          # INI edit tracking
│   │   │   └── migrations.rs         # Log format migrations
│   │   └── Cargo.toml
│   │
│   ├── nmm-archive/                  # Archive/mod format handling
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── formats/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── fomod.rs          # FOMod format
│   │   │   │   ├── omod.rs           # OMod format
│   │   │   │   └── archive.rs        # Generic archive (7z, zip, rar)
│   │   │   ├── extractor.rs
│   │   │   └── compressor.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-scripting/                # Script execution
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── xml_script/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── parser.rs         # XML script parser
│   │   │   │   ├── executor.rs       # Script executor
│   │   │   │   ├── conditions.rs     # Condition evaluation
│   │   │   │   └── ui.rs             # UI abstraction for options
│   │   │   ├── mod_script.rs         # Legacy ModScript interpreter
│   │   │   └── wasm/                 # WASM scripting (replaces C#)
│   │   │       ├── mod.rs
│   │   │       └── runtime.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-plugin-manager/           # Game plugin management
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── plugin.rs             # Plugin representation
│   │   │   ├── order.rs              # Load order management
│   │   │   ├── validator.rs          # Order validation
│   │   │   ├── discovery.rs          # Plugin discovery
│   │   │   ├── sorter.rs             # LOOT-based sorting
│   │   │   └── formats/
│   │   │       ├── mod.rs
│   │   │       ├── bethesda.rs       # .esp/.esm/.esl handling
│   │   │       └── plugins_txt.rs    # plugins.txt parser
│   │   └── Cargo.toml
│   │
│   ├── nmm-game-modes/               # Game mode plugin system
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── registry.rs           # Game mode registry
│   │   │   ├── detection.rs          # Game installation detection
│   │   │   └── gamebryo/             # Gamebryo engine base
│   │   │       ├── mod.rs
│   │   │       ├── base.rs           # GamebryoGameModeBase equivalent
│   │   │       ├── ini.rs            # INI file handling
│   │   │       └── settings.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-games/                    # Built-in game implementations
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── skyrim/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── skyrim_le.rs
│   │   │   │   ├── skyrim_se.rs
│   │   │   │   ├── skyrim_vr.rs
│   │   │   │   └── skyrim_gog.rs
│   │   │   ├── fallout/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── fallout3.rs
│   │   │   │   ├── fallout_nv.rs
│   │   │   │   ├── fallout4.rs
│   │   │   │   └── fallout4_vr.rs
│   │   │   ├── oblivion/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── oblivion.rs
│   │   │   │   └── oblivion_remastered.rs
│   │   │   ├── starfield.rs
│   │   │   ├── morrowind.rs
│   │   │   ├── witcher3.rs
│   │   │   ├── cyberpunk2077.rs
│   │   │   ├── baldurs_gate3.rs
│   │   │   └── [other games...]
│   │   └── Cargo.toml
│   │
│   ├── nmm-nexus-api/                # Nexus Mods API client
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs
│   │   │   ├── auth.rs               # API key / SSO
│   │   │   ├── endpoints/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── mods.rs
│   │   │   │   ├── files.rs
│   │   │   │   └── user.rs
│   │   │   └── models.rs
│   │   └── Cargo.toml
│   │
│   ├── nmm-profiles/                 # Profile management
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── profile.rs
│   │   │   ├── export.rs
│   │   │   └── import.rs
│   │   └── Cargo.toml
│   │
│   └── nmm-cli/                      # CLI application
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/
│       │   │   ├── mod.rs
│       │   │   ├── install.rs
│       │   │   ├── activate.rs
│       │   │   ├── profile.rs
│       │   │   └── list.rs
│       │   └── config.rs
│       └── Cargo.toml
│
├── apps/
│   └── nmm-desktop/                  # Tauri desktop application
│       ├── src-tauri/
│       │   ├── src/
│       │   │   ├── main.rs
│       │   │   ├── commands.rs       # Tauri commands
│       │   │   └── state.rs          # Application state
│       │   ├── Cargo.toml
│       │   └── tauri.conf.json
│       ├── src/                      # React frontend
│       │   ├── App.tsx
│       │   ├── components/
│       │   │   ├── ModList.tsx
│       │   │   ├── PluginOrder.tsx
│       │   │   ├── ProfileSelector.tsx
│       │   │   └── InstallWizard.tsx
│       │   └── hooks/
│       ├── package.json
│       └── vite.config.ts
│
└── tests/
    └── integration/
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            ├── install_tests.rs
            ├── profile_tests.rs
            └── fixtures/
```
