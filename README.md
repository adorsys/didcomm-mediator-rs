# didcomm-mediator-rs

A DIDComm mediator is a cloud agent that relays and routes messages between mobile agents. It is an essential component in the self-sovereign identity (SSI) ecosystem because it allows mobile agents to communicate with each other without being tied to centralized cloud infrastructures like Facebook, Signal, or Telegram. Unfortunately, mobile phones are not first-class citizens in web-based interactions. Therefore, messages sent by a mobile agent to another one must be routed and/or relayed through some sort of cloud agents, which are always available for web interaction (first-class citizen).

DIDComm mediators work by storing the DIDs (decentralized identifiers) of mobile agents. When a mobile agent wants to send a message to another mobile agent, it sends the message to the DIDComm mediator. The mediator then routes the message to the recipient's DIDComm mediator. The recipient's DIDComm mediator then delivers the message to the recipient's mobile agent.

The following diagram displays some cloud services that can be provided by a DIDComm mediator. In particular, services that take care of routing and relaying messages among mobile agents.

![sample cloud services](./mediator-server/docs/basic-arch.png)

Even though both (proxy and gateway) functionalities are specified in the same component in the DIDComm ecosystem, it is important to distinguish between the proxy and the gateway service. This distinction is made in the same way as the distinction between an SMTP server (which sends out emails) and an IMAP server (which delivers emails).

The data service can be considered a vital service to be provided to a mobile agent. Even though it is not fully aligned with the DIDComm messaging specification, displaying a data service here is essential so that we can see all the functionalities needed by a mobile agent to be complete.

Subscriptions will ensure a certain trust relationship between mobile agents and cloud services, and thereby serve as the foundation of the economic model on top of which cloud services are built.

A standardization of these cloud agents, in the same way email protocols were standardized, is essential for the success of the decentralized web.

## Self-Sovereign Identity (SSI)

* Self-Sovereign Identity (SSI) is a way for individuals to control their own digital identities.
* SSI uses decentralized identifiers (DIDs) and verifiable credentials (VCs) to allow individuals to share their identity information with others in a secure and privacy-preserving way.
* SSI has the potential to revolutionize the way that we interact with the internet, making it more secure, private, and user-centric.

## DIDComm Messaging

The DIDComm messaging specification can be found [here](https://identity.foundation/didcomm-messaging/spec/)

### Functionaly

* DIDComm is a messaging methodology that works with the decentralized identifier (DID) core spec to provide private, secure interaction between parties.
* DIDComm messaging is designed to be flexible and extensible, allowing for new protocols and applications to be built on top of it.
* The DIDComm specification is currently under development, but it has already been adopted by a number of projects, including the W3C Verifiable Credentials Working Group.
* DIDComm is a promising technology with the potential to revolutionize the way that we interact with the internet.

### Technicaly

* DIDComm messages are structured as JSON objects, and they can be exchanged over a variety of channels, including HTTP, WebSockets, and SMS.
* DIDComm messages can be used to send a variety of data, including requests, responses, notifications, and events.
* DIDComm messages are signed using the DID method, which ensures that they are tamper-proof and can be verified by the recipient.
* DIDComm is a secure and privacy-preserving messaging methodology that has the potential to be used in a wide variety of applications.

## Building and testing

To build and test the entire project, certain system packages are required especially for the **did-utils** and **oob-messages** crates. Both of these crates require **libssl-dev** and **pkg-config** to be installed on the system to build correctly. Without them, the build process will fail.

### Ubuntu

* **libssl-dev**: This package provides the development files for OpenSSL.
* **pkg-config**: This package provides a tool to help with the configuration of compiler and linker flags for libraries.  

You can install them using the following command:

```sh
sudo apt update
sudo apt install -y libssl-dev pkg-config
```

## Step-by-Step Guides

### Prerequisites
Ensure you have the following installed:
 * [Rust & Cargo](https://www.rust-lang.org/tools/install)

## Setup
1. Create a working directory eg(cd didcomm-mediator-rs) and cd into your directory.

 2. Clone the repository using the following command:
```sh
git clone https://github.com/adorsys/didcomm-mediator-rs.git
```

## Troubleshooting Tips
If you encounter any issues while running the application, here are some troubleshooting tips to help you resolve them
 
 ### Common Issues

   Build Errors:
  * Ensure that you have the required system packages installed. `libssl-dev` and `pkg-config` 
  * Ensure that you have the latest version of Rust and Cargo installed.
  * Check for any missing dependencies using 
```sh
cargo check
```

## Example
 Start the mediator service:

```sh
cd didcomm-mediator-rs/mediator
cargo run
```

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
