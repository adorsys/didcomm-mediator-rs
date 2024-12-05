# Reliability Requirements and Key Concepts for DIDComm Mediator Server

## Introduction
The DIDComm mediator server serves as a crucial infrastructure component for reliable and secure message exchange in decentralized identity systems. Ensuring its reliability is vital to maintain trust, stability, and seamless operations in real-world environments. This document outlines the reliability requirements and defines key concepts essential to achieving this goal.

---

## Core Metrics

### Uptime
- **Requirements**: 99.9% availability.
- **Measurement**: Downtime will be monitored and logged to ensure it remains below 8.76 hours anually
- **Assumptions**: The underlying cloud infrastructure e.g server instance supports this level of availability

### Recovery Time Objective(RTO)
- **Requirement**: Maximum of 5 minutes.
- **Strategies**:
    - Automated restart of services using orchestration tools e.g kubernetes
    - Failover to a secondary server if the primary instance fails

### Latency
- **Goal**: Ensure messages are delivered or queued within 200ms.
- **Measurement**: End-to-End message delivery time logged and monitored for latency metrics

### Error Rate
- **Goal**: Ensure message failures or retries are less than 0.05%
- **Strategies**: 
    - Implement retry logic with expontential backoff for transient errors.
    - Use dead-letter queues for unrecoverable failures.

---

## Key Concepts

### Fault Tolerance
- **Definition**: The ability of the system to continue operating correctly in the presence of component failures.
- **Strategies**
    - Retry mechanisms for transient errors.
    - Dead-letter queues for unprocessable messages.
    - Circuit breakers to isolate and contain failures in dependent services.

### Message Delivery Guarantees
- **Definition**: Assurance that messages are delivered reliably according to defined semantics.
- **Approach**:
  - Implement **at-least-once delivery**: Messages are retried until acknowledged by the recipient.
  - Use acknowledgments and message IDs to prevent duplication.

### Failover Mechanisms
- **Definition**: Techniques to switch operations from a failing component to a redundant component to ensure continuity.
- **Approach**:
  - Active-active server clusters for load balancing and redundancy.
  - Database replication with automatic failover.
  - Use a load balancer (e.g., HAProxy, AWS ALB) to route traffic to healthy instances.

### Health Monitoring
- **Definition**: Processes and tools to track the real-time health and performance of the server.
- **Approach**:
  - Health check endpoints (e.g., `/health`) that provide a summary of system status.
  - Monitoring of system metrics such as CPU, memory, disk I/O, and network usage.

### Logging and Alerting
- **Definition**: Capturing and analyzing system events to detect and respond to failures or degradation.
- **Approach**:
  - Centralized logging for errors and operational metrics.
  - Alerting systems (e.g., PagerDuty, Slack notifications) for critical events.

---

## Dependencies
- **Message Queue**: RabbitMQ for reliable message transport.
- **Database**: Mongodb with replication for high availability and durability.
- **Orchestration**: Kubernetes or equivalent for containerized deployments.

---

## Assumptions and Constraints
- A mediator instance must be deployed to achieve redundancy.
- A robust monitoring and alerting system must be in place before deployment.
- External dependencies (e.g., database, message queue) must have their reliability addressed separately.
