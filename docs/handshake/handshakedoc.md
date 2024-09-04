# DIDComm Mediator-rs Handshake Documentation
### 1. Overview
The DIDComm Mediator-rs handshake process is designed to establish secure communication between edge agents via a mediator (cloud agent). This documentation outlines the steps, message formats, and error-handling mechanisms involved in the handshake flow.
### 2. Handshake Flow
### 2.1 Communication Initiation
#### 2.1.1 Participant Identification
- DID Generation:
  - Each edge agent generates a Decentralized Identifier (DID). This process typically involves interaction with a mediator (cloud agent), which plays a crucial role in:
  - **Receiving Messages**: The mediator receives messages addressed to the edge agent.
  - **Forwarding Messages**: The mediator forwards messages sent out by the edge agent.
- DID Exchange:
  - Agents exchange their DIDs to identify each other. This exchange typically occurs within the context of an Out-of-Band (OOB) message, often initiated by one edge agent scanning the QR code of another.
#### 2.1.2 Initial Message
- Initiating Message:
  - The initiating edge agent sends a DIDComm handshake message to the mediator. This message includes the following components:
  - **DID of the Initiating Agent**: Identifies the sender.
  - **Nonce**: Ensures message freshness and prevents replay attacks.
  - **Service Information**: May include endpoint URLs, service types, and other metadata relevant to the communication.
### 2.2 Message Exchanges
#### 2.2.1 Encryption Methods
- Encryption Standards:
  - Messages are encrypted following DIDComm's encryption standards, typically using asymmetric encryption, such as public keys derived from the DIDs.
#### 2.2.2 Data Formats
  - Message Structure:
    - Messages are structured in JSON format, conforming to the DIDComm v2 specification. Each message includes:
    - **Headers**: Metadata about the message, such as type, timestamp, sender, and recipient.
    - **Payload**: The content of the message, encrypted as needed.
    - **Encryption Details**: Information about the encryption used, including keys and algorithms.
#### 2.2.3 Sequence of Exchanges
- Mediator Acknowledgment:
  - The mediator responds with its own DID and a nonce, acknowledging receipt of the initial message.
- Capability Negotiation:
  - Additional messages may be exchanged to negotiate capabilities, including supported encryption algorithms, message formats, and routing mechanisms.
- Routing Information:
  - The mediator provides routing information necessary for forwarding future messages between the agents.
### 2.3 Finalization
#### 2.3.1 Successful Connection
- Final Confirmation:
  - The handshake is finalized when both parties have successfully exchanged all necessary cryptographic information and verified each other’s identities.
  - A final confirmation message may be sent by each party to confirm the completion of the handshake process.
### 3. Error Handling
### 3.1 Error Detection
- Issue Identification:
  - If an error occurs, such as message tampering, failed verification, or timeout, the affected party sends an error message detailing the issue.
### 3.2 Retries
- Retry Mechanism:
  - Depending on the error, the parties may retry the handshake process a specified number of times.
### 3.3 Logging and Alerts
- Error Logging:
  - Errors are logged for further investigation.
- Notifications:
  - Alerts may be triggered to notify administrators or developers of the failure.
### 3.4 Fallback Mechanisms
- Alternative Channels:
  - If the handshake fails repeatedly, fallback mechanisms, such as alternative communication channels, may be employed to establish a connection.