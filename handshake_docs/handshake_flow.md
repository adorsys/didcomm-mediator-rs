# Handshake Flow for DIDComm Mediator-rs

## 1. Communication Initiation
### Participant Identification:
- Each mobile agent generates a DID (Decentralized Identifier).

- Agents exchange their DIDs to identify each other. The initial exchange might involve sending a DIDComm message that includes the agent’s DID and a nonce (a random value used for security purposes).
### Initial Message:
- The initiating agent sends a DIDComm handshake message to the mediator. This message contains:

  - **DID of the initiating agent**: Identifies the sender.
  - **Nonce**: Ensures message freshness and prevents replay attacks.
  - **Service Information**: May include endpoint URLs, service types, and other metadata relevant to the communication.
## 2. Message Exchanges
### Encryption Methods:
 - Messages are encrypted using DIDComm’s encryption standards, typically using asymmetric encryption (e.g., using public keys derived from the DIDs).
### Data Formats:
 - Messages are structured in JSON format, following the DIDComm v2 specification. A typical message contains:
   - **Headers**: Metadata about the message (e.g., type, timestamp, sender, recipient).
   - **Payload**: The actual content of the message, encrypted as needed.
   - **Encryption Details**: Information about the encryption used, such as keys and algorithms.
### Sequence of Exchanges:
 - **Mediator Acknowledgment**: The mediator responds with its own DID and a nonce, acknowledging the receipt of the initial message.
   - **Capability Negotiation**: Both parties may exchange additional messages to negotiate capabilities, such as supported encryption algorithms, message formats, and routing mechanisms.
   - **Routing Information**: The mediator may provide routing information necessary for forwarding future messages between the agents.
   - **Verification**: Each party verifies the identity of the other using cryptographic proofs, ensuring the authenticity and integrity of the exchanged DIDs and messages.
## 3. Finalization
### Successful Connection:
 - The handshake is finalized when both parties have successfully exchanged all necessary cryptographic information and have verified each other’s identities.
 - A final confirmation message may be sent by each party to indicate that the handshake process is complete.
### Error Handling:
 - **Error Detection**: If an error occurs (e.g., message tampering, failed verification, or timeout), the affected party sends an error message detailing the issue.
 - **Retries**: Depending on the error, the parties may retry the handshake process a specified number of times.
 - **Logging and Alerts**: Errors are logged for further investigation, and alerts may be triggered to notify administrators or developers of the failure.
 - **Fallback Mechanisms**: If the handshake fails repeatedly, fallback mechanisms (e.g., alternative communication channels) may be employed to establish a connection.