use sha2::{Sha256, Digest};

/// Compute the SHA256 hash of a given input.
///
/// # Arguments
///
/// * `bytes` - The input to compute the hash from.
///
/// # Returns
///
/// The SHA256 hash as a byte array of length 32.
pub fn sha256_hash(bytes: &[u8]) -> [u8; 32] {
    // Create a new Sha256 hasher
    let mut hasher = Sha256::new();
    // Update the hasher with the input bytes
    hasher.update(bytes);
    // Read hash digest and consume hasher
    let hash = hasher.finalize();
    // Convert the hash to a byte array of length 32 and return it
    hash.as_slice()[..32].try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the `sha256_hash` function.
    #[test]
    fn test_sha256_hash() {
        // Define the input bytes and expected hash
        let bytes = "Hello, world!".as_bytes();
        let expected: [u8; 32] = [ 0x31, 0x5f, 0x5b, 0xdb, 0x76, 0xd0, 0x78, 0xc4, 0x3b, 0x8a, 0xc0, 0x06, 0x4e, 0x4a, 0x01, 0x64, 0x61, 0x2b, 0x1f, 0xce, 0x77, 0xc8, 0x69, 0x34, 0x5b, 0xfc, 0x94, 0xc7, 0x58, 0x94, 0xed, 0xd3, ];

        // Compute the hash of the input bytes
        let hash = sha256_hash(bytes);

        // Print the computed hash
        println!("{:?}", hash);

        // Assert that the computed hash matches the expected hash
        assert_eq!(expected, hash);
    }
}
