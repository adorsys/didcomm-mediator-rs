# **Stress Testing and Failure Simulation Report**

### Test Overview

The purpose of these tests was to simulate high-load scenarios and potential failure conditions to assess the system’s resilience. The tests involved sending a large number of concurrent requests to the DIDComm mediator server and analyzing its performance, response times, and error rates under increasing load. Additionally, server recovery was evaluated by deliberately introducing failures during high-traffic scenarios.

### Test Methodology
**Stress Test Parameters:**

- Multiple phases of load with increasing and decreasing traffic.
- Simulated failure conditions (server crashes and network timeouts).
- Maximum concurrent virtual users: 10,000
- Duration: 5 minutes
- Request timeout: 10 seconds
- Tools Used:

Load tests were conducted using load-test.yml. Test reports were generated in JSON format for further analysis.

Results Summary
### 1. Initial Test Results (Moderate Load)
- Total Requests: 30,300
- Successful Requests (HTTP 200): 30,300 (100%)
- Errors: None
- Average Response Time: 3.7 ms
- Request Rate: 163 requests/second
- 95th Percentile Response Time: 10.9 ms
- Interpretation:

The system handled the moderate load effortlessly with no errors, low latency, and consistent response times.

### 2. Recent Test Results (Extreme Load)
- Total Requests: 507,001
- Successful Requests (HTTP 200): 121,614 (24%)
- Errors:
    - `ETIMEDOUT:` 385,287
    - `ECONNRESET:` 100
- Average Response Time: 8,100.3 ms
- Request Rate: 351 requests/second
- 95th Percentile Response Time: 9,999 ms
- Failed Virtual Users: 385,387

**Interpretation:**

Under extreme load, the server became overwhelmed, resulting in significant delays and a high number of timeouts (`ETIMEDOUT`). Only 24% of requests were successful. The average response time increased dramatically compared to the initial test. Many virtual users failed to complete the test due to connection resets and timeouts.

### Comparison and Analysis

| **Metric**               |**Initial Test (Moderate Load)**| **Extreme Load Test**     | **Difference**                             |
|--------------------------|--------------------------------|---------------------------|--------------------------------------------|
| **Total Requests**       | 30,300                         | 507,001                   | Significant increase in request volume     |
| **Successful Requests**  | 30,300 (100%)                  | 121,614 (24%)             | 76% failure rate under extreme load        |
| **Errors**               | None                           | 385,387 (ETIMEDOUT)       | Timeout errors due to server overload      |
| **Average Response Time**| 3.7 ms                         | 8,100.3 ms                | 2,186x increase in average response time   |
| **95th Percentile Time** | 10.9 ms                        | 9,999 ms                  | Requests hit the timeout limit             |
| **Request Rate**         | 163/sec                        | 351/sec                   | Higher request rate but with more failures |
|                          |                                |                           |                                            |  

### Key Observations
**a. Response Time:**

The response time grew from milliseconds to several seconds under high load, indicating the server struggled to handle the increased traffic.

**b. Timeout Errors (ETIMEDOUT):**

The majority of failures were due to timeouts, suggesting that the server reached its capacity limits and couldn’t respond within the 10-second threshold.

**c. Connection Resets (ECONNRESET):**

This indicates that the server forcefully closed some connections when it couldn’t process requests.

**d. Request Rate:**

The request rate increased from 163/sec to 351/sec, but with a high number of failures, suggesting the server hit a performance ceiling.

### Recommendations for Optimization
**Scale the Infrastructure:**

- **Horizontal Scaling:** Add more instances of the DIDComm mediator server behind a load balancer.
- **Vertical Scaling:** Upgrade server resources (CPU, memory).

**Introduce Caching:**

- Cache frequent responses to reduce server load.
- Consider using Redis for in-memory caching.

**Optimize Application Code:**

- Review and optimize any blocking operations or slow database queries.
- Use asynchronous I/O wherever possible.

**Monitor and Set Alerts**:

- Use Prometheus and Grafana to set up alerts for high CPU usage, high response times, and increased error rates.
- Ensure Discord webhooks notify you of critical failures.

**Retry Logic:**

Implement retry logic for critical requests, with exponential backoff to avoid overwhelming the server.