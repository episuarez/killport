//! killport-core: pure logic, no UI dependency. Tauri/CLI mount on top.

pub(crate) mod appinfo;
pub mod autostart;
pub(crate) mod classify;
pub mod config;
pub(crate) mod docker;
pub(crate) mod framework;
pub(crate) mod guard;
pub(crate) mod kill;
pub mod ports;
pub(crate) mod process;
pub(crate) mod project;
pub(crate) mod restart;
pub(crate) mod scan;
pub(crate) mod service;

pub use config::Config;
pub use kill::{kill, kill_tree, KillError, KillMode};
pub use restart::restart;
pub use scan::{scan, scan_fast, PortProcess};
