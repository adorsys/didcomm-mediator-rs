use did_utils::didcore::Document;
use std::borrow::Cow;

// This function is a hack to bypass certain constraints of the did:peer method specification.
// Its purpose is to uniquely identify the keys used to generate a Peer DID address in the store.
pub(crate) fn handle_vm_id<'a>(vm_id: &'a str, diddoc: &Document) -> Cow<'a, str> {
    if vm_id.starts_with('#') {
        Cow::Owned(diddoc.id.to_owned() + vm_id)
    } else {
        Cow::Borrowed(vm_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use did_utils::didcore::Document;

    #[test]
    fn test_handle_vm_id() {
        let diddoc = Document {
            id: "did:example:123".to_owned(),
            ..Default::default()
        };
        let expected_kid = "did:example:123#key-1";
        assert_eq!(handle_vm_id("#key-1", &diddoc), expected_kid);
        let expected_kid = "did:key:123#456";
        assert_eq!(handle_vm_id("did:key:123#456", &diddoc), expected_kid);

        let diddoc = Document::default();
        assert_eq!(handle_vm_id("#key-1", &diddoc), "#key-1");
        assert_eq!(handle_vm_id("key-1", &diddoc), "key-1");
    }
}
