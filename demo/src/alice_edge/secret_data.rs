use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
   pub(crate) static ref MEDIATOR_DID: Mutex<String> = Mutex::new(String::new());
   pub(crate) static ref ROUTING_DID: Mutex<String> = Mutex::new(String::new());
}
