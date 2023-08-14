use crate::models::RecipientKeyRecord;
use std::collections::HashMap;

#[derive(Default)]
pub struct RecipientKeyRecordMap {
    datastore: HashMap<String, RecipientKeyRecord>,
}

impl RecipientKeyRecordMap {
    pub fn new() -> Self {
        Self {
            datastore: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, key: String, value: RecipientKeyRecord) {
        self.datastore.insert(key, value);
    }

    pub fn get_entry_by_key(&self, key: &str) -> Option<&RecipientKeyRecord> {
        self.datastore.get(key)
    }

    pub fn delete_entry_by_key(&mut self, key: String) {
        self.datastore.remove(&key);
    }
}

// The test class
#[test]
fn test_map() {
    let value1 = &RecipientKeyRecord {
        mediatorMasterPubKeyId: String::from("sadfsadf"),
        mediatorHdPath: String::from("m.2.0.'2"),
        recipientMasterPubKeyId: String::from("asdfasdfasdf"),
        expirTs: 1542341234,
    };
    let value2 = &RecipientKeyRecord {
        mediatorMasterPubKeyId: String::from("erttyrtyerrt"),
        mediatorHdPath: String::from("m.1.0.'3"),
        recipientMasterPubKeyId: String::from("adssldfgksdf"),
        expirTs: 1542341234,
    };

    let mut datastore = RecipientKeyRecordMap::new();

    let recipientMasterPubKeyId1 = value1.recipientMasterPubKeyId.to_owned();
    let recipientMasterPubKeyId2 = value2.recipientMasterPubKeyId.to_owned();

    datastore.add_entry(recipientMasterPubKeyId1.to_owned(), value1.clone());
    datastore.add_entry(recipientMasterPubKeyId2.to_owned(), value2.clone());

    assert_eq!(datastore.get_entry_by_key(&recipientMasterPubKeyId1).unwrap(), value1);

    datastore.delete_entry_by_key(recipientMasterPubKeyId2.to_owned());

    assert_eq!(datastore.get_entry_by_key(&recipientMasterPubKeyId2), None);
}
