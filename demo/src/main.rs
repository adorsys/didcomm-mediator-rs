
use alice_edge::alice::{get_mediator_didoc, keylist_update_payload, mediate_request};
use bob_edge::bob::forward_msg;
const DIDCOMM_CONTENT_TYPE: &str = "application/didcomm-encrypted+json";

mod ledger;
mod alice_edge;
mod bob_edge;
#[tokio::main]
async fn main() {
    println!("\n=================== GET THE DID DOCUMENT ===================\n");
    get_mediator_didoc().await;

    println!("\n=================== MEDIATING REQUEST ===================\n");
    mediate_request().await;

    println!("\n=================== GET THE KEYLIST UPDATE PAYLOAD ===================\n");
    keylist_update_payload().await;
    forward_msg().await;
    // println!("\n=================== GET THE KEYLIST QUERY PAYLOAD ===================\n");
    // keylist_query_payload().await;
    // println!("\n=================== PICKUP REQUEST ===================\n");
    // test_pickup_request().await;
    // println!("\n=================== PICKUP DELIVERY ===================\n");
    // test_pickup_delivery_request().await;

    // println!("\n=================== MESSAGE RECEIVED ===================\n");
    // test_pickup_message_received().await;
}






// async fn keylist_query_payload() {
//     let client = Client::new();

//     // --- Building message from ALICE to MEDIATOR ---
//     let message = Message::build(
//         "id_alice_keylist_query".to_owned(),
//         "https://didcomm.org/coordinate-mediation/2.0/keylist-query".to_owned(),
//         json!({}),
//     )
//     .to(MEDIATOR_DID.to_owned())
//     .from(ALICE_DID.to_owned())
//     .finalize();

//     // --- Packing encrypted and authenticated message ---
//     let did_resolver =
//         ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
//     let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

//     let (message, _) = message
//         .pack_encrypted(
//             &MEDIATOR_DID,
//             Some(&ALICE_DID),
//             None,
//             &did_resolver,
//             &secrets_resolver,
//             &PackEncryptedOptions::default(),
//         )
//         .await
//         .expect("Unable to pack_encrypted");

//     // --- Sending message by Alice ---
//     println!("Edge agent is sending message \n{}\n", message);

//     let response = client
//         .post("http://localhost:3000/mediate")
//         .header(CONTENT_TYPE, "application/didcomm-encrypted+json")
//         .body(message)
//         .send()
//         .await
//         .unwrap()
//         .text()
//         .await
//         .unwrap();

//     // --- Unpack the message ---
//     let (message, _) = Message::unpack(
//         &response,
//         &did_resolver,
//         &secrets_resolver,
//         &UnpackOptions::default(),
//     )
//     .await
//     .unwrap();
//     println!("\n{:#?}\n", message);
// }
// async fn test_pickup_request() {
//     let did_resolver =
//         ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
//     let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

//     // Build message
//     let msg = Message::build(
//         "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//         "https://didcomm.org/messagepickup/3.0/status-request".to_owned(),
//         json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
//     )
//     .header("return_route".into(), json!("all"))
//     .to(MEDIATOR_DID.to_string())
//     .from(ALICE_DID.to_string())
//     .finalize();

//     // Encrypt message for mediator

//     let (msg, _) = msg
//         .pack_encrypted(
//             &MEDIATOR_DID,
//             Some(&ALICE_DID),
//             None,
//             &did_resolver,
//             &secrets_resolver,
//             &PackEncryptedOptions::default(),
//         )
//         .await
//         .expect("Unable to pack_encrypted");
//     let client = reqwest::Client::new();
//     let response = client
//         .post("http://localhost:3000/mediate")
//         .header(CONTENT_TYPE, DIDCOMM_CONTENT_TYPE)
//         .body(msg)
//         .send()
//         .await
//         .unwrap()
//         .text()
//         .await
//         .unwrap();

//     // unpack response
//     let (msg, _) = Message::unpack(
//         &response,
//         &did_resolver,
//         &secrets_resolver,
//         &UnpackOptions::default(),
//     )
//     .await
//     .unwrap();

//     println!("\nPickup Request Message\n{:#?}", msg);
// }
// async fn test_pickup_delivery_request() {
//     let did_resolver =
//         ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
//     let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

//     // Build message
//     let msg = Message::build(
//         "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//         "https://didcomm.org/messagepickup/3.0/delivery-request".to_owned(),
//         json!({"limit":1,"recipient_did":"did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
//     )
//     .header("return_route".into(), json!("all"))
//     .to(MEDIATOR_DID.to_string())
//     .from(ALICE_DID.to_string())
//     .finalize();

//     let (msg, _) = msg
//         .pack_encrypted(
//             &MEDIATOR_DID,
//             Some(&ALICE_DID),
//             None,
//             &did_resolver,
//             &secrets_resolver,
//             &PackEncryptedOptions::default(),
//         )
//         .await
//         .expect("Unable to pack_encrypted");
//     println!("{}", msg);
//     let client = reqwest::Client::new();
//     let response = client
//         .post("http://localhost:3000/mediate")
//         .header(CONTENT_TYPE, DIDCOMM_CONTENT_TYPE)
//         .body(msg)
//         .send()
//         .await
//         .unwrap()
//         .text()
//         .await
//         .unwrap();

//     // unpack response
//     let (msg, _) = Message::unpack(
//         &response,
//         &did_resolver,
//         &secrets_resolver,
//         &UnpackOptions::default(),
//     )
//     .await
//     .unwrap();

//     println!("\nPickup Delivery Message\n{:#?}", msg);
// }

// async fn test_pickup_message_received() {
//     let did_resolver =
//         ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
//     let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

//     // Build message
//     let msg = Message::build(
//         "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
//         "https://didcomm.org/messagepickup/3.0/messages-received".to_owned(),
//         json!({"message_id_list": vec!["66ec4d76e8aaed777d76acf9","66ec4d75e8aaed777d76acf8"]}),
//     )
//     .header("return_route".into(), json!("all"))
//     .to(MEDIATOR_DID.to_string())
//     .from(ALICE_DID.to_string())
//     .finalize();

//     let (msg, _) = msg
//         .pack_encrypted(
//             &MEDIATOR_DID,
//             Some(&ALICE_DID),
//             None,
//             &did_resolver,
//             &secrets_resolver,
//             &PackEncryptedOptions::default(),
//         )
//         .await
//         .expect("Unable to pack_encrypted");
//     println!("{}", msg);
//     let client = reqwest::Client::new();
//     let response = client
//         .post("http://localhost:3000/mediate")
//         .header(CONTENT_TYPE, DIDCOMM_CONTENT_TYPE)
//         .body(msg)
//         .send()
//         .await
//         .unwrap()
//         .text()
//         .await
//         .unwrap();

//     // unpack response
//     let (msg, _) = Message::unpack(
//         &response,
//         &did_resolver,
//         &secrets_resolver,
//         &UnpackOptions::default(),
//     )
//     .await
//     .unwrap();

//     println!("\nMessage Received\n{:#?}", msg);
// }
