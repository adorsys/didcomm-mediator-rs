mod util;

pub mod didgen;
pub mod persistence;
pub mod plugin;
pub mod web;

// Re-exports
pub use didgen::{didgen, validate_diddoc, Error};
