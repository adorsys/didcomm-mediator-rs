## oob-messages

A Rust library for implementing out of band messages for DID-based applications.

Out of band messages (OOB) messages are the initiators of a didcomm communication where by sender provides his identifier in an unencrypted messages (QR-code or Invitation link) which the other party can scan with his/her edge device, hence no private information should be send in this way. The protocol used here is the version 2 https://didcomm.org/out-of-band/2.0/invitation

## Features
-  Creates out of band invitation URL and QR codes.
  
## Installation

```rust
cargo install oob-messages
```

## USAGE
To be able to generate oob invitations, first begin by setting the following local environment variables
**SERVER_PUBLIC_DOMAIN**
**SERVER_LOCAL_PORT**
**STORAGE_DIRPATH**
```rust

// start by creating a new OOBMessages with default name oob_messages
let oobmessagesplugin = OOBMessages;

// initialize mounting of the oob invitation and qr code in the storage directory set in STORAGE_DIRPATH.
// Or returns an error if the plugins are not valid PluginError
oobmessagesplugin.mount();

// Then navigate to the following endpoints on your browser /, /oob_url and /oob_qr,where you will find the server's response, out of band url and out of band qr code respectively.
// Now both parties can start a communication using didcomm protocol.
oobmessagesplugin.routes();

// to revert the mounting initialization, call unmount function on the created oobmessagesplugin
oobmessagesplugin.unmount();

```
## DEBUGGING

```rust
// After setting the environment variables you can try checking if the were well set using calls to get_environment_variables function.
get_environment_variables(oobmessagesplugin);
```
