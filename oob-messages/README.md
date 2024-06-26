## Out Of Band Messages
Out of band messages (OOB) messages are the initiators of a didcomm communication where by sender provides which identifier in an unencrypted messages (QR-code or Invitation link) which the other party can scan with his/her edge device, hence no private information should be send in this way.

## Features
-  Creates out of band invitation URL and QR codes.

## Invitation
An out of band invitation message is usually of the form
```
{
  "type": "https://didcomm.org/out-of-band/2.0/invitation",
  "id": "<id used for context as pthid>",
  "from":"<sender's did>",
  "body": {
    "goal_code": "issue-vc",
    "goal": "To issue a Faber College Graduate credential",
    "accept": [
      "didcomm/v2",
      "didcomm/aip2;env=rfc587"
    ]
  },
  "attachments": [
    {
        "id": "request-0",
        "media_type": "application/json",
        "data": {
            "json": "<json of protocol message>"
        }
    }
  ]
}
```
The required fields for and out of band message in didcomm messaging are

**type** The header conveying the DIDComm MTURI.
**id** This value MUST be used as the parent thread ID (pthid) for the response message that follows. This may feel counter-intuitive — why not it in the thid of the response instead? The answer is that putting it in pthid enables multiple, independent interactions (threads) to be triggered from a single out-of-band invitation.
**from** for OOB usage. The DID representing the sender to be used by recipients for future interactions. 
more of the field can be found [here](https://identity.foundation/didcomm-messaging/spec/#discover-features-protocol-20)

The URL format is as follows
```https://<domain>/<path>?_oob=<encodedplaintextjwm>```
where ```encodedplaintextjwm = b64urlencode(<plaintextjwm>)```, the steps in creating an out of band invitation involves. 

1. Trimming of the widespaces from the plaintextjwm. e.g 
   ```{"type":"https://didcomm.org/out-of-band/2.0/invitation","id":"69212a3a-d068-4f9d-a2dd-4741bca89af3","from":"did:example:alice","body":{"goal_code":"","goal":""},"attachments":[{"id":"request-0","media_type":"application/json","data":{"json":"<json of protocol message>"}}]}```

2. Encoding it to a 64 base encoding e.g ```eyJ0eXBlIjoiaHR0cHM6Ly9kaWRjb21tLm9yZy9vdXQtb2YtYmFuZC8yLjAvaW52aXRhdGlvbiIsImlkIjoiNjkyMTJhM2EtZDA2OC00ZjlkLWEyZGQtNDc0MWJjYTg5YWYzIiwiZnJvbSI6ImRpZDpleGFtcGxlOmFsaWNlIiwiYm9keSI6eyJnb2FsX2NvZGUiOiIiLCJnb2FsIjoiIn0sImF0dGFjaG1lbnRzIjpbeyJpZCI6InJlcXVlc3QtMCIsIm1lZGlhX3R5cGUiOiJhcHBsaWNhdGlvbi9qc29uIiwiZGF0YSI6eyJqc29uIjoiPGpzb24gb2YgcHJvdG9jb2wgbWVzc2FnZT4ifX1dfQ```

3. Generating the inviation Url follows the following pattern ```https://<domain>/<path>?_oob=<encodedplaintextjwm>```.<br><domain> and <path> should be kept as short as possible, and the full URL should return human readable instructions when loaded in a browser. This is intended to aid new users. The _oob query parameter is required and is reserved to contain the DIDComm message string. Additional path elements or query parameters are allowed, and can be leveraged to provide coupons or other promise of payment for new users. e.g
```
   https://example.com/path?_oob=eyJ0eXBlIjoiaHR0cHM6Ly9kaWRjb21tLm9yZy9vdXQtb2YtYmFuZC8yLjAvaW52aXRhdGlvbiIsImlkIjoiNjkyMTJhM2EtZDA2OC00ZjlkLWEyZGQtNDc0MWJjYTg5YWYzIiwiZnJvbSI6ImRpZDpleGFtcGxlOmFsaWNlIiwiYm9keSI6eyJnb2FsX2NvZGUiOiIiLCJnb2FsIjoiIn0sImF0dGFjaG1lbnRzIjpbeyJpZCI6InJlcXVlc3QtMCIsIm1lZGlhX3R5cGUiOiJhcHBsaWNhdGlvbi9qc29uIiwiZGF0YSI6eyJqc29uIjoiPGpzb24gb2YgcHJvdG9jb2wgbWVzc2FnZT4ifX1dfQ
```
From encode URL generate QR code
![](QR_Code.png)
## USAGE
```rust
// generate oob message
let oob_message = OobMessage::new(&did);

//serialize oob message
let oob_url = OobMessage::serialize_oob_message(&oob_message, &url)
            .unwrap_or_else(|err| panic!("Failed to serialize oob message: {}", err));

// generate oob invitation
  let oob_inviation = retrieve_or_generate_oob_inv(
            &mut mock_fs,
            &server_public_domain,
            &server_local_port,
            &storage_dirpath,
        ).unwrap();

// Generate QR code
    let code = match QrCode::new(url.as_bytes()) {
        Ok(qr_code) => qr_code,
        Err(e) => return Err(format!("QR code generation error: {}", e)),
    };

    let image = code.render::<Luma<u8>>().build();

```
## Dependencies
- serde
- url
- base64
- multibase
  