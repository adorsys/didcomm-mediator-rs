use sha2::{Digest, Sha256};

pub fn sha256_hash(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);

    let hash = hasher.finalize();

    hash.as_slice()[..32].try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let bytes = "Hello, world!".as_bytes();
        let hash = sha256_hash(bytes);

        println!("{:?}", hash);

        let expected: [u8; 32] = [
            0x31, 0x5f, 0x5b, 0xdb, 0x76, 0xd0, 0x78, 0xc4, 0x3b, 0x8a, 0xc0, 0x06, 0x4e, 0x4a, 0x01, 0x64, 0x61, 0x2b, 0x1f, 0xce, 0x77, 0xc8, 0x69,
            0x34, 0x5b, 0xfc, 0x94, 0xc7, 0x58, 0x94, 0xed, 0xd3,
        ];

        assert_eq!(expected, hash);
    }
}
