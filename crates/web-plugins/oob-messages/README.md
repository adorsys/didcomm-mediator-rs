# oob-messages

Implementation of the [Out of Band](https://didcomm.org/out-of-band/2.0/invitation) protocol for the [Didcomm Mediator](https://github.com/adorsys/didcomm-mediator-rs/) server.

Out of band messages (OOB) messages are the initiators of a didcomm communication where by sender provides his identifier in an unencrypted messages (QR-code or Invitation link) which the other party can scan with his/her edge device, hence no private information should be send in this way. The [protocol](https://didcomm.org/out-of-band/2.0/invitation) used here is the version 2.
