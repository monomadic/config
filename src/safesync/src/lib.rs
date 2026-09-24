#[cfg(not(target_os = "macos"))]
compile_error!("safesync currently supports macOS only");

pub mod compare;
pub mod copy;
pub mod drive;
pub mod drives;
pub mod drives_ui;
pub mod engine;
pub mod filesystem;
pub mod lookup;
pub mod manifest;
pub mod plan;
pub mod scan;
pub mod ui;
