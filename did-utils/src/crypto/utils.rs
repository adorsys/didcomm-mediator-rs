//! Utility functions for cryptographic operations.

use super::traits::BYTES_LENGTH_32;

/// Generates a seed for the `ed25519` key pair.
///
/// If the initial seed is empty or invalid, generates a new seed.
///
/// # Arguments
///
/// * `initial_seed` - The initial seed to use, or empty if none.
///
/// # Returns
///
/// A `Vec` of bytes of length `BYTES_LENGTH_32`, containing the generated seed.
///
/// # Errors
///
/// Returns an error if the initial seed is invalid.
pub fn generate_seed(initial_seed: &[u8]) -> Result<[u8; BYTES_LENGTH_32], &str> {
    let mut seed = [0u8; BYTES_LENGTH_32];
    if initial_seed.is_empty() || initial_seed.len() != BYTES_LENGTH_32 {
        getrandom::getrandom(&mut seed).expect("couldn't generate random seed");
    } else {
        seed = match initial_seed.try_into() {
            Ok(x) => x,
            Err(_) => return Err("invalid seed size"),
        };
    }
    Ok(seed)
}

/// Clones the content of the slice into a new array.
///
/// It is important to clone the data, as we don't want key material to be hazardously modified.
///
/// # Arguments
///
/// * `slice` - The slice to clone.
///
/// # Returns
///
/// A new array containing the cloned data.
///
/// # Panics
///
/// Panics if the length of the slice is not equal to `BYTES_LENGTH_32`.
pub fn clone_slice_to_array(slice: &[u8; BYTES_LENGTH_32]) -> [u8; BYTES_LENGTH_32] {
    
    let mut array = [0u8; BYTES_LENGTH_32];

    array.clone_from_slice(slice);
    array
}