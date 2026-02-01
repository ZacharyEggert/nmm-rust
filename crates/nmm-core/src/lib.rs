//! Core domain types and traits for Nexus Mod Manager.
//!
//! This crate defines the fundamental abstractions used throughout NMM:
//!
//! - [`GameModeDescriptor`] / [`GameMode`] - Game mode abstraction
//! - [`ModInfo`] / [`Mod`] - Mod metadata and archive access
//! - [`ModFormat`] - Archive format handling
//! - [`InstallLog`] / [`IniEdit`] - Installation log tracking
//!
//! # Example
//!
//! ```rust
//! use nmm_core::{GameModeDescriptor, GameTheme};
//!
//! // Implement GameModeDescriptor for a new game
//! struct MyGameDescriptor;
//!
//! impl GameModeDescriptor for MyGameDescriptor {
//!     fn mode_id(&self) -> &str { "MyGame" }
//!     fn name(&self) -> &str { "My Game" }
//!     fn game_executables(&self) -> &[&str] { &["MyGame.exe"] }
//!     fn plugin_extensions(&self) -> &[&str] { &[] }
//!     fn critical_plugins(&self) -> &[&str] { &[] }
//!     fn official_plugins(&self) -> &[&str] { &[] }
//!     fn stop_folders(&self) -> &[&str] { &["Data"] }
//!     fn theme(&self) -> GameTheme {
//!         GameTheme { primary_color: "#4a90d9".into(), icon_path: None }
//!     }
//! }
//!
//! let _descriptor = MyGameDescriptor;
//! ```

mod error;
mod game_mode;
mod install_log;
mod mod_format;
mod mod_info;

pub use error::*;
pub use game_mode::*;
pub use install_log::*;
pub use mod_format::*;
pub use mod_info::*;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{
        FormatConfidence, GameMode, GameModeDescriptor, GameTheme, IniEdit, InstallLog,
        InstallLogError, Mod, ModError, ModFormat, ModFormatError, ModInfo, PluginError,
        ScriptType, ORIGINAL_VALUES_KEY,
    };
}
