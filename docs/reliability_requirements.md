# Reliability Requirements and Key Concepts for DIDComm Mediator Server

## 1. Introduction
The DIDComm mediator server serves as a crucial infrastructure component for reliable and secure message exchange in decentralized identity systems. Ensuring its reliability is vital to maintain trust, stability, and seamless operations in real-world environments. This document outlines the reliability requirements and defines key concepts essential to achieving this goal.

---

## 2. Core Metrics

### 2.1 Uptime
- **Requirements**: Ensure the server is operational at least 99.9% of the time.
- **Measurement**: Downtime will be monitored and logged to ensure it remains below 8.76 hours anually
- **Assumptions**: The underlying cloud infrastructure e.g server instance supports this level of availability

### 2.2 Recovery Time Objective(RTO)
- **Requirement**: Maximum of 5 minutes.
- **Strategies**:
    - Automated restart of services using orchestration tools e.g kubernetes
    - Failover to a secondary server if the primary instance fails

### 2.3 Latency
- **Goal**: Ensure messages are delivered or queued within 200ms.
- **Measurement**: End-to-End message delivery time logged and monitored for latency metrics

### 2.4 Error Rate
- **Goal**: Ensure message failures or retries are less than 0.05%
- **Strategies**: 
    - Implement retry logic with expontential backoff for transient errors.
    - Use dead-letter queues for unrecoverable failures.

---

## 3. Key Concepts

### 3.1 Fault Tolerance
- **Definition**: The ability of the system to continue operating correctly in the presence of component failures.
- **Strategies**
    - Retry mechanisms for transient errors.
    - Dead-letter queues for unprocessable messages.
    - Circuit breakers to isolate and contain failures in dependent services.

### 3.2 Message Delivery Guarantees
- **Definition**: Assurance that messages are delivered reliably according to defined semantics.
- **Approach**:
  - Implement **at-least-once delivery**: Messages are retried until acknowledged by the recipient.
  - Use acknowledgments and message IDs to prevent duplication.

### 3.3 Failover Mechanisms
- **Definition**: Techniques to switch operations from a failing component to a redundant component to ensure continuity.
- **Strategies:**
    - Deploy at least two mediator server instances to ensure redundancy, using an active-active or active-passive configuration:
        - **Active-Active:** Both instances process requests simultaneously, ensuring high availability and load balancing.
        - **Active-Passive:** One instance handles traffic while the other remains on standby to take over in case of failure.
    - Use a load balancer (e.g., HAProxy, AWS ALB) to route traffic to healthy instances and perform health checks to detect failures.
    - Replicate the database to ensure consistency and availability across redundant instances.

### 3.4 Health Monitoring
- **Definition**: Processes and tools to track the real-time health and performance of the server.
- **Approach**:
  - Health check endpoints (e.g., `/health`) that provide a summary of system status.
  - Monitoring of system metrics such as CPU, memory, disk I/O, and network usage.

### 3.5 Logging and Alerting
- **Definition**: Capturing and analyzing system events to detect and respond to failures or degradation.
- **Approach**:
  - Centralized logging for errors and operational metrics.
  - Alerting systems (e.g., PagerDuty, Slack notifications) for critical events.

---

## 4. Dependencies
- **Messaging Library:** Uses DIDComm v2.0 protocol.
- **Message Queue**: RabbitMQ for reliable message transport.
- **Database**: Mongodb with replication for high availability and durability.
- **Orchestration**: Kubernetes or equivalent for containerized deployments.

---

## 5. Assumptions
- The cloud infrastructure supports at least 99.9% availability.
- External dependencies like databases and message queues are configured for high availability.
- A minimum of two mediator server instances can be deployed to achieve redundancy. This ensures high availability and fault tolerance. The instances can be configured in either an active-active setup, where all instances process requests concurrently, or an active-passive setup, where one instance acts as a backup. Redundancy also requires the use of a load balancer to distribute traffic and detect instance failures.

---

## 6. Constraints
- Initial implementation is limited to a single-region deployment.
- Budget constraints may limit the use of advanced failover solutions (e.g., multi-region active-active setups).

---

## 7. Testability
To ensure the reliability requirements are met, the following tests will be performed:

1. **Fault Tolerance Tests:**

- Simulate network outages to validate retry mechanisms.
- Test circuit breakers by intentionally introducing faults.
2. **Message Delivery Tests:**

- Validate acknowledgment workflows under high load.
- Test dead-letter queues with deliberate message failures.
3. **Failover Tests:**

- Simulate instance failures to ensure traffic is rerouted via the load balancer.
- Test database failover scenarios by shutting down the primary node.
4. **Stress Tests:**

- Conduct load testing to simulate high traffic conditions.
- Perform endurance tests to assess long-term stability.

