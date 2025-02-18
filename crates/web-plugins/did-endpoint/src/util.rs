use did_utils::{didcore::Document, jwk::Jwk};
use multibase::Base;
use serde_json::json;
use std::borrow::Cow;

// This function is a hack to bypass certain constraints of the did:peer method specification.
// Its purpose is to uniquely identify the keys used to generate a Peer DID address in the store.
pub(crate) fn handle_vm_id<'a>(vm_id: &'a str, diddoc: &Document, jwk: &Jwk) -> Cow<'a, str> {
    let base58 = jwk_to_multicodec(jwk);

    if vm_id.starts_with('#') {
        Cow::Owned(diddoc.id.to_owned() + "#" + &base58)
    } else {
        Cow::Borrowed(vm_id)
    }
}

pub(crate) fn jwk_to_multicodec(jwk: &Jwk) -> String {
    let jwk = json!(jwk);
    let public_key = jwk["x"].as_str().unwrap();
    let bytes = Base::Base64Url.decode(public_key).unwrap();
    let alg = jwk["crv"].as_str().unwrap();

    let multicodec_prefix = match alg {
        "Ed25519" => [0xed, 0x01],
        "X25519" => [0xec, 0x01],
        _ => panic!("Unsupported algorithm"),
    };
    Base::Base58Btc.encode([multicodec_prefix.as_slice(), &bytes].concat())
}

// #[cfg(test)]
// mod tests {
//     use did_utils::didcore::Document;

//     use super::handle_vm_id;

//     #[test]
//     fn test_handle_vm_id() {
//         let diddoc = Document {
//             id: "did:example:123".to_owned(),
//             ..Default::default()
//         };
//         assert_eq!(handle_vm_id("#key-1", &diddoc), "did:example:123#key-1");
//         assert_eq!(handle_vm_id("did:key:123#456", &diddoc), "did:key:123#456");

//         let diddoc = Document::default();
//         assert_eq!(handle_vm_id("#key-1", &diddoc), "#key-1");
//         assert_eq!(handle_vm_id("key-1", &diddoc), "key-1");
//     }
// }
