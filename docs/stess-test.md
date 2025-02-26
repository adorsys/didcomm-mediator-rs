# **Stress Testing and Failure Simulation Report**

### Test Overview

The purpose of these tests was to simulate high-load scenarios and potential failure conditions to assess the system’s resilience. The tests involved sending a large number of concurrent requests to the DIDComm mediator server and analyzing its performance, response times, and error rates under increasing load. Additionally, server recovery was evaluated by deliberately introducing failures during high-traffic scenarios.

### Test Methodology
**Stress Test Parameters:**
- Target Server: https://didcomm-mediator.eudi-adorsys.com

- Phases:
    - Warm-up Phase: 15 seconds, increasing from 10 to 50 virtual users (VUs) per second.
    - Sustained Load Phase: 60 seconds, constant load of 50 VUs per second.
    - Stress Phase: 30 seconds, constant load of 100 VUs per second.
    - Cooldown Phase: 15 seconds, decreasing load to 0 VUs per second.

- Thresholds:
    - Response Time: 95% of requests should complete within 500ms.
    - Error Rate: Less than 10% of requests should fail.
- Scenario:
    - Basic Load Test: Sends HTTP GET requests to the root endpoint (/)    with a timeout of 3 seconds and 2 retries.

### How to Run and Analyze Artillery Load Tests 

**Step 1: Run the Load Test**
- Execute the following command: 
 ```bash
artillery run --output report.json load-test.yml
 ```
 This command runs the test based on the configurations in `load-test.yml` file and save the results in `report.json`.

**Step 2: Generate a Report**
- Run this command to convert the JSON report into an HTML file:
```bash
artillery report report.json
```
This command will converte the json data in the `report.html` file which you can open in a browser for a detailed visualization.

**Results Summary**


| **Metric**               |  **Values**                    | 
|--------------------------|--------------------------------|
| **Total Requests**       | 6,450                          |
| **Successful Requests**  | 6,450 (100%)                   |
| **Failed Requests**      | 1 (ETIMEDOUT)                  |
| **Average Response Time**| 3.7 ms                         |
| **95th Percentile Time** | 2,893.5 ms (~2.9 seconds)      |
| **99th Percentile (p99)** |3,464.1 ms (~3.5 seconds)      |
| **Request Rate**         | 56 requests/second             |
| **Virtual Users Created**| 6,450                          |
| **Virtual Users Failed** | 1                              |
| **Data Transferred**     | 146,321,361 bytes (~146 MB)    |
| **Average Response Time**| 990.1 ms (~1 second)           |

**Response Times**
- Min: 71 ms
- Max: 4,875 ms (~4.9 seconds)
- Mean: 990.1 ms (~1 second)
- Median: 772.9 ms (~0.8 seconds)
- p95: 2,893.5 ms (~2.9 seconds)
- p99: 3,464.1 ms (~3.5 seconds)

**Errors**
- Timeout Errors (ETIMEDOUT): 1
    - One request timed out, likely due to high load or network latency.

### Interpretation of Results
**1. Performance Under Load**
- The server handled 6,450 requests with an average response time of ~1 second.
- 95% of requests completed within ~2.9 seconds, and 99% of requests completed within ~3.5 seconds.
- The maximum response time was ~4.9 seconds, indicating some requests experienced significant delays.

**2. Error Analysis**
- There was 1 timeout error (ETIMEDOUT), which caused a single virtual user to fail.

- **This could be due to:**
    - High server load during the stress phase.
    - Network latency or connectivity issues.
    - A bottleneck in the server (e.g., database, CPU, or memory).

**3. Throughput**
- The server processed 56 requests per second on average.
- This throughput is acceptable for many applications but may need improvement for higher traffic scenarios.

**4. Threshold Compliance**
- Response Time Threshold: The p95 response time exceeded the threshold of 500ms, indicating that the server struggled to meet the performance target under high load.
- Error Rate Threshold: The error rate was 0.015% (1 failure out of 6,450 requests), which is well below the 10% threshold.

### Key Observations
**Response Time Degradation:**
- Response times increased significantly under high load, especially during the stress phase.
- The p95 response time of ~2.9 seconds suggests that the server struggled to handle the increased traffic.

**Timeout Error:**

- The single timeout error indicates that the server may have reached its capacity limits during the stress phase.

**Scalability:**
- The server performed well under moderate load but showed signs of strain under high load.

### Recommendations for Optimization

**1. Scale the Infrastructure**
- Horizontal Scaling: Add more instances of the DIDComm mediator server behind a load balancer to distribute the load.
- Vertical Scaling: Upgrade server resources (CPU, memory) to handle higher traffic.

**2. Optimize Application Code**
- Database Queries: Review and optimize slow or inefficient database queries.
- Caching: Implement caching for frequently accessed data (e.g., using Redis).
- Asynchronous Processing: Use asynchronous I/O to improve concurrency and reduce response times.

**4. Retry Logic**
- Implement retry logic with exponential backoff for failed requests to improve reliability.

**5. Increase Timeout Threshold**
- Consider increasing the request timeout from 3 seconds to 5 seconds to reduce timeout errors during high load.

**Conclusion**
The DIDComm Mediator Server performed well under moderate load but showed signs of strain during the stress phase. The p95 response time exceeded the threshold, and there was 1 timeout error. To improve performance and scalability, consider scaling the infrastructure, optimizing the application code, and implementing caching.