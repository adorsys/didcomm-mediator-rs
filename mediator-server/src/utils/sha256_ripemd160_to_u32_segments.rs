use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use ripemd::Ripemd160;

// Thansforms a string into a path of HD wallet addresses
// See BIP32 for more information
pub fn id_to_hd_path(input: &str) -> String {
    let address = sha256_ripemd160(input);
    let segments = convert_to_u32_vector(&address);
    vect_u32_to_u32_str(&segments)
}

#[test]
fn test_id_to_hd_path() {
    let input = "my_identifier";
    let expected_result = "14715472.12273097.7829168.5501977.11059137.4002663.27134";

    let result = id_to_hd_path(input);

    assert_eq!(result, expected_result);
}

// Computes SHA256 hash of input, then RIPEMD160 hash of SHA256 hash
fn sha256_ripemd160(input: &str) -> [u8; 20] {
    let input_bytes = input.as_bytes();

    // Compute SHA256 hash
    let sha256_hash = Sha256::digest(input_bytes);

    // Compute RIPEMD160 hash
    let mut ripemd160 = Ripemd160::new();
    ripemd160.update(&sha256_hash);
    let ripemd160_hash = ripemd160.finalize();

    // Convert hash to byte array
    let mut result = [0u8; 20];
    result.copy_from_slice(&ripemd160_hash[..]);
    result
}

#[test]
fn test_sha256_ripemd160() {
    let input = "Hello, world!";
    let expected_result = [0x8d, 0x15, 0x9f, 0x1c, 0x4f, 0x99, 0xd8, 0xed, 0x85, 0x8f, 0x78, 0x32, 0x31, 0x0d, 0xb3, 0x1c, 0xb9, 0x1e, 0x7, 0x45];

    let result = sha256_ripemd160(input);

    assert_eq!(result, expected_result);
}

// Produces a tring from the segments of a hash
fn vect_u32_to_u32_str(vect: &Vec<u32>) -> String {
    let mut result = String::new();
    for i in 0..vect.len() {
        result.push_str(&vect[i].to_string());
        if i < vect.len() - 1 {
            result.push('.');
        }
    }
    result
}

#[test]
fn test_vect_u32_to_u32_str() {
    let vect: Vec<u32> = vec![123, 456, 789];
    let expected_result = "123.456.789".to_string();

    let result = vect_u32_to_u32_str(&vect);

    assert_eq!(result, expected_result);
}

fn convert_to_u32_vector(hash: &[u8; 20]) -> Vec<u32> {
    let n = 4;
    let (chunks, rest) = split_into_chunks(&hash);
    // convert each chunk to u32 and return a vector of u32
    let mut result = Vec::with_capacity(n);
    for i in 0..chunks.len() {
        let chunk = left_pad_array_3_to_4(&chunks[i]);
        let value = u32::from_be_bytes(chunk);
        result.push(value);
    }
    // The last entry is a 16-bit value
    let chunk = left_pad_array_2_to_4(&rest);
    let value = u32::from_be_bytes(chunk);
    result.push(value);

    result
}

#[test]
fn test_convert_to_u32_vector() {
    let hash: [u8; 20] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
        0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14,
    ];

    let expected_result: Vec<u32> = vec![
        0x10203, 0x40506, 0x70809, 0xA0B0C, 0xD0E0F, 0x101112, 0x1314
    ];

    let result = convert_to_u32_vector(&hash);

    assert_eq!(result, expected_result);
}

fn split_into_chunks(hash: &[u8; 20]) -> (Vec<[u8; 3]>, [u8; 2]) {
    let mut chunks = Vec::with_capacity(6);
    
    for i in 0..6 {
        let start = i * 3;
        let chunk = &hash[start..start+3];
        let mut chunk_array = [0; 3];
        chunk_array.copy_from_slice(chunk);
        chunks.push(chunk_array);
    }
    
    let last_chunk = [hash[18], hash[19]];
    
    (chunks, last_chunk)
}

#[test]
fn test_split_into_chunks() {
    let hash: [u8; 20] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
        0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14,
    ];

    let (chunks, last_chunk) = split_into_chunks(&hash);

    // Assert the number of chunks
    assert_eq!(chunks.len(), 6);

    // Assert each chunk
    assert_eq!(chunks[0], [0x01, 0x02, 0x03]);
    assert_eq!(chunks[1], [0x04, 0x05, 0x06]);
    assert_eq!(chunks[2], [0x07, 0x08, 0x09]);
    assert_eq!(chunks[3], [0x0A, 0x0B, 0x0C]);
    assert_eq!(chunks[4], [0x0D, 0x0E, 0x0F]);
    assert_eq!(chunks[5], [0x10, 0x11, 0x12]);
    // Assert the last chunk
    assert_eq!(last_chunk, [0x13, 0x14]);
}

fn left_pad_array_3_to_4(array: &[u8; 3]) -> [u8; 4] {
    let mut padded = [0; 4];
    padded[1..].copy_from_slice(array);
    padded
}

#[test]
fn test_left_pad_array_3_to_4() {
    let input: [u8; 3] = [1, 2, 3];
    let expected: [u8; 4] = [0, 1, 2, 3];

    let result = left_pad_array_3_to_4(&input);

    assert_eq!(result, expected);
}

fn left_pad_array_2_to_4(array: &[u8; 2]) -> [u8; 4] {
    let mut padded = [0; 4];
    padded[2..].copy_from_slice(array);
    padded
}

#[test]
fn test_left_pad_array_2_to_4() {
    let array: [u8; 2] = [42, 127];
    let expected_result: [u8; 4] = [0, 0, 42, 127];
    let padded = left_pad_array_2_to_4(&array);
    assert_eq!(padded, expected_result);
}
