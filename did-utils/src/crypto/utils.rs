use super::traits::BYTES_LENGTH_32;

/// The length of an ed25519 `PublicKey`, in bytes.

// Generate 32 random bytes from the initial seed.
// Only generate random bytes if the initial seed is empty or invalid.
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

/// clone the content of the slice into a new array
/// It is important to clone the data, as we don't want key material to be hazardously modified.
pub fn clone_slice_to_array(slice: &[u8; BYTES_LENGTH_32]) -> [u8; BYTES_LENGTH_32] {
    // Create a new array of the expected length
    let mut array: [u8; BYTES_LENGTH_32] = [0; BYTES_LENGTH_32];
    // clone the data from the slice into the array
    array.clone_from_slice(slice);
    array
}
