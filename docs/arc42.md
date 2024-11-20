Here is an ARC42 architecture document tailored for the Rust-based `didcomm-mediator-rs` project. The document focuses on operational aspects while addressing project-specific nuances.

---

# DIDComm Mediator RS - ARC42 Architecture Documentation

## Table of Contents
1. [Introduction](#introduction)
2. [Architecture Constraints](#architecture-constraints)
3. [System Scope and Context](#system-scope-and-context)
4. [Solution Strategy](#solution-strategy)
5. [Building Block View](#building-block-view)
6. [Runtime View](#runtime-view)
7. [Deployment View](#deployment-view)
8. [Cross-cutting Concepts](#cross-cutting-concepts)
9. [Architecture Decisions](#architecture-decisions)
10. [Quality Requirements](#quality-requirements)
11. [Risks and Technical Debt](#risks-and-technical-debt)
12. [Product Management](#product-management)
13. [Glossary](#glossary)
14. [Appendix](#appendix)

---

## 1. Introduction

### 1.1 Document Goals
This document outlines the architecture of the `didcomm-mediator-rs`, a Rust-based implementation of a DIDComm v2 mediator. Its purpose is to ensure secure, efficient, and reliable routing of DIDComm messages while providing clarity to stakeholders, architects, and developers.

### 1.2 Stakeholders
- **Product Owners**: Define feature requirements.
- **Developers**: Build and maintain the mediator.
- **Operations Team**: Deploy and monitor the mediator in production.
- **Compliance Teams**: Ensure adherence to privacy and security regulations.
- **DIDComm Users**: End-users leveraging decentralized identity systems.

---

## 2. Architecture Constraints

### 2.1 Technical Constraints
- **Rust Programming Language**: Ensures high performance and memory safety.
- **Transport Agnosticism**: Supports various protocols (HTTP, Bluetooth, etc.).
- **No External State**: Minimal reliance on external systems for scalability.

### 2.2 Regulatory Constraints
- **Privacy Compliance**: Aligns with GDPR and similar regulations.
- **Decentralized Design**: Eliminates central authority dependencies.

---

## 3. System Scope and Context

### 3.1 Business Context
The mediator facilitates routing of DIDComm messages for agents unable to maintain direct communication channels. It acts as an intermediary, ensuring reliability and privacy.

### 3.2 External Components
- **DIDComm Agents**: Sending and receiving agents.
- **Transport Protocols**: Various transport layers, e.g., HTTP or WebSockets.
- **Cloud Infrastructure**: For scalable and fault-tolerant deployments.

### 3.3 Component Diagram
```mermaid
graph LR
    Agent1[Agent 1] --> Transport1
    Transport1 --> Mediator1
    Mediator1 --> Mediator2
    Mediator2 --> Transport2
    Transport2 --> Agent2[Agent 2]
```

---

## 4. Solution Strategy
A microservices approach ensures modularity and scalability. `didcomm-mediator-rs` employs Rust's async capabilities to handle concurrent message routing efficiently.

### Key Features:
The `didcomm-mediator-rs` project implements several essential protocols in the DIDComm v2 ecosystem, as outlined below:

| **Feature**                             | **Specification Status** | **Implementation Status** |
|-----------------------------------------|---------------------------|----------------------------|
| **Mediator Coordination Protocol**      | Adopted                  | ✅ Implemented             |
| **Pickup Protocol**                     | Adopted                  | ✅ Implemented             |
| **DID Rotation**                        | Accepted                 | ✅ Implemented             |
| **Cross-Domain Messaging/Routing Protocol** | Adopted                  | ✅ Implemented             |
| **Trust Ping Protocol**                 | Adopted                  | ✅ Implemented             |
| **Discover Features Protocol**          | Adopted                  | ⚪ In Progress              |
| **Out-of-Band Messaging**               | Adopted                  | ⚪ In Progress              |
| **Basic Message Protocol**              | Adopted                  | ⚪ In Progress              |
| **Acknowledgments (Acks)**              | Adopted                  | ❌ Not Implemented         |
| **Present Proof Protocol**              | Adopted                  | ❌ Not Implemented         |

### Key Highlights:
1. **Implemented Protocols**:
   - The mediator is fully equipped with foundational protocols like **Mediator Coordination**, **Pickup**, and **DID Rotation**, ensuring secure and reliable DIDComm message handling.
   - **Cross-Domain Messaging** facilitates communication across diverse network boundaries, enhancing interoperability.

2. **In Progress Features**:
   - **Discover Features Protocol** and **Out-of-Band Messaging** are under development to expand functionality.
   - **Basic Message Protocol** will enable straightforward message exchanges between agents.

3. **Future Development**:
   - Support for **Acknowledgments (Acks)** and the **Present Proof Protocol** is planned, targeting comprehensive compliance with DIDComm standards.

This modular and extensible implementation ensures the mediator evolves with the SSI ecosystem while maintaining high standards for security and performance.

---

## 5. Building Block View

### 5.1 Overview

The `didcomm-mediator-rs` project utilizes a plugin-based architecture, allowing modular and extensible implementation of DIDComm messaging protocols. The architecture is divided into two key plugin layers:

1. **Web Plugin Layer**: 
   - Enables plug-and-play of web endpoints.
   - Currently used to implement features such as DID endpoints, out-of-band messages, and DIDComm messaging.

2. **DIDComm Messaging Plugin Layer**:
   - Facilitates incremental addition of DIDComm sub-protocols.
   - Modular implementation ensures support for future protocols without major refactoring.

### 5.2 Detailed View

#### Top-Level Architecture

At the core is the **Generic Server**, responsible for hosting the overall application and enabling the plugin system. The main components are:

1. **Dispatcher**: Routes incoming requests to the appropriate web plugin.
2. **Web Plugin Layer**: Hosts modular implementations for endpoint-specific logic (e.g., DID endpoint generation, out-of-band messages).
3. **Common DIDComm Processing**: Handles general DIDComm request/response processing applicable across sub-protocols.

#### DIDComm Messaging Plugin Layer

Nested within the DIDComm Messaging endpoint is another plugin system for managing sub-protocols. This includes:

1. **Forward Protocol**: Handles message routing across agents.
2. **Pickup Protocol**: Manages message retrieval by offline agents.
3. **Mediator Coordination Protocol**: Supports agent registration and mediation setup.
4. **Plugin Utilities**: Shared utilities that facilitate sub-protocol implementations.

### 5.3 Building Block Diagram

The following diagram illustrates the layered architecture with the plugin-based system:

```mermaid
graph TD
    GenericServer[Generic Server] --> Dispatcher[Dispatcher]
    Dispatcher --> WebPluginLayer[Web Plugin Layer]
    WebPluginLayer --> DIDEndpoint[DID Endpoint]
    WebPluginLayer --> OOBMessaging[Out-of-Band Messages]
    WebPluginLayer --> DIDCommMessaging[DIDComm Messaging Endpoint]
    DIDCommMessaging --> CommonProcessing[Common DIDComm Processing]
    DIDCommMessaging --> MessagingPlugins[Messaging Plugin Layer]
    MessagingPlugins --> Forward[Forward Protocol]
    MessagingPlugins --> Pickup[Pickup Protocol]
    MessagingPlugins --> Coordination[Mediator Coordination Protocol]
    MessagingPlugins --> Trustping[Trust Ping Protocol]
```

---

### 5.4 Key Advantages of Plugin Architecture

1. **Modularity**: Each protocol or feature can be independently developed, tested, and deployed.
2. **Extensibility**: New protocols or endpoints can be added without significant architectural changes.
3. **Scalability**: Lightweight plugins ensure efficient handling of additional features and traffic.

---

## 6. Runtime View

### 6.1 Message Flow
1. Agent sends a message to the mediator.
2. Mediator stores or forwards the message based on the recipient's status.
3. Recipient retrieves the message via the pickup protocol.

### Sequence Diagram
```mermaid
sequenceDiagram
    Agent1->>Mediator: Send Message
    Mediator->>Storage: Store Message
    Agent2->>Mediator: Pickup Request
    Mediator->>Agent2: Deliver Message
```

### 6.2 Mediator Coordination Flow
The Mediation Coordination Protocol is a component of the DIDComm framework, facilitating secure and efficient message routing between agents. It enables a recipient agent to request a mediator agent to handle message forwarding on its behalf. 

**Sequence Diagram: Mediation Coordination Protocol**

```mermaid
sequenceDiagram
    participant Recipient
    participant Mediator

    Recipient->>Mediator: Mediate Request
    alt Mediator Accepts
        Mediator-->>Recipient: Mediate Grant
        Note right of Mediator: Provides routing information
    else Mediator Denies
        Mediator-->>Recipient: Mediate Deny
        Note right of Mediator: Declines mediation request
    end
    alt Recipient Registers Key
        Recipient->>Mediator: Keylist Update
        Mediator-->>Recipient: Keylist Update Response
    end
```

**Illustration: Runtime Interaction**

1. **Mediate Request**: The recipient sends a `mediate-request` message to the mediator, initiating the mediation process.

2. **Mediator's Response**:
   - **Grant**: If the mediator agrees, it responds with a `mediate-grant` message, providing necessary routing details.
   - **Deny**: If the mediator declines, it sends a `mediate-deny` message.

3. **Key Registration**: Upon receiving a grant, the recipient registers its keys with the mediator using a `keylist-update` message. The mediator acknowledges with a `keylist-update-response`.

This interaction ensures that messages intended for the recipient are appropriately routed through the mediator, enhancing communication reliability and security.  

---

## 7. Deployment View

### 7.1 Deployment Strategy

The deployment of the `didcomm-mediator-rs` project leverages a scalable, distributed architecture designed for high availability, fault tolerance, and efficient message routing.

---

### 7.2 Deployment Models

#### 1. **Single Instance Deployment (Minimal Setup)**

**Use Case**: Suitable for development or testing environments.

- A single mediator instance handles all requests and routing.
- Simplifies deployment but lacks fault tolerance and scalability.

**Advantages**:
- Minimal resource usage.
- Easy to configure and maintain.

**Disadvantages**:
- No fault tolerance: if the mediator fails, the service becomes unavailable.
- Limited scalability.

---

#### 2. **Distributed Deployment with Load Balancer (Recommended)**

**Use Case**: Ideal for production environments requiring high availability and scalability.

- Multiple mediator instances are deployed behind a load balancer.
- The load balancer distributes requests evenly across mediators, ensuring efficient utilization of resources.
- Supports horizontal scaling by adding more mediator instances as needed.

**Advantages**:
- High availability: If one mediator instance fails, others continue handling requests.
- Scalable: New instances can be added to handle increased load.
- Efficient routing: Load balancer optimizes resource utilization.

**Disadvantages**:
- Slightly higher complexity due to additional components (load balancer).

---

### 7.3 Deployment Diagram

Below is a component diagram showcasing the deployment with multiple mediators and a load balancer:

```mermaid
graph TD
    subgraph Internet
        User1[Recipient Agent]
        User2[Sender Agent]
    end

    User1 --> LB[Load Balancer]
    User2 --> LB

    subgraph Mediator Cluster
        Mediator1[Mediator Instance 1]
        Mediator2[Mediator Instance 2]
    end

    LB --> Mediator1
    LB --> Mediator2

    Mediator1 --> DB1[Message Storage DB]
    Mediator2 --> DB1
```

---

### 7.4 Deployment Considerations

#### **Cloud Deployment**

**Infrastructure**: 
- Use cloud platforms like AWS, Azure, or GCP.
- Services like Elastic Load Balancer (AWS), Application Gateway (Azure), or Cloud Load Balancer (GCP) can be employed for load balancing.

**Scaling**:
- Leverage auto-scaling groups to dynamically scale mediator instances based on traffic.

**Monitoring**:
- Use monitoring tools (e.g., Prometheus, CloudWatch, or Azure Monitor) to track mediator performance, error rates, and resource usage.

---

#### **On-Premises Deployment**

**Infrastructure**:
- Deploy mediators on virtual machines or containers (e.g., Docker).
- Use a software-based load balancer (e.g., HAProxy, NGINX) for request distribution.

**Scaling**:
- Add more mediator instances manually to handle increased traffic.

**Monitoring**:
- Use tools like Grafana and Prometheus to monitor system health.

---

### 7.5 Benefits of Load Balancer in Deployment

1. **Fault Tolerance**:
   - Ensures service continuity even if one mediator instance fails.

2. **Scalability**:
   - Handles growing traffic by distributing load across multiple instances.

3. **Optimized Resource Utilization**:
   - Prevents overloading a single mediator, ensuring consistent performance.

4. **Simplified Maintenance**:
   - Instances can be updated or replaced without downtime by redirecting traffic.

This detailed deployment view provides clarity on how the system can be deployed effectively, balancing performance, fault tolerance, and scalability.

---

## 8. Cross-cutting Concepts

### 8.1 Security

The security of `didcomm-mediator-rs` integrates both **application-level security** and **secure software development lifecycle (SSDLC) practices** to ensure robust, private, and resilient operations against potential vulnerabilities and attacks.

---

#### 8.1.1 End-to-End Encryption

DIDComm itself provides robust end-to-end encryption, ensuring that the mediator operates without access to message contents.

- **Encryption in Transit**: Messages are encrypted using DIDComm standards, safeguarding confidentiality during routing.
- **Confidentiality**: Only the intended recipient can decrypt and access the message content, ensuring data remains private.

---

#### 8.1.2 Authentication and Agent Identity Verification

Authentication ensures that only legitimate agents interact with the mediator, preventing unauthorized access and spam.

1. **Authentication During Mediation Coordination**:
   - Agents requesting mediation must authenticate using:
     - **DID Authentication**: Validates that the DID corresponds to the public key.
     - **Signed Credentials**: Ensures requests are cryptographically signed and verifiable.
     - **Trust Frameworks**: Optional integration with registries or decentralized credentials to validate agents.

2. **Anti-Spam Measures**:
   - **Rate Limiting**: Restricts the number of requests per agent within a given period.
   - **Credential Validation**: Verifies agent credentials or DID associations.
   - **Reputation Scoring**: Maintains agent reputation based on past behaviors.

---

#### 8.1.3 Authorization for Message Handling

The mediator enforces strict access controls to ensure only authorized agents can send and retrieve messages.

1. **Message Sending**:
   - Sender validation ensures:
     - The sender provides a cryptographically signed request.
     - The recipient's DID document lists the mediator as a valid routing endpoint.
     - Access control policies allow communication between the sender and recipient.

2. **Message Pickup**:
   - Before delivering stored messages, the mediator ensures:
     - The requesting agent provides a signed pickup request.
     - The private key used for signing matches the public DID associated with the mediator.

---

#### 8.1.4 Secure Storage and Key Management

Protecting private keys and sensitive data is paramount.

- **Encryption at Rest**: Private keys and secrets are encrypted using secure algorithms.
- **Memory Protections**: Runtime mechanisms prevent sensitive data from being swapped to disk.
- **Key Management**:
  - Secure storage mechanisms like HashiCorp Vault or AWS KMS are used.
  - Keys are rotated periodically to minimize risks.

---

#### 8.1.5 Traffic Control and Anti-Spam Measures

To ensure efficient operation and prevent abuse:
- **Rate Limiting and Throttling**: Prevents overloading by limiting agent requests.
- **Reputation-Based Blocking**: Blocks agents with poor reputations based on spam or malicious activity.
- **Audit Trails**: Logs all requests for later analysis and forensic investigation.

---

#### 8.1.6 Secure Logging and Monitoring

**Log Handling**:
- Sensitive data (e.g., private keys, PII) is redacted from logs.
- Logs are encrypted at rest and in transit.
- Access to logs is restricted to authorized personnel.

**Monitoring**:
- Tools like Prometheus, Grafana, and ELK Stack are employed to monitor:
  - API response times and error rates.
  - System performance (e.g., memory and CPU utilization).
- Alerts are configured for anomalies, unauthorized access, and potential attacks.

---

#### 8.1.7 Secure Software Development Lifecycle (SSDLC)

Incorporating SSDLC practices ensures that vulnerabilities are minimized from inception through deployment.

1. **Static Application Security Testing (SAST)**:
   - Tools like **SonarQube**, **CodeQL**, or **Checkmarx** are used to scan source code for vulnerabilities.
   - Integrated into CI/CD pipelines to analyze every commit and pull request.

2. **Dynamic Application Security Testing (DAST)**:
   - Tools like **OWASP ZAP** or **Burp Suite** test the application in running environments.
   - Simulates real-world attack scenarios to identify runtime vulnerabilities.

3. **Dependency Management**:
   - Tools like **Snyk**, **OWASP Dependency-Check**, or GitHub Dependabot monitor dependencies for vulnerabilities.
   - Enforces strict policies to restrict unverified or malicious modules.
   - Locks dependency versions to prevent accidental updates.

4. **Code and Image Signing**:
   - All code artifacts are cryptographically signed to ensure integrity.
   - Container images are signed using tools like **Cosign** to verify authenticity before deployment.

---

#### 8.1.8 Continuous Security Assessment

1. **Penetration Testing**:
   - Regular penetration testing by internal and external experts to identify and remediate vulnerabilities.

2. **Vulnerability Management**:
   - Monitor CVEs and apply patches promptly.
   - Update dependencies and application components regularly.

3. **Incident Response**:
   - A defined incident response plan ensures quick detection, isolation, and resolution of security breaches.
   - Logs and audit trails provide forensic support.

---

#### 8.1.9 Deployment Security

1. **Immutable Infrastructure**:
   - Deployments use containerized environments (e.g., Docker, Kubernetes) for consistency.
   - Infrastructure updates are handled via image-based deployments.

2. **Secure CI/CD Pipelines**:
   - Pipelines enforce checks for vulnerabilities, code integrity, and dependency security.
   - Deployments use RBAC (Role-Based Access Control) to restrict unauthorized changes.

3. **Load Balancer Security**:
   - Load balancers enforce TLS for secure communication.
   - Denial-of-Service (DoS) protections are applied at the load balancer level.

---

#### 8.1.10 Enhancements for Anti-Spam and Traffic Control

1. **Rate Limiting**: Prevents message flooding by enforcing request caps.
2. **Trust-Based Validation**: Agents are validated using decentralized identity frameworks.
3. **Agent Reputation**: Maintains behavior-based reputation scores to filter malicious agents.

---

#### Summary Table

| **Aspect**                 | **Practices**                                                                 |
|-----------------------------|-----------------------------------------------------------------------------|
| **End-to-End Encryption**   | Encrypt messages in transit using DIDComm protocols.                        |
| **Authentication**          | Use DID authentication and signed credentials for agent verification.      |
| **Authorization**           | Enforce access control policies for sending and retrieving messages.       |
| **Secure Logging**          | Mask sensitive data, encrypt logs, and restrict access.                    |
| **Monitoring**              | Use tools like Prometheus and Grafana for real-time anomaly detection.     |
| **Static Analysis (SAST)**  | Integrate SonarQube or CodeQL into CI/CD pipelines.                         |
| **Dynamic Analysis (DAST)** | Test live environments using OWASP ZAP or Burp Suite.                      |
| **Dependency Management**   | Use Snyk or OWASP Dependency-Check to monitor and secure dependencies.      |
| **Code Signing**            | Sign all artifacts and verify container images before deployment.          |
| **Incident Response**       | Have a clear plan for detection, containment, and recovery.                |

---

This combination integrates **application security** and **SSDLC best practices**, providing a comprehensive security framework for `didcomm-mediator-rs`.

### 8.2 Performance
- **Concurrency**: Async Rust handles high message throughput.
- **Caching**: Reduces database queries for frequent operations.

---

## 9. Architecture Decisions
- **Rust for Performance**: Chosen for safety and speed.
- **Transport Agnosticism**: Supports various communication protocols.
- **Cloud-First**: Optimized for deployment in cloud environments.

---

## 10. Quality Requirements

### Functional
- Reliable message storage and delivery.
- DID rotation support.

### Non-Functional
- **Scalability**: Supports 10,000+ concurrent connections.
- **Security**: Fully encrypted message routing.

---

## 11. Risks and Technical Debt
- **Risks**: Potential bottlenecks in message pickup under high loads.
- **Debt**: Optimization of transport layer abstraction.

---

## 12. Product Management
- **Tech Stack**: Rust, Actix-web, SQLite, Kubernetes.
- **Versioning**: Semantic versioning for compatibility.

---

## 13. Glossary
- **DID**: Decentralized Identifier.
- **DIDComm**: Messaging protocol for DIDs.
- **Mediator**: Intermediary facilitating message delivery.

---

This document can be extended with implementation-specific details and diagrams as the project evolves.