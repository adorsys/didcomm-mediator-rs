# Contact Exchange
The purpose of this document iis to detail how to proceed with the contact exchange between tow peers. This document will span many standards:

* **Out of Band**: The purpose of the [out of band protocol](https://github.com/hyperledger/aries-rfcs/blob/main/features/0434-outofband/README.md) is to allow Bob to discover Alice.
* **Contact Exchange**: The purpose of the [contact exchange protocol](https://github.com/hyperledger/aries-rfcs/blob/main/features/0023-did-exchange/README.md) is to allow Alice and Bob to exchange permanent credentials.
* **Pickup Protocol**: The purpose of the [contact exchange protocol](https://github.com/hyperledger/aries-rfcs/blob/main/features/0212-pickup/README.md) is to allow alice-edge-agent to pickup messages depositer for alice at alice-mediator

## Using Edge Agents
Bob and Alice will be using edge agents to access the network.
* **Bob Agent**: refered to in this document as ```bob-edge-agent```, is the application used by Bob to access the network.
* **Alice Agent**: refered to in this document as ```alice-edge-agent```, is the application used by Alice to access the network.

## Relaying thru Cloud Agents
Mediation is essential, as we do not want ```alice-edge-agent``` and ```bob-edge-agent``` to expose their network meta data to untrusted cloud agents.

In this situation we will have two cloud agents:
* **Alice Mediator**: refered to in this document as ```alice-mediator```, is the component relaying all messages sent by Alice to the network or received by Alice from the network.
* **Bob Mediator**: known as ```bob-mediator```, is the component relaying all messages sent by Bob to the network or received by Bob from the network.

In this case, we assume that the services of the cloud agent at ```alice-mediator.com``` has a web site, sign boards and oder prints that display the public QRCode that can be scanned by the edge agent app. 

## Registering with a Mediator
befor Alice and Bob use mediators, the have to subscribe to services of those respective mediators. The information each of them need for subscribtion the did of the mediator. In the Alice case, ```did:web:alice-mediator.com:alice_mediator_pub``` and respectively in the Bob's case ```did:web:bob-mediator.com:bob_mediator_pub```. These URL service many purposes:

* **Domain Name Resolution**: the mediator did allow us to resolve the domain of the medaitor at ```alice-mediator.com```
* **DID Resolution**: under ```https://alice-mediator.com/alice_mediator_pub/did.json```, we can find the mediator did document. But this did document does not contain any proof of integrity.
* **Integrity Protected Did Document**: under ```https://alice-mediator.com/alice_mediator_pub/did/pop.json?challenge=aasdfasdfadsf```, we can request an authenticated did document and the mediator will return a freshly produced presentation signed by the public key ```alice_mediator_pub``` present in the URL. Recall that content adresseing the did document will prevent the mediator from rotating keys, as any modification to the document would change the document address and thus the URL. Recall we need a verification of the did of this mediator, because we else do not have control over the dns infrastructure between alice_edge_agent and the alice_mediator.

### Mediation Request
After 
* reading the QRCode of alice-mediator, 
* loading and verifying the authenticity of the did of alice-mediator,
* alice-edge-agent can send a mediation request to alice-mediator,

The mediation request will look like:

```json
{
    "@id": "id_alice_mediation_request",
    "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-request",
    "did": "did:key:alice_identity_pub@alice_mediator",
    "services": ["inbox", "outbox"],
    "antiSpam": "anti spam token"
}
```

### Mediation Grant
Alice gets a positive response (mediation grant). Recall that a mediation grant looks like:
```json
{
  "@id": "id_alice_mediation_grant",
  "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-grant",
  "endpoint": "https://alice-mediators.com",
  "dic": [
    {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://www.dial.com/ns/a-spam/v1"
      ],
      "type": ["VerifiableCredential", "InboxChannel"],
      "id": "https://alice-mediator.com/id_alice_inbox_channel",
      "issuer": "did:web:alice-mediator.com:alice_mediator_pub",
      "credentialSubject": {
        "id": "did:key:alice_identity_pub@alice_mediator",
        "service-level": "gold"
      },
      "proof": ["proof from alice_mediator_pub"]
    },
    {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://www.dial.com/ns/a-spam/v1"
      ],
      "type": ["VerifiableCredential", "Outbox-Channel"],
      "id": "https://alice-mediator.com/id_alice_outbox_channel",
      "issuer": "did:web:alice-mediator.com:alice_mediator_pub",
      "credentialSubject": {
        "id": "did:key:alice_identity_pub@alice_mediator",
        "service-level": "gold"
      },
      "proof": ["proof from alice_mediator_pub"]
    },
  ]
}
```

Remember that the DIC can be an opaque string, solely readable to the issuer, in this case alice-mediator. In this case the mediation grant file will look like:

```json
{
  "@id": "id_alice_mediation_grant",
  "@type": "https://didcomm.org/coordinate-mediation/2.0/mediate-grant",
  "endpoint": "http://alice-mediators.com",
  "dic": ["outbox:alice_out_opaque_dic", "inbox:alice_in_opaque_dic"]
}
```
Now Alice can send/receive all her future network requests through alice-mediator.com and stay private to the decentralized SSI network. Recall that Alice might decide to enroll with many mediators and therefore reduce dependency to a single mediator.

* **Dedicated Interaction Channels**
After Alice and Bob enrollment with mediators,
  * ```alice_out_opaque_dic``` is used to authenticate requests sent by Alice to alice-mediator.com
  * ```alice_in_opaque_dic``` is used to authenticate requests sent by Alice's contacts to alice-mediator.com for delivery to Alice.
  * ```bob_out_opaque_dic``` is used to authenticate requests sent by Bob to bob-mediator.com
  * ```bob_in_opaque_dic``` is used to authenticate requests sent by Bob's contacts to bob-mediator.com for delivery to Alice.
  
We will be using the opaque representation to reduce verbosity bellow.

Sample services provided by a mediator are:
* **DID Resolution Requests**: alice-mediator will expose an interface for alice-edge-agent to resolve other DIDs. This is critical, as
  * alice-edge-agent resolving DID thru random servers on the internet will be exposing alice-edge-agent network metadata to those servers.
  * nevertheless, alice-medaitor.com as a central point of resolution can now actively interfere into all alice-edge-agent communication. 
  * we solve this problem by (1) integrity protection of resolved DIDs, (2) end to end ecrypt messages, (3) allowing alice-edge-agent to enroll with many mediators.
* **Outgoing Messages (Outbox)**: Alice will also be depositing her outgoing messages with alice-mediator.com for forward to other agents, 
  * preventing random mediators from discovering alice-edge-agent's network metadata,
  * preventing common errors like unresponsive mediators, ...
  * alice-mediator will be using alice_out_opaque_dic to authenticate all request sent by alice-edge-agent.
* **Incomming Messages (Inbox)**: Alice will have the alice-mediator receive and store messages sent by Bob. For this purpose, alice-edge-agent will issuer a DICs named ```bob_2_alice_in_opaque_dic```, that will be presented to alice-mediator with each delivery request from Bob (precisely bob-mediator) to alice-mediator.
* **Short URL Storage & Resolution Services**: alice-mediator.com can allow Alice to store (resp. resolve) short URLs for retrival by Alice's contacts. As alice-mediator is an internet service trusted by Alice.
  * recall that we can integrity protect those short URLs by having them IPFS content addressed.
  * a short url will perconsequence look like: ```https://mediator.com/short/QmZtmD2qt6fJot32nabSP3CUjicnypEBz7bHVDhPQt9aAy```, where ```QmZtmD2qt6fJot32nabSP3CUjicnypEBz7bHVDhPQt9aAy``` is the IPFS multihash of the file. 
  * in most cases, the file cached will be encrypted for the recipient. A jwk needed to decrypt the file will then be available as separate qrcode as well.
    ```json
    {
      
      "kty": "OKP",
      "crv": "Ed25519",
      "d": "private_key_of_recipient",
      "x": "public_key_of_recipient"
    }
    ```

### Contact Exchange : Out of Band Invitation
Now that Alice has a relationship with alice-mediator, Alice can generate DIDs for exchange with Bob. In order for Alice to connect with Bob, she wants to start with an out of band message that can be scanned by Bob.

Beside a mediation-request that need a special type of spam protection, every other request flowing thru the system will have to be spam protected with authorization attributes. Even for an initial OoB-invitation, alice will have to generate a temporal keypair to be sent to Bob, and then use it to Bin the authorization token (DIC) issued to Bob.

```json
{
  "kty": "OKP",
  "crv": "Ed25519",
  "d": "bob_invitation_priv_produced_by_alice",
  "x": "bob_invitation_pub_produced_by_alice"
}
```

This jwk will also be represented by the did ```did:key:bob_invitation_pub_produced_by_alice``` and be available for scanning by Bob.

In order to allow Bob to be able to deposit a reply at alice-mediator, Alice needs to produce an authorization (DIC) used by Bob to access alice-mediator. The DIC will have the form:

```json
{
  "delegate": "did:key:bob_invitation_pub_produced_by_alice",
  "dic": "alice_in_opaque_dic",
  "proof":["proof from alice_identity_pub@alice_mediator - see did core capability delegation"]
}
```

This is a wrapper arround the original DIC named ```alice_in_opaque_dic``` issued by ```alice-mediator```. It is some sort of verifiable credential, that embeds another verifiable credential for delegation. The DIC will be enccrypted using a key exchange from ```"issuer":"did:web:alice-mediator.com:alice_mediator_pub"```. The final result will be a jwe called ```bob_2_alice_in_opaque_dic_temp``` for _Bob to Alice dedicated interaction channel_.

The invitation document to be presented in OoB-form by Alice to Bob will look like:

```json
{
    "@type": "https://didcomm.org/connections/1.0/invitation",
    "@id": "id_alice_oob_2_bob",
    "label": "Alice",
    "recipientKeys": ["did:key:alice_identity_pub@bob"],
    "serviceEndpoint": "https://alice-mediator.com/inbox",
    "routingKeys": ["did:web:alice-mediator.com:alice_mediator_pub","did:key:alice_identity_pub@bob"],
    "dic":"bob_2_alice_in_opaque_dic_temp"
}
```

Recall that ```alice_identity_pub@bob``` is a new public key that Alice generates with the sole purpose of communicating with Bob. She does not reuse this key for any other contact, thus avoiding correlations and linking.

This invitation document is finally encrypted for Bob using a key exchange derived from ```did:key:bob_invitation_pub_produced_by_alice``` and deposited at Alice's mediator, content referenced in the form:
```https://mediator.com/short/QmZtmD2qt6fJot32nabSP3CUjicnypEBz7bHVDhPQt9aAy```. From this reference, a QRCode is produced, looking like

![edge qr](./qr_edge_invite.png)

This document once retrieved, can only be decrypted if Bob also scanns/receives the associated jwk referenced with ```did:key:bob_invitation_pub_produced_by_alice```

### Contact Exchange : Request
We assume Bob is also registered with a mediator named ```bob-mediator```. After sucessfull processing of the invitation document provided by Alice, Bob is in possession of ```did:key:alice_identity_pub@bob```, the permanent contact key generated by alice-edge-agent for the sole purpose of communicating with Bob.

Now that Bob know the permanent contacct key of Alice, Bob can generate ```lice_2_bob_in_opaque_dic```, the permanent opaque DIC produced by Bob to allow Alice to deposit messages at bob-mediator.com.

Beside the ```alice_2_bob_in_opaque_dic```, bob-edge-agent will also generate ```bob_identity_pub@alice```, the permanent contact key to be used for communication with Alice.

The contact exchange request will allow Bob to send both ```alice_2_bob_in_opaque_dic``` and ```bob_identity_pub@alice``` to Alice.

bob-edge-agent generates a [contact exchange request](https://github.com/hyperledger/aries-rfcs/blob/main/features/0023-did-exchange/README.md#1-exchange-request) to be sent to Alice. The request contains:

```json
{
  "@id": "id_bob_contact_exchange_request_to_alice",
  "@type": "https://didcomm.org/didexchange/1.0/request",
  "~thread": { 
      "thid": "id_bob_contact_exchange_request_to_alice",
      "pthid": "id_alice_oob_2_bob"
  },
  "label": "Bob",
  "goal_code": "aries.rel.build",
  "goal": "To create a relationship",
  "did": "did:key:bob_identity_pub@alice",
  "dic":"alice_2_bob_in_opaque_dic"
}
```

**1. Authenticate generated contact key**
In order to authenticate the new generated did ```did:key:bob_identity_pub@alice```, bob-edge-agent will use it to sign the request. The resulting document will look like:

```json
{
  "payload":"eyJpZCI6IjEyM...(base64 encoded document)",
  "signatures":[
    {
      "protected":"eyJ0...",
      "signature":"gcW3l...",
      "header":{
          "kid":"did:key:bob_identity_pub@alice"
      }
    }
  ]
}  
```  

**2. Auth-Encrypt the message for alice-edge-agent**:
This document is consequently signed/encrypted for consumption by Alice. For this purpose, we use the [didcomm authenticated encryption authcrypt(plaintext)](https://identity.foundation/didcomm-messaging/spec/#iana-media-types). In this case, authcrypt (using ECDH-1PU) allow us to:
* guarantee confidentiality and integrity of the message. 
* prove the possession of the private key associated with ```did:key:bob_invitation_pub_produced_by_alice```

The resulting document will look like:

```json
{
  "id": "generated identifier",
  "type": "application/didcomm-encrypted+json",
  "from": ["did:key:bob_identity_pub@alice"],
  "to": "did:key:alice_identity_pub@bob",
  "body": {
    "ciphertext":"KWS7gJU7 ...",
    "protected":"eyJlcGsiOns ...",
    "recipients":[
        {
          "encrypted_key":"ZIL6Leligq1Xps_229nlo1xB_tGxOEVoEEMF...",
          "header":{
            "kid":"did:key:alice_identity_pub@bob"
          }
        }
    ],
    "tag":"nIpa3EQ29hgCkA2cBPde2HpKXK4_bvmL2x7h39rtVEc",
    "iv":"ESpmc ..."
  }
}    
```

We call this file an ```edge_agent_incomming_message```

**3. Auth-Encrypt the message for Inbox at alice-mediator**
Alice mediator has to know that bob-edge-agent is authorized to deposit messages for Alice. For that pupose, the packet delivered to alice-mediator must expose a ```dic``` or ```authorization``` that can be cosumed by alice-mediator. The packet produced by bob-edge-agent to be delivered to alice-mediator contains two files:
* the payload to be transfered. This is the payload encrypted for alice-mediator.
* the token authorizaing delivery of that payload. Assuming the transfer occurs over https, the token will be transported over the ```Authorization header``` of the http post request. Because the transport type is know to bob-edge-agent at message creation time, a token compatible with the correct transport mechanism will be used.

A sample DIC to be embedded into a bearer token will look like:

```json
{
  "typ":"JWT",
  "alg":"ES256",
  "iss":"did:key:bob_invitation_pub_produced_by_alice",
  "sub":"did:key:bob_invitation_pub_produced_by_alice",
  "exp":1516269022,
  "aud":"did:web:alice-mediator.com:alice_mediator_pub",
  "jti":"random nonce",
  "authorization":"bob_2_alice_in_opaque_dic_temp",
  "content-address":"QmZtmD2qt6fJ2ojd3abSP3CUjicnypEBz7bHVDhPQt9aAy"
}
```

From these claims, bob-edge-agent will create a jwt signed using the invitation public key ```did:key:bob_invitation_pub_produced_by_alice```. The produced JWT will be called ```mediator_inbox_access_token```.

A sample packet sent to alice-mediator will look like:

```json
{
  "id":"generated identifier",
  "type":"application/didcomm-encrypted+json",
  "from":"did:key:bob_invitation_pub_produced_by_alice",
  "to":["did:web:alice-mediator.com:alice_mediator_pub"],
  "authorization":"bob_2_alice_in_opaque_dic_temp",
  "body": {
    "ciphertext":"KWS7gJU7 ...",
    "protected":"eyJlcGsiOns ...",
    "recipients":[
        {
          "encrypted_key":"ZIL6Leligq1Xps_229nlo1xB_tGxOEVoEEMF...",
          "header":{
            "kid":"did:web:alice-mediator.com:alice_mediator_pub#key-1"
          }
        }
    ],
    "tag":"nIpa3EQ29hgCkA2cBPde2HpKXK4_bvmL2x7h39rtVEc",
    "iv":"ESpmc ..."
  }
}    
```

This file also contains the entry ```"authorization":"bob_2_alice_in_opaque_dic_temp"``` and ist auth-encrypted using ```did:key:bob_invitation_pub_produced_by_alice``` as sender key. Producing enougth authentication to allow the consumer of the file to authenticate the channel without the need to evaluate the access token. This is important as it simplifies the documentation of the request and the transport of the request over non https.

We call this file a ```mediator_inbox_message```

**4. Auth-Encrypt the message for Outbox at bob-mediator**
In our design, bob-edge-agent never connect to an untrusted mediator. All requests are sent to (resp. received from) bob-mediator. Therefore, bob-edge-agent will be encrypting the request for delivery to the outbox at bob-mediator. The content to be encrypted is the combination of both ```mediator_inbox_access_token``` and ```mediator_inbox_message```. The plain text file will look like:

```json
{
  "Mediator-Inbox-Access-Token":"mediator_inbox_access_token",
  "Mediator-Inbox-Message":"mediator_inbox_message"
}
```

This payload will be base64 encoded and fed into the encryption cypher stream. The resulting encrypted packet will look like:

```json
{
  "id": "generated identifier",
  "type": "application/didcomm-encrypted+json",
  "from":"did:key:bob_identity_pub@bob_mediator",
  "to": ["did:web:bob-mediator.com:bob_mediator_pub"],
  "body": {
    "ciphertext":"KWS7gJU7 ...",
    "protected":"eyJlcGsiOns ...",
    "recipients":[
        {
          "encrypted_key":"ZIL6Leligq1Xps_229nlo1xB_tGxOEVoEEMF...",
          "header":{
            "kid":"did:web:bob-mediator.com:bob_mediator_pub#key-1"
          }
        }
    ],
    "tag":"nIpa3EQ29hgCkA2cBPde2HpKXK4_bvmL2x7h39rtVEc",
    "iv":"ESpmc ..."
  }
}    
```

This final document is called a ```mediator_outbox_message```. It is the document sent by bob-edge-agent to bob-mediator. This document does not need to be encrypted again, as we assume there is an encrypted connection between bob-edge-agent and bob-mediator.

Nevertheless, bob-edge-agent still has to authenticate the channel used to send the message. If we assume that bob-mediator exposes a https interface, bob-edge-agent will have to produce a bearer token to authenticate the connection. The bearer token will look like:

```json
{
  "typ":"JWT",
  "alg":"ES256",
  "iss":"did:key:bob_identity_pub@bob_mediator",
  "sub":"did:key:bob_identity_pub@bob_mediator",
  "exp":1516269022,
  "aud":"did:web:bob-mediator.com:bob_mediator_pub",
  "jti":"random nonce",
  "authorization":"bob_out_opaque_dic",
  "content-address":"QmZtmD2qt6fJ2ojd3abSP3CUjicnypEBz7bHVDhPQt9aAy"
}
```
The content address is the ITFS mutihash of the payload. The produced JWT will be called a ```mediator_outbox_access_token```.

Recall that the ```mediator_outbox_message``` above does not contain an ```authorization``` entry, as the file is sent unencrypted. Authentication is solely perfromed putting the ```mediator_outbox_access_token``` in the http Authorization header. If edge-agent and mediator where using another protocol to transport the outbox message, we would add the claim ```"authorization":"bob_out_opaque_dic"``` to the message and auth-encrypt the message to turn it into a presentation.

### Pickup Message
Alice will be waiting for the contact exchange request to be available at ```alice-mediator```. The way the mediator notifies ```alice-edge-agent``` for the availability of this request is out of the scope of this specification. In order to receive the contact exchange request, ```alice-edge-agent``` sends a [pickup request](https://github.com/hyperledger/aries-rfcs/blob/main/features/0212-pickup/README.md) to ```alice-mediator```.

The pickup protocol generally starts with a status request that looks like:

```json
{
  "@id": "id_alice_pickup_status_request",
  "@type": "https://didcomm.org/messagepickup/1.0/status-request"
}
```

alice-edge-agent authenticates this status request with a ```mediator_outbox_access_token```.

Upon reception of the status request, the returned status response looks like:
```json
{
    "@id": "id_alice_pickup_status_response",
    "@type": "https://didcomm.org/messagepickup/1.0/status",
    "message_count": 7,
    "duration_waited": 3600,
    "last_added_time": "2019-05-01 12:00:00Z",
    "last_delivered_time": "2019-05-01 12:00:01Z",
    "last_removed_time": "2019-05-01 12:00:01Z",
    "total_size": 8096
}
```

This is a synchronous response to the status-request and does not need to be encrypted for alice-edge-agent, as TLS takes care of the authentication of the connection between alice-edge-agent and alice-mediator.

The ```message_count``` is the only required attribute in the status response.

Finaly alice-edge-agent will use the following request to load messages:

```json
{
  "@id": "id_alice_batch_pickup_request",
  "@type": "https://didcomm.org/messagepickup/1.0/batch-pickup",
  "batch_size": 10
}
```

This request will of course be authenticated with a ```mediator_outbox_access_token```.

The server will return a reponse containing messages, looking like:

```json
{
  "@id": "id_alice_batch_pickup_response",
  "@type": "https://didcomm.org/messagepickup/1.0/batch",
  "messages~attach": [
      {
          "@id" : "06ca25f6-d3c5-48ac-8eee-1a9e29120c31",
          "message" : {}
      },
      {
          "@id" : "344a51cf-379f-40ab-ab2c-711dab3f53a9a",
          "message" : {}
      }
  ]
}
```

### Contact Exchange : Response
Upon processing of Bob's contact exchange request, Alice receives the permament contact key of Bob ```did:key:bob_identity_pub@alice```. Alice can now use this to produce ```bob_2_alice_in_opaque_dic```, the authorization token used by bob-edge-agent to send messages to alice-edge-agent. Alice can now use the [contact exchange response](https://github.com/hyperledger/aries-rfcs/blob/main/features/0023-did-exchange/README.md#2-exchange-response) to send this permanent authorization token to alice-edge-agent. The plain text response will look like:

```json
{
  "@id": "id_alice_contact_exchange_response_to_bob",
  "@type": "https://didcomm.org/didexchange/1.0/response",
  "~thread": {
    "thid": "id_bob_contact_exchange_request_to_alice",
    "pthid": "id_alice_oob_2_bob"
  },
  "did": "did:key:alice_identity_pub@bob",
  "dic":"bob_2_alice_in_opaque_dic"
}
```

**1. Auth-Encrypt the message for bob-edge-agent**:
The plain text response will be auth-encrypted. The auth-encrypted message will look like:

```json
{
  "id": "generated identifier",
  "type": "application/didcomm-encrypted+json",
  "from":"did:key:alice_identity_pub@bob",
  "to": ["did:key:bob_identity_pub@alice"],
  "body": {
    "ciphertext":"KWS7gJU7 ...",
    "protected":"eyJlcGsiOns ...",
    "recipients":[
        {
          "encrypted_key":"ZIL6Leligq1Xps_229nlo1xB_tGxOEVoEEMF...",
          "header":{
            "kid":"did:key:bob_identity_pub@alice"
          }
        }
    ],
    "tag":"nIpa3EQ29hgCkA2cBPde2HpKXK4_bvmL2x7h39rtVEc",
    "iv":"ESpmc ..."
  }
}    
```

**2. Auth-Encrypt the message for Inbox at bob-mediator**
alice-edge-agent will produce a ```mediator_inbox_message```. bob-mediator has to know that alice-edge-agent is authorized to deposit messages for Bob. For that pupose, the packet delivered to bob-mediator must expose an ```authorization``` that can be cosumed by bob-mediator. The packet produced by alice-edge-agent to be delivered to bob-mediator contains two files:
* the payload to be transfered. This is the payload encrypted for bob-mediator.
* the token authorizing delivery of that payload.

The bearer token will look like:
```json
{
  "typ":"JWT",
  "alg":"ES256",
  "iss":"did:key:alice_identity_pub@bob",
  "sub":"did:key:alice_identity_pub@bob",
  "exp":1516269022,
  "aud":"did:web:bob-mediator.com:bob_mediator_pub",
  "jti":"random nonce",
  "authorization":"alice_2_bob_in_opaque_dic",
  "content-address":"QmZtmD2qt6fJ2ojd3abSP3CUjicnypEBz7bHVDhPQt9aAy"
}
```

From these claims, alice-edge-agent will create a jwt called ```mediator_inbox_access_token```.

The corresponding mediator_inbox_message sent to bob-mediator will look like:

```json
{
  "id":"generated identifier",
  "type":"application/didcomm-encrypted+json",
  "from":"did:key:alice_identity_pub@bob",
  "to":["did:web:bob-mediator.com:bob_mediator_pub"],
  "authorization":"alice_2_bob_in_opaque_dic",
  "body": {
    "ciphertext":"KWS7gJU7 ...",
    "protected":"eyJlcGsiOns ...",
    "recipients":[
        {
          "encrypted_key":"ZIL6Leligq1Xps_229nlo1xB_tGxOEVoEEMF...",
          "header":{
            "kid":"did:web:bob-mediator.com:bob_mediator_pub#key-1"
          }
        }
    ],
    "tag":"nIpa3EQ29hgCkA2cBPde2HpKXK4_bvmL2x7h39rtVEc",
    "iv":"ESpmc ..."
  }
}    
```
**3. Auth-Encrypt the message for Outbox at alice-mediator**
alice-edge-agent auth-encrypts the request for delivery to the outbox at alice-mediator. The content to be encrypted is the combination of both ```mediator_inbox_access_token``` and ```mediator_inbox_message```. The plain text file will look like:

```json
{
  "Mediator-Inbox-Access-Token":"mediator_inbox_access_token",
  "Mediator-Inbox-Message":"mediator_inbox_message"
}
```

This payload will be base64 encoded and fed into the encryption cypher stream. The resulting encrypted packet will look like:

```json
{
  "id": "generated identifier",
  "type": "application/didcomm-encrypted+json",
  "from":"did:key:alice_identity_pub@alice_mediator",
  "to": ["did:web:alice-mediator.com:alice_mediator_pub"],
  "body": {
    "ciphertext":"KWS7gJU7 ...",
    "protected":"eyJlcGsiOns ...",
    "recipients":[
        {
          "encrypted_key":"ZIL6Leligq1Xps_229nlo1xB_tGxOEVoEEMF...",
          "header":{
            "kid":"did:web:alice-mediator.com:alice_mediator_pub#key-1"
          }
        }
    ],
    "tag":"nIpa3EQ29hgCkA2cBPde2HpKXK4_bvmL2x7h39rtVEc",
    "iv":"ESpmc ..."
  }
}    
```

This final document is called a ```mediator_outbox_message```.

alice-edge-agent has to authenticate the channel used to send the message. If we assume that alice-mediator exposes a https interface, alice-edge-agent will have to produce the following bearer token to authenticate the connection:

```json
{
  "typ":"JWT",
  "alg":"ES256",
  "iss":"did:key:alice_identity_pub@alice_mediator",
  "sub":"did:key:alice_identity_pub@alice_mediator",
  "exp":1516269022,
  "aud":"did:web:alice-mediator.com:alice_mediator_pub",
  "jti":"random nonce",
  "authorization":"alice_out_opaque_dic",
  "content-address":"QmZtmD2qt6fJ2ojd3abSP3CUjicnypEBz7bHVDhPQt9aAy"
}
```
The content address is the ITFS mutihash of the payload. The produced JWT will be called a ```mediator_outbox_access_token```.

### Contact Exchange : Complete
This is the last message, sent by Bob to Alice, to complete the contact exchange. A sample contact example of the contact exchange message looks like:

```json
{
  "@id": "id_bob_contact_exchange_complete_to_alice",
  "@type": "https://didcomm.org/didexchange/1.0/complete",
  "~thread": {
      "thid": "id_bob_contact_exchange_complete_to_alice",
      "pthid": "id_alice_oob_2_bob"
  },
  "did": "did:key:bob_identity_pub@alice"
}
```

The trajectory looks same as the trajectory of the contact request message to Alice, except that bob-edge-agent uses permanent contact credentials instead of invitation credentials.
