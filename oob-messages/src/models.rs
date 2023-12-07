use crate::constants::OOB_INVITATION_2_0;
use base64::{encode_config, STANDARD};
use image::{DynamicImage, Luma};
use multibase::Base::Base64Url;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::error::Error;
use url::{ParseError, Url};

use did_endpoint::util::filesystem::FileSystem;

#[cfg(test)]
use std::io::{Error as IoError, ErrorKind, Result as IoResult};

// region: --- Model

// OOB Inv: Is an unencrypted message (In the form of an URL or QR code that contains a b64urlencode JWM) with the mediator public DID.

// The out-of-band protocol consists in a single message that is sent by the sender.

// This is the first step in the interaction with the Mediator. The following one is the mediation coordination where a 'request mediation' request is created and performed.
/// e.g.:
/// ```
// {
//     "type": "https://didcomm.org/out-of-band/2.0/invitation",
//     "id": "0a2c57a5-5662-48a8-bca8-78275cef3c80",
//     "from": "did:peer:2.Ez6LSms555YhFthn1WV8ciDBpZm86hK9tp83WojJUmxPGk1hZ.Vz6MkmdBjMyB4TS5UbbQw54szm8yvMMf1ftGV2sQVYAxaeWhE.SeyJpZCI6Im5ldy1pZCIsInQiOiJkbSIsInMiOiJodHRwczovL21lZGlhdG9yLnJvb3RzaWQuY2xvdWQiLCJhIjpbImRpZGNvbW0vdjIiXX0",
//     "body": {
//       "goal_code": "request-mediate",
//       "goal": "RequestMediate",
//       "label": "Mediator",
//       "accept": [
//         "didcomm/v2"
//       ]
//     }
//   }
/// ```

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OobMessage {
    #[serde(rename = "type")]
    oob_type: String,
    id: String,
    from: String,
    body: Body,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "use")]
struct Body {
    goal_code: String,
    goal: String,
    label: String,
    accept: Vec<String>,
}

impl OobMessage {
    fn new(did: &str) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let body = Body {
            goal_code: String::from("request-mediate"),
            goal: String::from("Request Mediate"),
            label: String::from("Mediator"),
            accept: vec![String::from("didcomm/v2")],
        };

        OobMessage {
            oob_type: String::from(OOB_INVITATION_2_0),
            id,
            from: String::from(did),
            body,
        }
    }

    fn serialize_oob_message(oob_message: &OobMessage, url: &str) -> Result<String, String> {
        let plaintext =
            to_string(oob_message).map_err(|e| format!("Serialization error: {}", e))?;
        let encoded_jwm = Base64Url.encode(plaintext.as_bytes());

        Ok(format!("{}?_oob={}", url, encoded_jwm))
    }
}

// Receives server path/port and local storage path and returns a String with the OOB URL.
pub fn retrieve_or_generate_oob_inv<'a>(
    fs: &mut dyn FileSystem,
    server_public_domain: &str,
    server_local_port: &str,
    storage_dirpath: &str,
) -> Result<String, String>  {
    // Construct the file path
    let file_path = format!("{}/oob_invitation.txt", storage_dirpath);

    // Attempt to read the file directly
    if let Ok(content) = fs.read_to_string(&file_path) {
        // If successful, return the content
        eprintln!("OOB Invitation successfully retrieved from file");
        return Ok(content);
    }

    // If the file doesn't exist, proceed with creating and storing it
    let did =
        url_to_did_web_id(&format!("{}:{}/", server_public_domain, server_local_port)).unwrap();
    let oob_message = OobMessage::new(&did);
    let url: &String = &format!("{}:{}", server_public_domain, server_local_port);
    let oob_url = OobMessage::serialize_oob_message(&oob_message, url).map_err(|e| format!("Serialization error: {}", e))?;

    // Attempt to create the file and write the string
    to_local_storage(fs, &oob_url, storage_dirpath);

    Ok(oob_url)
}

// Function to generate and save a QR code image
pub fn retrieve_or_generate_qr_image(
    fs: &mut dyn FileSystem,
    base_path: &str,
    url: &str,
) -> Result<String, String> {
    let path = format!("{}/qrcode.txt", base_path);

    // Check if the file exists in the specified path, otherwise creates it
    if let Ok(existing_image) = fs.read_to_string(&path) {
        return Ok(existing_image);
    }

    // Generate QR code
    let code = match QrCode::new(url.as_bytes()) {
        Ok(qr_code) => qr_code,
        Err(e) => return Err(format!("QR code generation error: {}", e)),
    };

    let image = code.render::<Luma<u8>>().build();


    // Convert the image to a PNG-encoded byte vector
    let dynamic_image = DynamicImage::ImageLuma8(image);
    let mut buffer = Vec::new();
    dynamic_image
        .write_to(&mut buffer, image::ImageOutputFormat::Png)
        .expect("Error encoding image to PNG");

    // Save the PNG-encoded byte vector as a base64-encoded string
    let base64_string = encode_config(&buffer, STANDARD);

    match  fs.write_with_lock(&path, &base64_string) {
        Ok(_) => {
        },
        Err(error_message) => {
            return Err(format!("Error writing with lock: {}", error_message));
        },
    }

    Ok(base64_string)
}

