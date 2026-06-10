//! killport-core: pure logic, no UI dependency. Tauri/CLI mount on top.

pub mod appinfo;
pub mod autostart;
pub mod classify;
pub mod config;
pub mod docker;
pub mod framework;
pub mod guard;
pub mod kill;
pub mod ports;
pub mod process;
pub mod project;
pub mod restart;
pub mod scan;
pub mod service;

pub use config::Config;
pub use kill::{kill, kill_tree, KillError, KillMode};
pub use restart::restart;
pub use scan::{scan, scan_fast, PortProcess};
