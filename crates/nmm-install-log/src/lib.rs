//! SQLite-backed installation log for Nexus Mod Manager.
//!
//! This crate manages the persistent record of which mods have installed
//! which files, INI edits, and game-specific values.  It uses SQLite via
//! rusqlite.
//!
//! # Schema
//!
//! The database contains these tables:
//!
//! * `schema_meta`  — version tracking and sequence counter
//! * `mods`         — registry of active mods
//! * `file_owners`  — ownership stack for installed data files
//! * `ini_edits`    — ownership stack for INI edits
//! * `gsv_edits`    — ownership stack for game-specific value edits
//!
//! See [`schema::apply`] for details on schema creation and migration.

pub mod error;
pub mod schema;
