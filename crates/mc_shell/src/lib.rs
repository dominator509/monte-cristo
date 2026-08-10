//! Monte Cristo presentation shell library.
//!
//! All presentation logic lives in this crate's library so integration tests
//! can access it. The binary (main.rs) is a thin entry point.

pub mod a11y;
pub mod app;
pub mod audio;
pub mod config;
pub mod fsroot;
pub mod input;
pub mod obs;
#[cfg(feature = "debug-overlay")]
pub mod overlay;
pub mod persistence;
pub mod render;
pub mod ui;
