use base64::prelude::{Engine as _, BASE64_STANDARD};
use did_utils::didcore::Document;

// This function is a hack to bypass certain constraints of the did:peer method specification.
// Its purpose is to uniquely identify the keys used to generate a Peer DID address in the store.
pub(crate) fn handle_vm_id(vm_id: &str, diddoc: &Document) -> String {
    let kid = if vm_id.starts_with('#') {
        diddoc.id.to_owned() + vm_id
    } else {
        vm_id.to_owned()
    };

    BASE64_STANDARD.encode(kid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use did_utils::didcore::Document;

    use super::handle_vm_id;

    #[test]
    fn test_handle_vm_id() {
        let diddoc = Document {
            id: "did:example:123".to_owned(),
            ..Default::default()
        };
        let expected_kid = BASE64_STANDARD.encode("did:example:123#key-1");
        assert_eq!(handle_vm_id("#key-1", &diddoc), expected_kid);
        let expected_kid = BASE64_STANDARD.encode("did:key:123#456");
        assert_eq!(handle_vm_id("did:key:123#456", &diddoc), expected_kid);

        let diddoc = Document::default();
        assert_eq!(
            handle_vm_id("#key-1", &diddoc),
            BASE64_STANDARD.encode("#key-1")
        );
        assert_eq!(
            handle_vm_id("key-1", &diddoc),
            BASE64_STANDARD.encode("key-1")
        );
    }
}
