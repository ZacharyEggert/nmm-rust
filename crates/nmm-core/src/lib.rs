//! Core domain types and traits for Nexus Mod Manager.
//!
//! This crate defines the fundamental abstractions used throughout NMM:
//!
//! - [`GameModeDescriptor`] / [`GameMode`] - Game mode abstraction
//! - [`ModInfo`] / [`Mod`] - Mod metadata and archive access
//! - [`ModFormat`] - Archive format handling
//!
//! # Example
//!
//! ```rust
//! use nmm_core::{GameModeDescriptor, ModInfo};
//!
//! // Implement GameModeDescriptor for a new game
//! struct MyGameDescriptor;
//!
//! impl GameModeDescriptor for MyGameDescriptor {
//!     fn mode_id(&self) -> &str { "MyGame" }
//!     fn name(&self) -> &str { "My Game" }
//!     // ... other required methods
//! }
//! ```

mod error;
mod game_mode;
mod mod_format;
mod mod_info;

pub use error::*;
pub use game_mode::*;
pub use mod_format::*;
pub use mod_info::*;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{
        FormatConfidence, GameMode, GameModeDescriptor, GameTheme, Mod, ModError, ModFormat,
        ModFormatError, ModInfo, ScriptType,
    };
}
