mod didgen;
pub mod plugin;
mod util;
mod web;

// Re-exports
pub use didgen::{didgen, validate_diddoc, Error};
