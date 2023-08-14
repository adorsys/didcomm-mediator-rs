# Architecture Documentation


A DIDComm mediator is a cloud agent that relays and routes messages between mobile agents. It is an essential component in the self-sovereign identity (SSI) ecosystem because it allows mobile agents to communicate with each other without being tied to centralized cloud infrastructures like Facebook, Signal, or Telegram. Unfortunately, mobile phones are not first-class citizens in web-based interactions. Therefore, messages sent by a mobile agent to another one must be routed and/or relayed through some sort of cloud agents, which are always available for web interaction (first-class citizen).

DIDComm mediators work by storing the DIDs (decentralized identifiers) of mobile agents. When a mobile agent wants to send a message to another mobile agent, it sends the message to the DIDComm mediator. The mediator then routes the message to the recipient's DIDComm mediator. The recipient's DIDComm mediator then delivers the message to the recipient's mobile agent.

The following diagram displays some cloud services that can be provided by a DIDComm mediator. In particular, services that take care of routing and relaying messages among mobile agents.

![sample cloud services](../docs/basic-arch.png)

# Entities

## Message Endpoints
Message enpoints are entities at the beginning or end of a message transmission process. These are:
- The Message Sender
- The Message Recipient

## Mediators
A mediator is an entity that plays a role inside a message transmission chain. This can be:
- The Sender Agent
- The Recipient Agent
Futher, we might witness the design of privacy oriented agent types like privacy-relay agents or even some time stamping relay types that perform neutra documentation of message delivery between sender and receiver.

# Roles

## Message Sender
This is the entity that is entending to send out a message.

### Security Constraint: Identity Correlation
* The identity of the sender shall not be unintentionaly correlatable among multiple receivers. This means the sender shall be able to produce a different connection identifier for each recipient.
* Beside the sender agent, no other entity shall be able to discover network properties of the sender's mobile agent (like IP addresses, protocol used to deliver message to sender agent, ...)
* Many sender agents shall not be able to correlate messages resulting from a single sender (except in network contrained cases like with ip address correlation).
### Security Constraint: Content Integrity
* The recipient shall be able to verify that the message is from the intended sender. This means that the recipient needs to know a public key solely controlled by the sender.

## Message Receiver
This is the entity that is intending to receive a message
### Security Constraint: Identity Correlation
* The identity of the receiver shall not be unintentionaly correlatable among multiple senders. This means the receiver shall be able to produce a different connection identifier for each sender.
* Beside the sender agent, no other entity shall be able to discover network properties of the receiver's mobile agent (like IP addresses, protocol used to deliver message to sender agent, ...)
### Security Constraint: Content Secrecy
* Beside the recipient, none of the intermediary agents shall be able to read the content of the message produced by the sender for the recipient. This means that the sender has to know a public key solely controlled by the recipient.

## Sender Agent
This is the agent that collect message produced by a sender, with the purpose of relaying them to other cloud agents.
### Security Constraint: Auth, DoS
A sender agent must be able to identify a sender requesting the forward of a message. This can e.g happen by having the sender sign the forward request with a key known to the sender agent.
### Security Constraint: Correlation
A network observer shall not be able to correlate all message associated with the identity of a sender at the sender agent. This means that a sender shall be able to encrypt the envelope with a general public key provided by the sender agent.

## Recipient Agent
This is the agent that stores the message for final delivery to the recipient.
### Security Constraint: Auth, DoS
* A recipient agent must be able to identify the recipient for which message is being received. This can e.g happen by having the sender encrypt the request with a key known to the recipient agent.
* A recipient agent shall be able to determine, that the recipient authorized transmission of this message. This means we will need some sort of __recipient auth__ known to sender and recipient agent.
* A recipient agent shall be able to discard messages originating from a given recipient auth.
### Security Constraint: Correlation
Beside the recipient agent, sender angent and other random network observers shall not be able to correlate messages associated with the identity of a sender. This means that a sender shall be able to encrypt the envelope with a general public key provided by the recipient agent.

