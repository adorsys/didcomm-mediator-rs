use alice_edge::alice::{
    get_mediator_didoc, handle_live_delivery_change, keylist_query_payload, keylist_update_payload, mediate_request, test_pickup_delivery_request, test_pickup_message_received, test_pickup_request
};
use bob_edge::bob::forward_msg;
const DIDCOMM_CONTENT_TYPE: &str = "application/didcomm-encrypted+json";
pub const BOB_DID: &str = "did:example:bob";

mod alice_edge;
mod bob_edge;
pub(crate) mod constants;
mod ledger;
#[tokio::main]
async fn main() {
    println!("\n=================== GETTING MEDIATOR DID DOCUMENT ===================\n");
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
    test_pickup_message_received().await;
    
    println!("\n=================== LIVE DILIVERY ===================\n");
    handle_live_delivery_change().await;

    println!("\n=================== KEYLIST QUERY RESPONSE ===================\n");
    keylist_query_payload().await;
}
