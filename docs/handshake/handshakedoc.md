# Detailed DIDComm Mediator-rs Handshake Documentation

## 1. Overview
The handshake process for the DIDComm Mediator-rs protocol establishes secure communication between two edge agents, mediated by a cloud agent. This process involves identity generation, Out-of-Band (OOB) connection establishment, message exchange, encryption, and final verification. The purpose of this document is to offer a detailed explanation of each step in the handshake flow, with special attention to OOB methods.

---

## 2. Handshake Flow

### 2.1 Communication Initiation

#### 2.1.1 Participant Identification
- **DID (Decentralized Identifier) Generation:**
  - **What happens:** Each edge agent generates its own DID. The DID represents a unique identifier associated with the agent and is crucial for decentralized communication. It enables agents to interact without a central authority, offering privacy and security benefits.
  - **Interaction with Mediator:** 
    - The mediator (cloud agent) is involved in this process because it plays a dual role: receiving messages on behalf of the edge agent and forwarding them to the appropriate recipient. This allows edge agents to remain lightweight and mobile, without needing to constantly manage their own communication channels.
  - **Why it's important:** The exchange of DIDs sets the foundation for all future communication, allowing agents to authenticate each other and manage message routing securely. Since the mediator handles message forwarding, it allows agents to communicate asynchronously without maintaining an always-online state.

- **DID Exchange via Out-of-Band (OOB) Message:**
  - **Out-of-Band Communication (OOB):**
    - **What it is:** OOB is a method of establishing a connection without a prior existing channel. In DIDComm, this typically involves an edge agent initiating contact by sharing a QR code containing its DID and service information.
    - **How it works:**
      1. One agent generates a QR code containing its DID and relevant metadata (such as service endpoints and encryption keys).
      2. The second agent scans the QR code using their mobile device or software agent.
      3. Once scanned, the receiving agent decodes the information and initiates the handshake by contacting the mediator and sending a message containing the scanned DID.
    - **Why it's important:** OOB messaging enables agents to establish a connection without requiring prior knowledge of each other’s network location or direct communication channels. It allows communication to be initiated even in offline environments where scanning is required for initial DID exchange.

#### 2.1.2 Initial Message
- **Handshake Message:**
  - **What happens:** Once DIDs are exchanged via OOB, the initiating edge agent sends the first handshake message to the mediator. This message typically contains the following elements:
    - **DID of the Initiating Edge Agent:** This identifies the sender.
    - **Nonce:** The nonce is a random value added to prevent replay attacks, ensuring that old messages cannot be reused maliciously.
    - **Service Information:** This includes details such as endpoint URLs, service types, and other metadata necessary to establish a communication channel.
  - **Why it's important:** This initial handshake message is the first formal communication between the agents, serving to introduce the initiating agent and establish a connection through the mediator. The nonce ensures that each handshake is unique and cannot be exploited by a malicious actor.

---

### 2.2 Message Exchanges

#### 2.2.1 Encryption Methods
- **Asymmetric Encryption:**
  - **What happens:** Messages exchanged between agents and the mediator are encrypted using asymmetric encryption, such as public-key cryptography.
  - **How it works:** Each agent has a pair of cryptographic keys: a public key and a private key. The public key is shared with others, while the private key remains secret. When one agent sends a message, it encrypts the message using the recipient’s public key. The recipient can then decrypt the message using their private key.
  - **Why it's important:** Asymmetric encryption ensures that only the intended recipient can read the message, even if the message is intercepted. This is critical for maintaining the privacy and security of communications in the DIDComm protocol.

#### 2.2.2 Data Formats
- **Message Structure (JSON):**
  - **What happens:** All DIDComm messages follow a standardized JSON structure that conforms to the DIDComm v2 specification. Each message contains three main components:
    - **Headers:** These include metadata about the message, such as its type (handshake, data exchange, error, etc.), timestamp, sender, and recipient.
    - **Payload:** This is the main body of the message. In the context of a handshake, the payload will contain details about the initiating agent, encryption keys, and service information. The payload is encrypted.
    - **Encryption Details:** This section includes the encryption keys, algorithms, and other cryptographic information needed for the recipient to decrypt the message.
  - **Why it's important:** Standardized message formats allow for consistent communication between diverse agents and mediators. JSON is a human-readable format, which also aids in debugging and understanding the message exchanges.

#### 2.2.3 Sequence of Exchanges
- **Mediator Acknowledgment:**
  - **What happens:** After receiving the initial handshake message, the mediator acknowledges the receipt by responding with its own DID and a nonce.
  - **Why it's important:** This acknowledgment serves two purposes: it confirms the mediator’s identity to the initiating agent, and it also ensures that the message was received correctly. The use of a nonce ensures message freshness and prevents replay attacks.
  
- **Capability Negotiation:**
  - **What happens:** Following the initial exchange, the two agents may exchange additional messages to negotiate capabilities such as:
    - Supported encryption algorithms (e.g., which public-key cryptosystems they will use),
    - Preferred message formats,
    - Routing mechanisms to be used by the mediator.
  - **Why it's important:** Capability negotiation ensures that both agents and the mediator can communicate efficiently and securely using mutually compatible methods. This step is crucial in heterogeneous environments where different agents may have different capabilities.

- **Routing Information:**
  - **What happens:** The mediator provides routing information that allows the agents to communicate through it. This may include IP addresses, domain names, and other network-related information that ensures messages are routed correctly.
  - **Why it's important:** Since the edge agents may be mobile or operating on constrained devices, they rely on the mediator to route messages between them. Routing information provided by the mediator is essential for ensuring seamless communication.

---

### 2.3 Finalization

#### 2.3.1 Successful Connection
- **What happens:** The handshake is finalized when both agents have exchanged all necessary cryptographic information, verified each other’s identities, and confirmed that the channel is secure.
- **Final Confirmation:** A final confirmation message is typically sent by each agent to acknowledge the successful establishment of the communication channel.
- **Why it's important:** The successful connection ensures that both agents can trust the communication channel and that any messages sent will be securely routed through the mediator.

---

## 3. Error Handling

### 3.1 Error Detection
- **What happens:** If an error occurs during the handshake process (e.g., due to message tampering, failed verification, or a timeout), the affected agent sends an error message back to the other party and the mediator.
- **Why it's important:** Error detection helps ensure that potential attacks or misconfigurations are quickly identified, allowing the agents to take corrective action or abort the handshake process.

### 3.2 Retries
- **What happens:** Depending on the nature of the error, the agents may retry the handshake process a specified number of times. Retries are limited to avoid potential denial-of-service attacks.
- **Why it's important:** Retry mechanisms provide resilience, ensuring that temporary failures (e.g., network issues) do not prevent the handshake from being completed.

### 3.3 Logging and Alerts
- **What happens:** Errors are logged by both the mediator and the agents. In the case of repeated failures, alerts may be triggered to notify administrators or developers.
- **Why it's important:** Logging and alerts provide valuable diagnostic information for troubleshooting and help prevent prolonged communication outages.

### 3.4 Fallback Mechanisms
- **What happens:** If the handshake fails after several retries, fallback mechanisms may be employed. These could involve alternative communication channels (e.g., switching to a different mediator or attempting direct communication).
- **Why it's important:** Fallback mechanisms ensure that agents can still establish a connection even in cases where the primary mediator is unreachable or malfunctioning.

---

## 4. Conclusion
This detailed documentation provides an in-depth explanation of the DIDComm Mediator-rs handshake processes found in the handshake flow. It outlines the technical mechanisms that enable secure communication between agents, focusing on Out-of-Band (OOB) methods, encryption, message formats, error handling, and fallback mechanisms.