# Sample DID
```json
{
    "issuanceDate": "2020-12-16T11:00:50.268126Z",
    "COMMENT-ON-issuer":"This is the uid of this channel end. Presented here as issuer because the document is self signed.",
    "issuer": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I",
    "type": [
        "VerifiableCredential",
        "DIDDocument"
    ],
    "COMMENT-ON-proof":"For each signature key, document owner provided a self signature of this document.",
    "proof": [
        {
            "issuer": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I",
            "created": "2020-12-16T11:00:50.268126Z",
            "proofPurpose": [
                "authentication"
            ],
            "COMMENT-ON-JcsBase64Ed25519Signature2020":"This is our signature specification.",
            "type": "JcsBase64Ed25519Signature2020",
            "verificationMethod": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I#key-Ed25519-0",
            "signatureValue": "gehfhbgVIDTVj001ImEQ2Rg2g2de1dnEkqUMfpe08UYpJwuIftUaDFHEV8BiEuCV8YHFLgwwqxxKeF4gGSKDCw",
            "nonce": "43f84868-0632-4471-b6dd-d63fa12c21f6"
        },
        {
            "issuer": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I",
            "created": "2020-12-16T11:00:50.268126Z",
            "proofPurpose": [
                "authentication"
            ],
            "type": "JcsBase64Ed25519Signature2020",
            "verificationMethod": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I#key-Ed25519-1",
            "signatureValue": "yCXhjprHQaxFf7XmrYX0Q775t_6sTqbqDlm8JAdFN3mlt1pWJ4WQIoJAka3hcQDJCTV6GwprjluuiIyroO-xBQ",
            "nonce": "96c1eaf5-c488-4163-9202-febc2cbdb0e8"
        }
    ],
    "credentialSubject": {
        "id": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I",
        "type": "DIDDocument",
        "COMMENT-ON-did":"This is the generated DID Document",
        "did": {
            "id": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I",
            "authentication": [
                {
                    "id": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I#key-Ed25519-0",
                    "publicKeyBase58": "GQrrQYhAwZMaL4Nh3dX8zQmuPUDjogYcDKQxVAaxhCDj",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I"
                },
                {
                    "id": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I#key-Ed25519-1",
                    "publicKeyBase58": "3LtW9rpowyKbe3XAs83dMEGwkqFr8kitoEXm3WQi2rWR",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I"
                }
            ],
            "COMMENT-ON-assertionMethod":"These are signature keys. Just referencing authentication keys.",
            "assertionMethod": [
                "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I#key-Ed25519-0",
                "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I#key-Ed25519-1"
            ],
            "COMMENT-ON-keyAgreement":"These are encryption keys. Can be used to encrypt messages sent to subject.",
            "keyAgreement": [
                {
                    "id": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I#key-X25519-0",
                    "publicKeyBase58": "BM47QQFMLShAFNV6B5vCWov5WXSU6Dnpp89WG16toCVb",
                    "type": "X25519KeyAgreementKey2019",
                    "controller": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I"
                },
                {
                    "id": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I#key-X25519-1",
                    "publicKeyBase58": "GYtfGchVBbvhtuGhZo9qbeykM7Wv7qwFnVWnGmutqsWR",
                    "type": "X25519KeyAgreementKey2019",
                    "controller": "did:sw:ep:MBSO-DVB3-JCSR-I3BZ-HDQI-CXRP-FNSL-I"
                }
            ],
            "service": [{
                "id":"did:gateway:3VCK-B6VQ-IFLQ-Q6TC-T243-6PJH-SYIA#linked-domain",
                "type": "LinkedDomains", 
                "serviceEndpoint": "https://gateway.mediator-rs.com"
            }],            
            "@context": "https://www.w3.org/ns/did/v1"
        },
        "COMMENT-ON-udf":"Just to avoid referencing the same document many times. A counter signature can just include this udf instead of the whole document.",
        "udf": "MCHD-D77I-3VCK-B6VQ-IFLQ-Q6TC-T243-6PJH-SYIA-4QFU-SNRE-LXM7-BX6P-M4IV-QL4U-ABA2-2T7J-MMTG-HN47-TY4T-6N75-C6JW-UTHN-3YAW-2BWC-YJI"
    },
    "@context": [
        "https://www.w3.org/2018/credentials/v1",
        "https://secure-wallet.org/vc/v1"
    ]
}
```
# Workflows


## Recipient Registration
In the registration process, we want message recipient to register with a recipient agent.

The recipient agent must be able to:
- Identify that an incommning message is assigned to a registered recipient.
- Receive and store that message for delivery to the target recipient.

The design of the recipient identifier shall nevertheless prevent correlation of recipient identifier by multiple senders.

## Scalability


Even though both (proxy and gateway) functionalities are specified in the same component in the DIDComm ecosystem, it is important to distinguish between the proxy and the gateway service. This distinction is made in the same way as the distinction between an SMTP server (which sends out emails) and an IMAP server (which delivers emails).

The data service can be considered a vital service to be provided to a mobile agent. Even though it is not fully aligned with the DIDComm messaging specification, displaying a data service here is essential so that we can see all the functionalities needed by a mobile agent to be complete.

Subscriptions will ensure a certain trust relationship between mobile agents and cloud services, and thereby serve as the foundation of the economic model on top of which cloud services are built.

A standardization of these cloud agents, in the same way email protocols were standardized, is essential for the success of the decentralized web.

In this request, the recipient provides an extended public key. 
This extended public is signed by the recipient as a proof of ownership of the private key.
- This is the identity key of the recipient.
- This extended public key is sent to the mediator, 
- The mediator returns a signature of the identity key of the recipient.
- The mediator does not store any additional info on the receiver.

For each contact, 
- the recipient will derive a separate peer did public keys from the extended public key.
- the recipient uses the public key of the mediator to encrypt:
  - the identity public key of the recipient
  - the mediator signature of the identity public key of the recipient
  - the hd path of the peer did public key of the contact
- This encrypted object is the auth token of the sender.

For a forward request, the sender will :
- add the auth token to the message
- encrypt/sign the message with the public key of the mediator

In order to process the forward request
- decrypt verify the forward message,
- decrypt the auth token and 
  - verify that the ginature associate with the identity key of the reciver
  - use the hd path to derive the public key assigned by the receiver to the contact
  - verify that this public key is use to encrypt/sign the inner message
  - store the message for pickup by the receiver.
  - evetl. notify the reciver for available message.
