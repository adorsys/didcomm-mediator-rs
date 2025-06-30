extern crate did_utils;

use multibase::Base;

fn main() {
    let public_key = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    ];

    let base58_public_key = Base::Base58Btc.encode(public_key);

    println!("The base58 public key is: {base58_public_key}");

    let base58_multi_public_key = multibase::encode(Base::Base58Btc, public_key);

    println!("The base58 multi public key is: {base58_multi_public_key}");
}
