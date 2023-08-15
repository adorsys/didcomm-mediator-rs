use serde::{Serialize, Deserialize};
use crate::constants::MEDIATE_GRANT_2_0;


// region: --- Model
 
/// A mediate request message. 
/// 
/// e.g.: 
/// ```
/// {
///    "id": "123456780",
///    "type": "https://didcomm.org/coordinate-mediation/2.0/mediate-request",
/// }
/// ```
/// 
/// In this request, the recipient provides an extended public key. 
/// This extended public is signed by the recipient as a proof of ownership of the private key.
/// - This is the identity key of the recipient.
/// - This extended public key is sent to the mediator, 
/// - The mediator returns a signature of the identity key of the recipient.
/// - The mediator does not store any additional info on the receiver.
///
/// For each contact, 
/// - the recipient will derive a separate peer did public keys from the extended public key.
/// - the recipient uses the public key of the mediator to encrypt:
///   - the identity public key of the recipient
///   - the mediator signature of the identity public key of the recipient
///   - the hd path of the peer did public key of the contact
/// - This encrypted object is the auth token of the sender.
/// 
/// For a forward request, the sender will :
/// - add the auth token to the message
/// - encrypt/sign the message with the public key of the mediator
/// 
/// In order to process the forward request
/// - decrypt verify the forward message,
/// - decrypt the auth token and 
///   - verify that the ginature associate with the identity key of the reciver
///   - use the hd path to derive the public key assigned by the receiver to the contact
///   - verify that this public key is use to encrypt/sign the inner message
///   - store the message for pickup by the receiver.
///   - evetl. notify the reciver for available message.
/// 
/// The message pickup request is signed with the identity key of the receiver.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediateRequest {
    pub id: String,
    #[serde(rename = "@type")]
    pub type_field: String,
}

// A struct that can contain a message that looks like this
// ```
// {
//    "id": "123456780",
//    "type": "https://didcomm.org/coordinate-mediation/2.0/mediate-deny",
// }
// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediateDeny {
    pub id: String,
    #[serde(rename = "@type")]
    pub type_field: &'static str,
}

// A struct that can contain a message that looks like this
// ```
// {
//     "id": "123456780",
//     "type": "https://didcomm.org/coordinate-mediation/2.0/mediate-grant",
//     "body": {
//           "routing_did": "did:peer:z6Mkfriq1MqLBoPWecGoDLjguo1sB9brj6wT3qZ5BxkKpuP6"
//      }
// }
// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediateGrant {
    pub id: String,
    #[serde(rename = "@type")]
    pub type_field: &'static str,
    pub body: MediateGrantBody,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediateGrantBody {
    pub routing_did: String,
}

// A constructor for MediateGrant
impl MediateGrant {
    pub fn new(id: String, routing_did: String) -> Self {
        MediateGrant {
            id,
            type_field: MEDIATE_GRANT_2_0,
            body: MediateGrantBody {
                routing_did,
            },
        }
    }
}

// endregion: --- Model