fn to_local_storage(fs: &mut dyn FileSystem, oob_url: &str, storage_dirpath: &str) {
    // Ensure the parent directory ('storage') exists
    if let Err(e) = fs.create_dir_all(storage_dirpath) {
        eprintln!("Error creating directory: {}", e);
        return;
    }

    let file_path = format!("{}/oob_invitation.txt", storage_dirpath);

    // Attempt to write the string directly to the file
    if let Err(e) = fs.write(&file_path, oob_url) {
        eprintln!("Error writing to file: {}", e);
    } else {
        println!("String successfully written to file.");
    }
}

/// Turns an HTTP(S) URL into a did:web id.
fn url_to_did_web_id(url: &str) -> Result<String, Box<dyn Error>> {
    let url = Url::parse(url)?;

    // Validate the scheme
    match url.scheme() {
        "http" | "https" => (),
        _ => return Err(ParseError::IdnaError.into()),
    }

    let domain = url.domain().ok_or(ParseError::EmptyHost)?;
    let port = url.port().map(|port| format!("%3A{}", port)).unwrap_or_default();
    let path = url.path().replace('/', ":").trim_start_matches(':').to_string();

    Ok(format!("did:web:{}{}{}", domain, port, path))
}

#[cfg(test)]
#[derive(Default)]
pub struct MockFileSystem;

#[cfg(test)]
impl FileSystem for MockFileSystem {
    fn read_to_string(&self, path: &str) -> IoResult<String> {
        match path {
            p if p.ends_with("oob_invitation.txt") => {
                Ok(include_str!("../test/storage/oob_invitation.txt").to_string())
            }
            p if p.contains("qrcode.txt") => {
                Ok(include_str!("../test/storage/qrcode.txt").to_string())
            }
            _ => Err(IoError::new(ErrorKind::NotFound, "NotFound")),
        }
    }

    fn write(&mut self, _path: &str, _content: &str) -> IoResult<()> {
        Ok(())
    }

    fn read_dir_files(&self, _path: &str) -> IoResult<Vec<String>> {
        Ok(vec![])
    }

    fn create_dir_all(&mut self, _path: &str) -> IoResult<()> {
        Ok(())
    }

    fn write_with_lock(&self, _path: &str, _content: &str) -> IoResult<()>{
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::dotenv_flow_read;
    
    #[test]
    fn test_create_oob_message() {
        let did = url_to_did_web_id(&format!("https://testadress.com:3000/")).unwrap();

        let oob_message = OobMessage::new(&did);

        assert_eq!(oob_message.oob_type, OOB_INVITATION_2_0);
        assert!(!oob_message.id.is_empty());
        assert_eq!(oob_message.from, did);
        assert_eq!(oob_message.body.goal_code, "request-mediate");
        assert_eq!(oob_message.body.goal, "Request Mediate");
        assert_eq!(oob_message.body.label, "Mediator");
        assert_eq!(oob_message.body.accept, vec!["didcomm/v2"]);
    }

    #[test]
    fn test_serialize_oob_message() {
        // Assuming url_to_did_web_id and dotenv_flow_read return Results, you should handle errors.
        let did = url_to_did_web_id(&format!("https://testadress.com:3000/")).expect("Failed to get DID");

        let server_public_domain =
            dotenv_flow_read("SERVER_PUBLIC_DOMAIN").expect("Failed to read SERVER_PUBLIC_DOMAIN");
        let server_local_port =
            dotenv_flow_read("SERVER_LOCAL_PORT").expect("Failed to read SERVER_LOCAL_PORT");

        let url = format!("{}:{}", server_public_domain, server_local_port);

        let oob_message = OobMessage::new(&did);

        // Use unwrap_or_else to handle the error case more gracefully
        let oob_url = OobMessage::serialize_oob_message(&oob_message, &url)
            .unwrap_or_else(|err| panic!("Failed to serialize oob message: {}", err));

        assert!(!oob_url.is_empty());
        assert!(oob_url.starts_with(&format!("{}?_oob=", url)));
        assert!(oob_url.contains("_oob="));
    }

    #[test]
    fn test_retrieve_or_generate_oob_inv() {
        // Test data
        let server_public_domain = dotenv_flow_read("SERVER_PUBLIC_DOMAIN").unwrap();
        let server_local_port = dotenv_flow_read("SERVER_LOCAL_PORT").unwrap();
        let storage_dirpath = String::from("testpath");

        let mut mock_fs = MockFileSystem;

        let result = retrieve_or_generate_oob_inv(
            &mut mock_fs,
            &server_public_domain,
            &server_local_port,
            &storage_dirpath,
        );
        assert!(result.is_ok());

        let didpath = format!("{storage_dirpath}/oob_invitation.txt");
        let file_content = mock_fs.read_to_string(&didpath).unwrap();

        assert_eq!(result.unwrap(), file_content);
    }

    #[test]
    fn test_retrieve_or_generate_qr_image() {
        let mut mock_fs = MockFileSystem;
        let url = "https://example.com";
        let storage_dirpath = String::from("testpath");
    
        let result = retrieve_or_generate_qr_image(&mut mock_fs, &storage_dirpath, &url);
        assert!(result.is_ok());
    
        let image_data = result.unwrap();

        let expected_result = mock_fs
            .read_to_string(&format!("{}/qrcode.txt", storage_dirpath))
            .unwrap();
    
        assert_eq!(image_data, expected_result);
    }
}
