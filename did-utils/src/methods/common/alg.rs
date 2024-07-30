use num_bigint::{BigInt, Sign};

use crate::{
    crypto::Error as CryptoError,
    key_jwk::{Bytes, Ec, EcCurves, Jwk, Key, Okp, OkpCurves, Parameters},
};

/// Supported cryptographic algorithms.
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(unused, clippy::upper_case_acronyms)]
pub enum Algorithm {
    Ed25519,
    X25519,
    Secp256k1,
    BLS12381,
    P256,
    P384,
    P521,
    RSA,
}

use Algorithm::*;

// See:
// - https://w3c-ccg.github.io/did-method-key/#signature-method-creation-algorithm
// - https://w3c-ccg.github.io/did-method-key/#encryption-method-creation-algorithm
impl Algorithm {
    /// Returns the multicodec prefix associated with the algorithm.
    ///
    /// # Returns
    ///
    /// A two-byte array representing the multicodec prefix.
    pub fn muticodec_prefix(&self) -> [u8; 2] {
        match self {
            Ed25519 => [0xed, 0x01],
            X25519 => [0xec, 0x01],
            Secp256k1 => [0xe7, 0x01],
            BLS12381 => [0xeb, 0x01],
            P256 => [0x80, 0x24],
            P384 => [0x81, 0x24],
            P521 => [0x82, 0x24],
            RSA => [0x85, 0x24],
        }
    }

    /// Creates an `Algorithm` enum variant from the given multicodec prefix.
    ///
    /// # Parameters
    ///
    /// - `prefix`: A two-byte array representing the multicodec prefix.
    ///
    /// # Returns
    ///
    /// An `Option` containing the corresponding `Algorithm` variant.
    pub fn from_muticodec_prefix(prefix: &[u8; 2]) -> Option<Self> {
        match prefix {
            [0xed, 0x01] => Some(Ed25519),
            [0xec, 0x01] => Some(X25519),
            [0xe7, 0x01] => Some(Secp256k1),
            [0xeb, 0x01] => Some(BLS12381),
            [0x80, 0x24] => Some(P256),
            [0x81, 0x24] => Some(P384),
            [0x82, 0x24] => Some(P521),
            [0x85, 0x24] => Some(RSA),
            _ => None,
        }
    }

    /// Returns the length of the public key for the algorithm, if known.
    ///
    /// # Returns
    ///
    /// An `Option` containing the length of the public key in bytes.
    pub fn public_key_length(&self) -> Option<usize> {
        match self {
            Ed25519 => Some(32),
            X25519 => Some(32),
            Secp256k1 => Some(33),
            BLS12381 => None,
            P256 => Some(33),
            P384 => Some(49),
            P521 => None,
            RSA => None,
        }
    }

    /// Builds a JSON Web Key from raw public key bytes.
    ///
    /// # Parameters
    ///
    /// - `raw_public_key_bytes`: The raw public key bytes.
    ///
    /// # Returns
    ///
    /// A `Result` containing the constructed `Jwk` or a `CryptoError`.
    pub fn build_jwk(&self, raw_public_key_bytes: &[u8]) -> Result<Jwk, CryptoError> {
        match self {
            Ed25519 => Ok(Jwk {
                key: Key::Okp(Okp {
                    crv: OkpCurves::Ed25519,
                    x: Bytes::from(raw_public_key_bytes.to_vec()),
                    d: None,
                }),
                prm: Parameters::default(),
            }),
            X25519 => Ok(Jwk {
                key: Key::Okp(Okp {
                    crv: OkpCurves::X25519,
                    x: Bytes::from(raw_public_key_bytes.to_vec()),
                    d: None,
                }),
                prm: Parameters::default(),
            }),
            Secp256k1 => {
                let uncompressed = self.uncompress_public_key(raw_public_key_bytes)?;
                Ok(Jwk {
                    key: Key::Ec(Ec {
                        crv: EcCurves::P256K,
                        d: None,
                        x: Bytes::from(uncompressed[1..33].to_vec()),
                        y: Bytes::from(uncompressed[33..].to_vec()),
                    }),
                    prm: Parameters::default(),
                })
            }
            P256 => {
                let uncompressed = self.uncompress_public_key(raw_public_key_bytes)?;
                Ok(Jwk {
                    key: Key::Ec(Ec {
                        crv: EcCurves::P256,
                        d: None,
                        x: Bytes::from(uncompressed[1..33].to_vec()),
                        y: Bytes::from(uncompressed[33..].to_vec()),
                    }),
                    prm: Parameters::default(),
                })
            }
            // TODO! Extend implementation to other algorithms
            _ => Err(CryptoError::Unsupported),
        }
    }

