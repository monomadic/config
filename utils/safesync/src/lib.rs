#[cfg(not(target_os = "macos"))]
compile_error!("safesync currently supports macOS only");

pub mod compare;
pub mod enrollment;
pub mod filesystem;
pub mod lookup;
pub mod maintenance;
pub mod manifest;
pub mod plan;
pub mod scan;
