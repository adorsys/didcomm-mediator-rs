use crate::Result;
use serde::{Deserialize, Serialize};
use std::{sync::{Arc, Mutex}};
// use std::marker::PhantomData;


use self::{recipient_key_record_map::RecipientKeyRecordMap};

mod recipient_key_record_map;

// region: --- Model

/// A recipient key record.
/// 
/// Privacy by Design
/// 
/// - A recipient shall have a distinct keys (public) for each relationship. Those distinct did shall be built
/// such as to prevent any sort of correlation of recipient identity among other agents.
/// 
/// - Despite distincts keys, the target recipient of a forward request must be identifiable by the mediator,
/// upon recieving the did provided by the sender of the forward request.
/// 
/// - In orther to provide for efficient management of keys, the recipient shall be able to derive those keys
/// from a single master keypair. This capability can be achieved using hierarchical deterministic keys. See BIP332.
/// 
/// - In orther to allow for lookup efficiency in the protocol, the recipient shall hot have to register 
/// every single derived key with the mediator.
/// 
/// A design  of the form <did: encrypt(recipient-master-pub@hdpath, mediator-pub)> where:
/// - the mediator-pub is the public key used to encrypt the forward message
/// - the recipient-master-pub@hdpath is the reference to the recipient as known to the mediator and a hdpath
/// used to derived the public key and a hdpath used by the recipient to generate the public key known to
/// the sender agent.
/// 
/// This design allow us :
/// - to have one single record per (recipient, mediator) relationship in the mediator database
/// - to have one single record per (recipient, sender) relationship in the recipient database
/// - to only have to store a master public key in each both recipient and mediator database.
/// 
/// It is essential fo the mediator not to have a separate mediator public key per recipient, as this will
/// obviously lead to the corellation of all did produced by that recipient.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RecipientKeyRecord {
    // The mediator master public key id. 
    // As we do not want mediator to mainttain too many keys, this master public key will be used 
    // to derive the recipient master contact key.
    pub mediatorMasterPubKeyId: String,

    // The mediator hd path used to derive the mediator master key for this recipient
    pub mediatorHdPath: String,

    // The recipient master public key id.
    // used to identify this record.
    pub recipientMasterPubKeyId: String,

    // The expiration of this entry
    pub expirTs: u64,
}

// A constructor for a RecipientKeyRecord
impl RecipientKeyRecord {

    pub fn new(mediatorMasterPubKeyId: String, 
        mediatorHdPath: String,
        recipientMasterPubKeyId: String,
        expirTs: u64,
    )   -> Self 
    {
        RecipientKeyRecord {
            mediatorMasterPubKeyId,
            mediatorHdPath,
            recipientMasterPubKeyId,
            expirTs
        }
    }
}

// endregion: --- Model

// region --- Store

// endregion --- Store

// region: --- Controller

#[derive(Clone)]
pub struct RecipientController {
    recipient_key_record_map: Arc<Mutex<RecipientKeyRecordMap>>,

    // phantom: PhantomData<&'a RecipientKeyRecordMap>,
}

// Constructor for the recipient controller.
impl RecipientController {
    pub async fn new() -> RecipientController {
        Self {
            recipient_key_record_map: Arc::default(),
            // phantom: PhantomData,
        }
    }
}

impl RecipientController {
    pub async fn process_mediation_request(&self, recipientMasterPubKeyId: &str) -> Result<&str> {
        // Make sure we do not have an entry with this recipientMasterPubKeyId

        // resolve a mediatorMasterPubKeyId for use for this entry

        // generate a mediatorHdPath for this entry
        
        todo!()
    }

    pub(crate) async fn list_keys(&self) -> Result<Vec<String>> {
        todo!()
    }
}

// endregion: --- Controller
