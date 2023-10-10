# DIC (Dedicated Interaction Channel)

# Introduction

In the constantly changing world of network communication, ensuring security and control is of utmost importance. Conventional network systems frequently struggle with problems like unauthorized access, susceptibility to attacks, and a deficiency in giving users strong control over their data exchanges. In light of these challenges, we present **DIC** or **Dedicated Interaction Channels**.

DICs provide a new and intuitively secure way for network agents to interact with each other. DIC does not rediscover any wheel. It simplifies understanding of current idioms, The idea is as simple as:

- For **YOU** to talk to **ME**, I have to give you a **Dedicated Talk Line**.
- For **YOU** to send **ME** a mail, I have to give you a **Dedicated Mail Box**.
- If **YOU** use the **DIC** to **spam ME**, I **block** YOU.

This is nothing else than has been implemented out there by WhatsApp, Signal and other point to point messaging protocols.

Think of a DIC as **YOU** having an account on **MY Server**, giving **YOU** as **account holder** both the privilege and responsibility to engage in interactions with **MY Server**. You can login on that server and upload a message for me.

Further, DICs also give **YOU** the possibility to delegate sub-account to **HIM** another user, associating this sub-account with well defines subset of capabilities.

This unique features lets users customize their network interactions by choosing who can communicate with them, creating a structured yet adaptable networking environment.

# Terminology

## Edge vs. Cloud Agent

In the context of Self-Sovereign Identity (SSI), an "edge agent" and a "cloud agent" refer to two distinct components of the SSI ecosystem, each serving a specific role in managing and securing digital identities. Here's an explanation of each and the key distinctions between them:

### Edge Agent

An edge agent, also known as a user agent or personal agent, is a component of the SSI architecture that resides on the user's device or at the user's "edge." This edge can be a smartphone, computer, or any other personal computing device.

The primary function of an edge agent is to act on behalf of the user, managing their digital identity and credentials. It stores and controls the user's private keys, enabling them to create, manage, and share their verifiable credentials securely.

Users have complete control over their edge agent, ensuring that their identity and personal data remain under their ownership and consent. It aligns with the core SSI principle of user autonomy and self-sovereignty.

### Cloud Agent

A cloud agent, on the other hand, is a component of the SSI ecosystem that operates in a cloud or server environment. It typically provides additional services and functionalities to support the SSI infrastructure.

Cloud agents are responsible for tasks such as discovering other parties in the SSI network, facilitating communication and negotiation between different edge agents, and ensuring the availability and reliability of identity-related services.

Cloud agents may store decentralized identifiers (DIDs), public keys, and other information that can be accessed by edge agents as needed. However, crucial user-specific data and private keys remain on the user's edge agent for security and privacy reasons.

Cloud agents are instrumental in making SSI networks scalable by handling certain network-level functions and enabling interoperability between different edge agents.

We will be designing DIC by directly applying the use case of agent to agent interaction.

# Registration

In order to interact comfortably with other agents, an edge agend need one or more accounts with cloud agents. -**Outbox**: Edge agent will be able to deposit messages to cloud agent for forward to other agents, thus controlling which network components sees network metadata of the edge agent (privacy) -**Inbox**: Other agents can send messages to the edge agent, by forwarding those message to a receptioning cloud agent specified by the edge agent.

During the registration process, the **Cloud Agent** will give a dedicated interaction channel (DIC) to the edge agent.

Dedicated channel are important here, as - the **Outbox Cloud Agent** does not want to be spammed by any network component. Therefore, each message to the **Outbox** will be authenticated as such by the **Edge Agent DIC**

- the **Inbox Cloud Agent** would like to receive only messages authorized for reception by the edge agent. In order to authorize a message for reception, the **Edge Agent** will give a **Delegate Dedicated Interaction Channel (DDIC)** to the sending agent.

## Structure of a DIC

In order to design for performance, we are moving away from the world of databases toward the world of signed handles. This means I can give a DIC to a party by just storing the public key of that party in my database. But this approach is not scalable, as I will have to lookup the DIC whenever the party sends a message.

If instead of storing the public key, we just returning a **Signed Credential (VC)** to the edge agent, we can authenticate each incoming message by just **verifying the presentation (VP)** accompanying the message.

## Structure of a DDIC

In the same stream, we can allow a DIC holder to delegate some capabilities to another agent by producing a DDIC for that agent. A DDIC is a **Verifiable Credential (VC)** produced by the DIC holder, that contains the **Encrypted Original DIC**. The DDIC discloses no information on the content of the DIC, but allow the cloud agent to reception command from delegated agents.

## Sample Cloud Agent DID

A typical cloud agent DID would look like:

```json
{
  "@context": "https://www.w3.org/ns/did/v1",
  "id": "did:example:123456789abcdefghi",
  "verificationMethod": [
    {
      "controller": "did:example:123456789abcdefghi",
      "id": "did:example:123456789abcdefghi#keys-1",
      "privateKeyMultibase": "z6cV1FRHHiLwFFXLZmC1eFVshyo4V2UX5bgCfyXaVaXhK",
      "type": "Ed25519VerificationKey2018"
    },
    {
      "controller": "did:example:123456789abcdefghi",
      "id": "did:example:123456789abcdefghi#keys-2",
      "privateKeyMultibase": "z998dnQCyxJV8wFxCva14T6S7TUZhzVzzwn4FymntyLMA",
      "type": "X25519KeyAgreementKey2019"
    }
  ],
  "assertionMethod": ["did:example:123456789abcdefghi#keys-1"],
  "authentication": ["did:example:123456789abcdefghi#keys-1"],
  "keyAgreement": ["did:example:123456789abcdefghi#keys-2"],
  "service": [
    {
      "id": "did:example:123456789abcdefghi#keys-1",
      "serviceEndpoint": "http://mediators-r-us.com/mediate",
      "type": "mediate"
    },
    {
      "id": "did:example:123456789abcdefghi#keys-2",
      "serviceEndpoint": "http://mediators-r-us.com/send",
      "type": "send"
    },
    {
      "id": "did:example:123456789abcdefghi#keys-2",
      "serviceEndpoint": "http://mediators-r-us.com/receive",
      "type": "receive"
    },
    {
      "id": "did:example:123456789abcdefghi#keys-2",
      "serviceEndpoint": "http://mediators-r-us.com/admin",
      "type": "admin"
    }
  ]
}
```