    /// Uncompresses a compressed public key.
    ///
    /// # Parameters
    ///
    /// - `compressed_key_bytes`: The compressed public key bytes.
    ///
    /// # Returns
    ///
    /// The bytes representing the uncompressed key or a `CryptoError`.
    pub fn uncompress_public_key(&self, compressed_key_bytes: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if let Some(required_length) = self.public_key_length() {
            if required_length != compressed_key_bytes.len() {
                return Err(CryptoError::InvalidKeyLength);
            }
        }

        let sec1_generic = |p: BigInt, a: BigInt, b: BigInt| {
            let sign_byte = compressed_key_bytes[0];
            let sign = match sign_byte {
                0x02 => 0u8,
                0x03 => 1u8,
                _ => return Err(CryptoError::InvalidPublicKey),
            };

            let x = BigInt::from_bytes_be(Sign::Plus, &compressed_key_bytes[1..]);
            let y_sq = (x.modpow(&BigInt::from(3u32), &p) + &a * &x + &b) % &p;
            let mut y = y_sq.modpow(&((&p + BigInt::from(1)) / BigInt::from(4)), &p);

            if &y % BigInt::from(2) != (sign % 2).into() {
                y = &p - &y;
            }

            let mut z = vec![0x04];
            z.append(&mut x.to_bytes_be().1);
            z.append(&mut y.to_bytes_be().1);

            Ok(z)
        };

        match self {
            Secp256k1 => {
                let p = "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f";
                let p = BigInt::from_bytes_be(Sign::Plus, &hex::decode(p).unwrap());

                let a = BigInt::from(0);
                let b = BigInt::from(7);

                sec1_generic(p, a, b)
            }
            P256 => {
                let p = "ffffffff00000001000000000000000000000000ffffffffffffffffffffffff";
                let p = BigInt::from_bytes_be(Sign::Plus, &hex::decode(p).unwrap());

                let a = &p - BigInt::from(3);

                let b = "5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b";
                let b = BigInt::from_bytes_be(Sign::Plus, &hex::decode(b).unwrap());

                sec1_generic(p, a, b)
            }
            _ => Err(CryptoError::Unsupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_can_build_secp256k1_jwk() {
        let (alg, bytes) = decode_multibase_key("zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme");
        assert_eq!(alg, Secp256k1);

        let uncompressed = alg.uncompress_public_key(&bytes).unwrap();
        assert_eq!(uncompressed.len(), 65);

        let jwk = alg.build_jwk(&bytes).unwrap();
        let expected: Value = serde_json::from_str(
            r#"{
                "kty": "EC",
                "crv": "secp256k1",
                "x": "h0wVx_2iDlOcblulc8E5iEw1EYh5n1RYtLQfeSTyNc0",
                "y": "O2EATIGbu6DezKFptj5scAIRntgfecanVNXxat1rnwE"
            }"#,
        )
        .unwrap();

        assert_eq!(
            json_canon::to_string(&jwk).unwrap(),      //
            json_canon::to_string(&expected).unwrap(), //
        )
    }

    #[test]
    fn test_can_build_p256_jwk() {
        let (alg, bytes) = decode_multibase_key("zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169");
        assert_eq!(alg, P256);

        let uncompressed = alg.uncompress_public_key(&bytes).unwrap();
        assert_eq!(uncompressed.len(), 65);

        let jwk = alg.build_jwk(&bytes).unwrap();
        let expected: Value = serde_json::from_str(
            r#"{
                "kty": "EC",
                "crv": "P-256",
                "x": "fyNYMN0976ci7xqiSdag3buk-ZCwgXU4kz9XNkBlNUI",
                "y": "hW2ojTNfH7Jbi8--CJUo3OCbH3y5n91g-IMA9MLMbTU"
            }"#,
        )
        .unwrap();

        assert_eq!(
            json_canon::to_string(&jwk).unwrap(),      //
            json_canon::to_string(&expected).unwrap(), //
        )
    }

    #[test]
    fn test_cannot_build_unsupported_jwk() {
        let multibase_keys = [
            (
                BLS12381, //
                concat!(
                    "zUC7K4ndUaGZgV7Cp2yJy6JtMoUHY6u7tkcSYUvPrEidqBmLCTLmi6d5WvwnUqejscAk",
                    "ERJ3bfjEiSYtdPkRSE8kSa11hFBr4sTgnbZ95SJj19PN2jdvJjyzpSZgxkyyxNnBNnY"
                ),
            ),
            (
                P384, //
                "z82Lm1MpAkeJcix9K8TMiLd5NMAhnwkjjCBeWHXyu3U4oT2MVJJKXkcVBgjGhnLBn2Kaau9",
            ),
            (
                P521, //
                "z2J9gaYxrKVpdoG9A4gRnmpnRCcxU6agDtFVVBVdn1JedouoZN7SzcyREXXzWgt3gGiwpoHq7K68X4m32D8HgzG8wv3sY5j7",
            ),
            (
                RSA, //
                concat!(
                    "z4MXj1wBzi9jUstyPMS4jQqB6KdJaiatPkAtVtGc6bQEQEEsKTic4G7Rou3iBf9vPmT5dbkm9qsZsu",
                    "VNjq8HCuW1w24nhBFGkRE4cd2Uf2tfrB3N7h4mnyPp1BF3ZttHTYv3DLUPi1zMdkULiow3M1GfXkoC",
                    "6DoxDUm1jmN6GBj22SjVsr6dxezRVQc7aj9TxE7JLbMH1wh5X3kA58H3DFW8rnYMakFGbca5CB2Jf6",
                    "CnGQZmL7o5uJAdTwXfy2iiiyPxXEGerMhHwhjTA1mKYobyk2CpeEcmvynADfNZ5MBvcCS7m3XkFCMN",
                    "UYBS9NQ3fze6vMSUPsNa6GVYmKx2x6JrdEjCk3qRMMmyjnjCMfR4pXbRMZa3i"
                ),
            ),
        ];

        for (expected_alg, multibase_key) in multibase_keys {
            let (alg, bytes) = decode_multibase_key(multibase_key);
            assert_eq!(alg, expected_alg);
            assert!(matches!(alg.build_jwk(&bytes).unwrap_err(), CryptoError::Unsupported));
        }
    }

    #[test]
    fn test_key_decompression_fails_on_invalid_key_length() {
        let bytes = hex::decode("023d4de48a477e309548a0ed8ceee086d1aaeceb11f0a8e3a0ffb3e5f44602de1800").unwrap();
        let uncompressed = P256.uncompress_public_key(&bytes);
        assert!(matches!(uncompressed.unwrap_err(), CryptoError::InvalidKeyLength));
    }

    #[test]
    fn test_key_decompression_fails_on_invalid_sign_byte() {
        let bytes = hex::decode("113d4de48a477e309548a0ed8ceee086d1aaeceb11f0a8e3a0ffb3e5f44602de18").unwrap();
        let uncompressed = P256.uncompress_public_key(&bytes);
        assert!(matches!(uncompressed.unwrap_err(), CryptoError::InvalidPublicKey));
    }

    fn decode_multibase_key(key: &str) -> (Algorithm, Vec<u8>) {
        let (_, multicodec) = multibase::decode(key).unwrap();

        let prefix: &[u8; 2] = &multicodec[..2].try_into().unwrap();
        let bytes = &multicodec[2..];

        (Algorithm::from_muticodec_prefix(prefix).unwrap(), bytes.to_vec())
    }
}
