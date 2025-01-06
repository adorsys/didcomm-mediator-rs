use super::traits::BYTES_LENGTH_32;

// Generate a seed from an optional initial seed.
// If the initial seed is empty or invalid, generate a random seed.
pub(super) fn generate_seed(initial_seed: &[u8]) -> Result<[u8; BYTES_LENGTH_32], &str> {
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

// Clone a slice into an array.
pub(super) fn clone_slice_to_array(slice: &[u8; BYTES_LENGTH_32]) -> [u8; BYTES_LENGTH_32] {
    let mut array = [0u8; BYTES_LENGTH_32];

    array.clone_from_slice(slice);
    array
}
