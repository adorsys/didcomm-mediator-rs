# **Server Health Monitoring & Metrics Documentation**

### **Overview**

This system integrates real-time health checks and resource monitoring for the DidComm Mediator server. It provides key metrics such as CPU load, memory usage, and API response times, all of which are captured by Prometheus and visualized in Grafana. Alerts are triggered for system anomalies, ensuring quick detection and response.
#
### **Key Features**

- **Health chceck endpoint**: To monitor server status.
- **Prometheus Metrics**: Collect and scrape system and application-level metrics.
- **Grafana Dashboad**: Visualize metrics and track performance.
- **Automated Alert**: Receive notifications for system anomalies like high CPU usage or slow API responses.
#
### Server Endpoints
- **Route:** ```/health```
- **Description:** Checks the server's operational status.
- **Response:**
    ```json
    {
        "Status": "OK",
    }
    ```
#
### Metrics Enpoint
- **Route:** ```/metrics```
- **Description:**  Exposes Prometheus-compatible metrics, which are scraped by Prometheus for monitoring.
- **Response:** Raw Prometheus metrics data (e.g., CPU, memory usage).
#
### Prometheus Metrics

#### Prometheus collects the following metrics:
- **CPU Usage:** Tracks CPU time spent in different modes (idle, system, user). Alert if usageexceeds 80% for 2 minutes.
- **Memory Usage:** Monitors available memory. Alert if usage exceeds 85% for 2 minutes.
- **Disk Usage:** Monitors disk space. Alert if disk space is below 20%.
- **API Response Time:** Measures API response times. Alert if the 95th percentile response time exceeds 1 second.
- **HTTP Errors:** Tracks failed HTTP requests. Alert if failure rate exceeds 5% over 5 minutes.
#
### Alerting

Prometheus triggers alerts based on defined conditions:

**Important Alerts**
- **InstanceDown:** Triggered if the server is unreachable for 30 seconds.
- **HighCPUUsage:** Triggered if CPU usage is over 80% for 2 minutes.
- **HighMemoryUsage:** Triggered if memory usage exceeds 85% for 2 minutes.   
- **SlowAPIResponse:** Triggered if API response times are too slow (95th percentile > 1s).
- **DiskSpaceLow:** Triggered if disk space is below 20%.  

Alerts are routed to Alertmanager, which handles notification via "any" application(e.g slack discord etc) (configured in ```alertmanager.yml```).
#
### Docker Setup

The monitoring stack is managed using Docker Compose. It includes:
- **Prometheus:** Collects metrics.
- **Grafana:** Visualizes metrics.
- **Alertmanager:** Sends notifications based on alerts.
- **Node Exporter:** Exposes system metrics (CPU, memory, disk usage).
### Build and run
```bash
docker compose up --build
```
### Access Grafana Dashboard:
- **URL:** ```http://localhost:3001```
#### Login: 
- **username:** ```admin```
- **password**: ```admin```

### Access Prometheus:
- **URL:** ```http://localhost:9090```

### Access Alertmanager:
- **URL:** ```http://localhost:9093```
#
### Conclusion

This setup ensures continuous monitoring of the DidComm Mediator server’s health and performance. By leveraging Prometheus for metrics collection and Grafana for visualization, combined with real-time alerting via Alertmanager, the system provides a robust way to monitor and react to any issues that may arise.