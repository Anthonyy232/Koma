//! Koma on any host: its data source and what it draws. No platform code.

mod desktop;
pub mod ink;
mod source;

pub use source::Koma;

#[allow(clippy::all)]
pub mod shapes {
    include!(concat!(env!("OUT_DIR"), "/shapes.rs"));
}

#[allow(clippy::all)]
pub mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

/// One grant per line.
pub const GRANTS: &str = "device.global-shortcuts
device.login-item
device.screen-recording capture.purpose
device.window-management
fs.read app:/tmp
fs.write app:/tmp
fs.write screenshots:/";
