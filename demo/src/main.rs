use alice_edge::alice::{
    get_mediator_didoc, keylist_update_payload, mediate_request, test_pickup_delivery_request,
    test_pickup_request,
};
use bob_edge::bob::forward_msg;
const DIDCOMM_CONTENT_TYPE: &str = "application/didcomm-encrypted+json";
pub const BOB_DID: &str = "did:example:bob";

mod alice_edge;
mod bob;
mod bob_edge;
mod ledger;
#[tokio::main]
async fn main() {
    println!("\n=================== GET THE DID DOCUMENT ===================\n");
    get_mediator_didoc().await;

    println!("\n=================== MEDIATING REQUEST ===================\n");
    mediate_request().await;

    println!("\n=================== GET THE KEYLIST UPDATE PAYLOAD ===================\n");
    keylist_update_payload().await;

    println!("\n=================== FORWARDING MESSAGES ===================\n");
    forward_msg().await;

    println!("\n=================== PICKUP REQUEST ===================\n");
    test_pickup_request().await;

    println!("\n=================== PICKUP DELIVERY ===================\n");
    test_pickup_delivery_request().await;

    println!("\n=================== MESSAGE RECEIVED ===================\n");
   // test_pickup_message_received().await;
    // println!("\n=================== GET THE KEYLIST QUERY PAYLOAD ===================\n");
    // keylist_query_payload().await;

}
