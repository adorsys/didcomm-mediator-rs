/// The length of an ed25519 `PublicKey`, in bytes.
pub const BYTES_LENGTH_32: usize = 32;

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

// Copy the data from a slice into an array.
pub fn copy_slice_to_array(slice: &[u8]) -> Option<[u8; BYTES_LENGTH_32]> {
    if slice.len() != BYTES_LENGTH_32 {
        // Return None if the slice length is not as expected
        return None;
    }

    // Create a new array of the expected length
    let mut array = [0u8; BYTES_LENGTH_32];

    // Copy the data from the slice into the array
    array.copy_from_slice(slice);

    // Return the array as an Option
    Some(array)
}
