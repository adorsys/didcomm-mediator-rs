use did_utils::didcore::Document;
use std::borrow::Cow;

pub(crate) fn handle_vm_id<'a>(vm_id: &'a str, diddoc: &Document) -> Cow<'a, str> {
    if vm_id.starts_with('#') {
        if let Some(also_known_as) = diddoc.also_known_as.as_ref() {
            Cow::Owned(format!("{}{}", also_known_as[0], vm_id))
        } else {
            Cow::Borrowed(vm_id)
        }
    } else {
        Cow::Borrowed(vm_id)
    }
}

#[cfg(test)]
mod tests {
    use did_utils::didcore::Document;

    use super::handle_vm_id;

    #[test]
    fn test_handle_vm_id() {
        let diddoc = Document {
            also_known_as: Some(vec!["did:example:123".to_owned()]),
            ..Default::default()
        };
        assert_eq!(handle_vm_id("#key-1", &diddoc), "did:example:123#key-1");
        assert_eq!(handle_vm_id("key-1", &diddoc), "key-1");

        let diddoc = Document::default();
        assert_eq!(handle_vm_id("#key-1", &diddoc), "#key-1");
        assert_eq!(handle_vm_id("key-1", &diddoc), "key-1");
    }
}
