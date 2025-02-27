# DIDComm Mediator - Health Monitoring and Metrics

### Overview

This document outlines the health monitoring and metrics setup for the DIDComm Mediator server. It includes details on health check endpoints, system metrics collection, and alerting mechanisms to ensure high availability and performance.

### Health Check Endpoints

The server exposes a health check endpoint to monitor the status of the MongoDB connection and overall system health.

**Endpoint:** `/health`

- Method: GET

- Response:
  - `200 OK` if the server and MongoDB connection are healthy.
  - `503 Service Unavailable` if MongoDB is unreachable.

### Metrics Collection

The server collects and exposes metrics using Prometheus.

**Endpoint:** `/metrics`

**Method:** GET

**Description:** Exposes Prometheus-formatted metrics, including HTTP request counts, response times, and system resource usage.

**Integration:** Uses `axum_prometheus::PrometheusMetricLayer` to capture and expose relevant data.

### Monitoring Stack

The monitoring stack is based on Prometheus, Grafana, and Alertmanager.

**Prometheus**

- Scrapes metrics from the `/metrics` endpoint.
- Tracks system resource usage via Node Exporter and MongoDB Exporter.
- Scrape interval: 5s.

**Grafana**

- Visualizes metrics with preconfigured dashboards.
- Dashboards include:
  - API response times
  - CPU and memory usage
  - MongoDB connection status
  - Number of active requests

**Alertmanager**

- Triggers alerts based on Prometheus rules.
- Sends notifications to a configured Discord webhook.

**Alerting Rules**

| Alert Name             | Conditions                                                              | Severity | Action                               |
| ---------------------- | ------------------------------------------------------------------------ | -------- | ------------------------------------- |
| `InstanceDown`         | `up{job="didcomm-mediator"} == 0` for 10s                                  | Critical | Notify via Discord webhook             |
| `HighCPUUsage`         | `(100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[2m])) * 100)) > 80` for 30s | Warning  | Notify via Discord webhook             |
| `HighMemoryUsage`      | `(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100 > 85` for 1m | Warning  | Notify via Discord webhook             |
| `SlowAPIResponse`      | `histogram_quantile(0.95, rate(api_response_time_seconds_bucket[5m])) > 1` for 2m | Warning  | Notify via Discord webhook             |
| `MongoDBInstanceDown`  | `up{job="mongodb"} == 0` for 10s                                        | Critical | Notify via Discord webhook             |
| `HighMongoDBConnections` | `mongodb_connections{state="current"} > 100` for 30s                      | Warning  | Notify via Discord webhook             |
| `DiskSpaceLow` | `(1 - (node_filesystem_free_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})) * 100 > 90` for 1h | Warning | Notify via Discord webhook |


### Deployment Configuration

The monitoring stack is defined in the values.yaml file. Key configurations include:

- **MongoDB Monitoring:** Uses MongoDB Exporter to track performance.
- **Prometheus Operator:** Manages metric collection.
- **Node Exporter:** Monitors system resources.
- **Alertmanager:** Configured for Discord notifications.

### Future Improvements

- Extend `/health` to include system metrics (CPU, memory, disk usage).
- Enhance `/metrics` with request latency per endpoint.
- Add structured logging with `tracing::instrument`.

### Conclusion

This setup ensures real-time visibility into the DIDComm Mediator's health and performance. It enables proactive issue detection and automated alerts for quick response to critical failures.