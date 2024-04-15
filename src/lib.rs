#![doc = include_str!("../README.md")]

#[cfg(feature = "bevy")]
pub mod asset_state;
pub mod identifier;
pub mod manifest;
#[cfg(feature = "bevy")]
pub mod plugin;