The "service" entry is the most critical part in the context of mediation, the "service" entry allows an edge agent to register for a mediation service offered by the cloud agent. When an edge agent registers for this mediation service, the cloud agent assumes the role of receiving, storing, and forwarding messages on behalf of the registered edge agent. This functionality is pivotal for cases where an edge agent may not be available or reachable at all times. The cloud agent steps in as a reliable intermediary, ensuring seamless communication and coordination within the SSI network.

In the DIDComm ecosystem, registration of edge agents with cloud agents is known under **Route Coordination** or **Mediation Coordination**.

## Mediation Request

A sample document can be found [here](https://github.com/hyperledger/aries-rfcs/tree/main/features/0211-route-coordination#mediation-request), containing this sample doc:

```json
{
  "@id": "1234567816746574234",
  "@type": "<baseuri>/mediate-request"
}
```

For the purpose of DIC, this request needs:

**Anti Spam**

We can only protect a server providing such a service by implementing some sort of anti spam mechanism. This will be either of:

- a prepayment mechanism, ensuring that call is depositing some valuable guaranty of behaving honest (cryptocurrency, ...), or
- some sort of verifiable credential prooving preauthentication by some sort of authority.

**Interaction Key**
The sender of this request must also provide a public key he will be using to produce authenticating/asserting presentations

**Confidentiality**
Response to this message will contain information that is destinated solely to the sender of the request. In this case, request must also contain a public key that can be use for encryption key exchange.

For authentication, assertion and key exchange, a DID is provide all essential component. Therefore, adding a appropriate DID of the sender to the request will be sufficient in that case.

The resulting request would look like:

```json
{
  "@id": "1234567816746574234",
  "@type": "<baseuri>/mediate-request",
  // any did method could work.
  "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "services": ["inbox", "outbox"],
  "anti-spam": "any vp here"
}
```

A sample anti spam might look like this:

```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2"],
  "type": ["VerifiablePresentation"],
  "id": "http://example.edu/credentials/3732",
  "holder": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "verifiableCredential": [
    {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://www.dial.com/ns/crypto-checks/v1"
      ],
      "type": ["VerifiableCredential", "Crypto-Check"],
      "credentialSubject": {
        "id": "did:ethr:0xb9c5714089478a327f09197987f16f9e5d936e8a",
        "amount": {
          "value": 200,
          "currency": "EUR"
        }
      },
      "id": "https://www.dial.com//37325264562435234",
      "issuer": "did:key#z7dNyxjs9BUfsbX11VG4BGDMB3Wg1Pq2NqhSwTBT8UuRC",
      "validFrom": "2023-03-05T19:23:24Z",
      "validUntil": "2023-12-31T19:23:24Z",
      "proof": ["issuer proof here"]
    }
  ],
  "proof": ["presenter proof here"]
}
```

Another anti spam can be a simple email:

```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2"],
  "type": ["VerifiablePresentation"],
  "id": "http://example.edu/credentials/3732",
  "holder": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "verifiableCredential": [
    {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://www.dial.com/ns/a-spam/v1"
      ],
      "type": ["VerifiableCredential", "A-Spam"],
      "credentialSubject": {
        "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#official",
        "e-mail": "abc@mail.com"
      },
      "id": "https://www.dial.com//37325264562435234",
      "issuer": "did:key#z7dNyxjs9BUfsbX11VG4BGDMB3Wg1Pq2NqhSwTBT8UuRC",
      "proof": ["issuer proof"]
    }
  ],
  "proof": ["presenter proof"]
}
```

## Mediation Grant

The positive response of a mediation request is a mediation grant. It is generally associated with a file like:

```json
{
  "@id": "1234567816746574234",
  "@type": "<baseuri>/mediate-grant",
  "endpoint": "http://mediators-r-us.com/.wellknown/did.json",
  "dic": [
    {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://www.dial.com/ns/a-spam/v1"
      ],
      "type": ["VerifiableCredential", "Inbox-Channel"],
      "credentialSubject": {
        "id": "did of edge agent",
        "service-level": "gold"
      },
      "id": "https://www.dial.com//37325264562435234asdfas",
      "issuer": "did of cloud agent",
      "proof": ["proof of cloud agent"]
    },
    {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://www.dial.com/ns/a-spam/v1"
      ],
      "type": ["VerifiableCredential", "Outbox-Channel"],
      "credentialSubject": {
        "id": "did of edge agent",
        "service-level": "gold"
      },
      "id": "https://www.dial.com//37325264562435234asdfas",
      "issuer": "did of cloud agent",
      "proof": ["proof of cloud agent"]
    }
  ]
}
```

Whenever the edge agent uses this key to encrypt a communication, the cloud agent will know that the message is from this very edge agent. The routing key entry has following components:

## Mediation Deny

A deny response will look like:

```json
{
  "@id": "1234567816746574234",
  "@type": "<baseuri>/mediate-deny"
}
```

## Discard DDIC

An edge agent can issuer a DDIC to the contact, so that the contact deposits a message on behalf of the edge agent. In some cases before the DDIC expires, the issuer might want to discard it. In that case, the issuer send a discard message to the cloud agent, with the DDIC to discard.
