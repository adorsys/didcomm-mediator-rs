use std::fs;

use serde_json::Value;

#[tokio::main]
async fn main() {
    println!("=================== GET THE DID DOCUMENT ===================");
    get_mediator_didoc().await.unwrap();

    println!("=================== GET THE MEDIATE REQUEST ===================");
   
    
    println!("=================== GET THE KEYLIST UPDATE ===================");

    
}

async fn get_mediator_didoc() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let did_doc = client
        .get("http://localhost:3000/.well-known/did.json")
        .send()
        .await?
        .text()
        .await?;

    // Check if the DID document is valid JSON before saving it
    let did_doc_json: Value = serde_json::from_str(&did_doc)
        .map_err(|e| format!("Failed to parse DID document: {}", e))?;

    // Save the DID document in pretty JSON format
    fs::write("./mediator_didoc.json", serde_json::to_string_pretty(&did_doc_json).unwrap()).unwrap();



   Ok(())
}