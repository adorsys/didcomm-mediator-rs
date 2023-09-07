use did_utils::{
    crypto::{ed25519::Ed25519KeyPair, traits::CoreSign},
    didcore::Jwk,
};
use multibase::Base::Base64Url;

fn main() {
    let jwk: Jwk = serde_json::from_str(
        r#"{
            "kty": "OKP",
            "crv": "Ed25519",
            "x": "tjOTPcs4OEMNrmn2ScYZDS-aCCbRFhJgaAmGnRsdmEo"
        }"#,
    )
    .unwrap();

    let payload = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    let signature = "2QH7Qrt8clEn4ETh9lgcGUyo26cJj1U8U0CBFQvgCWHe1dwXXXb16SzPTVNVGm-J6m6eALjWrxuJfmbApdoBAQ";
    let signature = Base64Url.decode(signature).unwrap();

    let keypair: Ed25519KeyPair = jwk.try_into().expect("ConversionError");
    assert!(keypair.verify(payload, &signature).is_ok());
}
