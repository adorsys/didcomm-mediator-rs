use didcomm::{
    did::resolvers::ExampleDIDResolver, secrets::resolvers::ExampleSecretsResolver, Message,
    PackEncryptedOptions,
};
use ledger::{ALICE_DID, ALICE_DID_DOC, ALICE_SECRETS, MEDIATOR_DID, MEDIATOR_DID_DOC};
use serde_json::json;

mod ledger;
#[tokio::main]
async fn main() {
    keylist_update_payload().await;
    // repudiable_authenticated_encryption().await;
}

async fn keylist_update_payload() {
    // --- Building message from ALICE to MEDIATOR ---
    let msg = Message::build(
        "id_alice_keylist_update_request".to_owned(),
        "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
        json!({"updates": [
            {
                "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                "action": "add"
            },
            {
                "recipient_did": "did:key:alice_identity_pub2@alice_mediator",
                "action": "remove"
            }
        ]}),
    )
    .header("return_route".into(), json!("all"))
    .to(MEDIATOR_DID.to_owned())
    .from(ALICE_DID.to_owned())
    .finalize();

    // --- Packing encrypted and authenticated message ---
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    let (msg, _) = msg
        .pack_encrypted(
            &MEDIATOR_DID,
            Some(&ALICE_DID),
            None,
            &did_resolver,
            &secrets_resolver,
            &PackEncryptedOptions::default(),
        )
        .await
        .expect("Unable to pack_encrypted");

    // --- Sending message by Alice ---
    println!("Edge agent is sending message \n{}\n", msg);
}
