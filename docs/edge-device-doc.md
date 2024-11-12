# DIDComm Messaging for Edge Device Registration and Message Exchange
## Introduction
With the growth of IoT and connected devices, secure, private communication between devices like smart sensors, appliances, and any device has become essential. Decentralized Identifier Communication (DIDComm) provides a protocol that ensures private messaging, enabling devices to communicate without intermediaries directly accessing data. This makes DIDComm ideal for IoT networks where privacy, security, and decentralization are paramount.


![sample cloud services](work-flow.png)

### Benefits of DIDComm Messaging for Edge Devices
* **Enhanced Security:** Encryption protects message content, even when routed through intermediaries.
* **Privacy Preservation:** Only routing information is visible to intermediaries, keeping message content confidential.
* **Decentralization:** Devices control the communication flow without reliance on centralized systems.
* **Reliable Message Delivery:** Coordinated through a Mediator, communication remains effective even in complex networks.
This guide outlines the DIDComm messaging process, including participant roles, secure key exchange, and message flow, with an emphasis on privacy and security.

### Key Roles in DIDComm Messaging
 * **Mediator:**
The Mediator facilitates message routing between devices, handling tasks such as forwarding, error management, and key updates. This support allows devices to connect securely, even in networks with limited reachability.

### Edge Device 1
Edge Device 1 initiates communication with Edge Device 2. Before beginning the message exchange, it connects with the Mediator to obtain essential routing details, establishing a secure, private channel.

### Edge Device 2
Edge Device 2 is the intended recipient. However, it can also initiate communication, allowing for dynamic role interchangeability between devices.

## DIDComm Messaging Phases
The DIDComm process for edge device communication consists of three main phases:

* **Phase 1: Out-of-Band (OOB) Setup and Establishing Mediation**
Before DIDComm communication begins, Edge Device 1 and Edge Device 2 perform an Out-of-Band (OOB) setup to establish a secure channel. They exchange essential information, such as:

   * **Decentralized Identifiers (DIDs)** to represent each device.
   * **Routing Information** such as the Mediator’s routing DID.
    * **Secure Key Exchange:** During this setup, both devices securely exchange cryptographic keys, which will be used to encrypt all subsequent communication. This ensures that the initial link between the devices is both private and verifiable.
    The OOB setup establishes a trusted foundation, enabling Edge Device 1 and Edge Device 2 to communicate securely without the risk of exposing sensitive information.

    * **Mediation Request:**
    Edge Device 1 sends a mediation request to the Mediator, signaling its intent to use the Mediator’s services for secure message routing.

    * **Mediation Grant:**
    The Mediator grants the request, providing Edge Device 1 with a routing identifier (DID) and creating a private connection for secure messaging with Edge Device 2.

* **Phase 2: Message Exchange Preparation**
With the mediation relationship in place, Edge Device 1 prepares to send a secure message to Edge Device 2 by ensuring that cryptographic keys are up-to-date and properly aligned with the Mediator’s setup.

    * **Key Management:** Edge Device 1 reviews its encryption keys, updating them if necessary due to device identity changes or new recipient requirements. It then sends a keylist-update message to the Mediator, specifying any additions, removals, or changes to its encryption keys.

    * **Secure Key Exchange:** Edge Device 1 and the Mediator exchange and update any necessary public keys. This maintains an encrypted path for all messages, allowing the Mediator to route messages securely, without access to their content.

    With secure keys in place, Edge Device 1 can now initiate or respond to messages, confident in the privacy and security of the communication pathway.

* **Phase 3: Message Pickup and Delivery**
With routing and encryption set, Edge Device 1 is ready to exchange messages with Edge Device 2 via the Mediator. This phase involves requests, deliveries, and acknowledgments to ensure reliable message exchange.

    * **Message Pickup:** Edge Device 1 periodically checks with the Mediator for any incoming messages from Edge Device 2. This provides asynchronous communication, allowing messages to be retrieved as needed.

    * **Message Creation and Delivery:** Edge Device 1 encrypts a message for Edge Device 2, using DIDComm’s layered encryption, or “onion encryption,” for added security. It includes Edge Device 2’s routing DID (provided by the Mediator) as part of the routing information, specifying how the Mediator should forward the message.

    * **Message Forwarding by the Mediator:** Upon receiving the message, the Mediator unpacks only the outer “onion” layer to reveal routing details. This allows the Mediator to forward the message to Edge Device 2 without accessing its content, preserving privacy.

    * **Message Pickup by Edge Device 2:** The Mediator routes Edge Device 1’s message to Edge Device 2, which decrypts it using its private key. This ensures secure and confidential message retrieval.

    * **Acknowledgment of Receipt:** After decrypting and reading Edge Device 1’s message, Edge Device 2 sends an acknowledgment back through the Mediator. This confirmation helps Edge Device 1 verify that its message has successfully reached the intended recipient, adding reliability and traceability to the exchange.

### Security, Privacy, and Decentralization in DIDComm
DIDComm’s decentralized (a system without a central authority, where decision-making and control are distributed among multiple nodes, using peer-to-peer communication and decentralized identifier management) architecture ensures that communication remains private and secure, without relying on centralized intermediaries

* **Role Interchangeability:** Edge Device 1 and Edge Device 2 can dynamically shift between sender and recipient roles based on communication needs, enabling either device to initiate or respond to messages as situations evolve.
* **Privacy Preservation:** The Mediator only unpacks routing information, keeping the inner message content confidential. Each encryption layer reinforces privacy, allowing the Mediator to route messages securely.

### Conclusion
DIDComm messaging provides a robust framework for private, secure communication between devices. By coordinating through a Mediator, devices like Edge Device 1 and Edge Device 2 can reliably connect and exchange messages, even in complex network environments.
