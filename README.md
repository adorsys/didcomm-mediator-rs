# DIDComm-Mediator

[![CI](https://github.com/adorsys/didcomm-mediator-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/adorsys/didcomm-mediator-rs/actions/workflows/rust.yml)
[![DIDComm Messaging Clients Utilities](https://img.shields.io/badge/DIDComm%20Messaging%20Clients%20Utilities-repo-blue.svg)](https://github.com/adorsys/didcomm-messaging-clients-utilities)
[![DIDComm Messaging Spec](https://img.shields.io/badge/DIDComm%20Messaging-Specification-blue.svg)](https://identity.foundation/didcomm-messaging/spec/)
[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)](https://github.com/adorsys/didcomm-mediator-rs/blob/main/LICENSE)

A DIDComm mediator is a cloud agent that facilitates secure, decentralized communication between mobile agents in the self-sovereign identity (SSI) ecosystem. This mediator acts as a cloud-based intermediary, enabling mobile agents to communicate without reliance on centralized infrastructures like Facebook, Signal, or Telegram.

For further understanding checkout the [docs](docs/mediator-doc.md)  

## Project Feature Implementation Tracker

|Message types  | Spec Status | Implementation Status|
|---------------|-------------|----------------------|
|[Mediator Coordination Protocol](https://didcomm.org/coordinate-mediation/2.0/)| ADOPTED| ✅|
|[Pickup Protocol](https://didcomm.org/messagepickup/3.0/)| ADOPTED | ✅ |
|[DID Rotation](https://didcomm.org/book/v2/didrotation) | ACCEPTED | ✅ |
|[Cross-Domain Messaging/ Routing Protocol](https://identity.foundation/didcomm-messaging/spec/#routing-protocol-20) | ADOPTED | ✅|
|[Trust Ping Ptotocol](https://identity.foundation/didcomm-messaging/spec/#trust-ping-protocol-20) | ADOPTED|✅|
|[Discover Features Protocol](https://didcomm.org/discover-features/2.0/) | ADOPTED | ✅ |
|[Out of band Messaging](https://identity.foundation/didcomm-messaging/spec/#out-of-band-messages) | ADOPTED | ✅|
|[Basic Message Protocol](https://didcomm.org/basicmessage/2.0/#:~:text=The%20BasicMessage%20protocol%20describes%20a,message%20type%20used%20to%20communicate.) | ADOPTED|⚪|
|[Acks](https://github.com/hyperledger/aries-rfcs/tree/main/features/0015-acks)| ADOPTED |❌ |
|[Present Proof Protocol](https://didcomm.org/present-proof/3.0/)| ADOPTED | ❌|

## Building and testing

To build and test the project, ensure that **libssl-dev** and **pkg-config** are installed on your system.

### Ubuntu Setup

Install the necessary packages with:

```sh
sudo apt update
sudo apt install -y libssl-dev pkg-config
```

### Prerequisites

Ensure you have [Rust & Cargo](https://www.rust-lang.org/tools/install) installed.

## Setup

1. Optionally, create a working directory and navigate into it.

2. Clone the repository:

```sh
git clone git@github.com:adorsys/didcomm-mediator-rs.git
```

## Running the Project

### Environmental variables

You need to create a **`.env`** file in the root directory of the project. Take a look at the [.env.example](.env.example) file for more information about the needed variables.

### Running with Docker Compose

The easiest way to run the project is with [Docker Compose](https://docs.docker.com/compose/).

Run the following command:

```sh
docker-compose up --build
```

This will build the docker image of the mediator server with all its prerequisites and run it.

### Running with Cargo

#### MongoDB

This project uses MongoDB as the database. You need to have [MongoDB](https://www.mongodb.com) installed and running on your system.

Another option is to use [Docker](https://www.docker.com):

```sh
docker pull mongo
docker run --name mongodb -d mongo
```

#### AWS Secrets Manager

The server uses AWS Secrets Manager for storing the mediator secrets. You need to configure AWS credentials in your environment. For tests you can use [LocalStack](https://localstack.cloud/) as a local AWS service.

> check the `.env.example` file for needed variables

You can now start the mediator server:

```sh
cargo run
```

The output should look like this:

![image](docs/server-output.webp)

## License

This project is licensed under the Apache License. See the [LICENSE](https://github.com/adorsys/didcomm-mediator-rs/blob/main/LICENSE) file for details.
