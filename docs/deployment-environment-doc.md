# **Application Deployment Documentation**

## **1. Prerequisites**

- **Tools and Software Required**:
  - Helm (e.g., `Helm 3.x`)
  - Kubernetes (e.g., `1.25+`)
  - Minikube or any other Kubernetes cluster setup
  - Other dependencies (e.g., Docker, kubectl)

- **Environment Setup**:
  - Access to a Kubernetes cluster
  - Required credentials or configurations (e.g., kubeconfig, Helm setup)


## **2. Development Deployment**

To run the application in a development environment, use Docker Compose:

```bash
docker-compose up --build
```

## **3. Production Deployment**

In production, deploy the application using Helm, as outlined in the next section. This involves setting up and configuring the necessary Helm charts.

## **4. Helm Chart Structure**

- **Chart Overview**:  
  This section explains the layout of the Helm chart and its components.
  - `values.yaml`: Contains default configurations.
  - `templates/`: Includes critical resources like Deployment, Service, ConfigMap, etc.
  - **Critical Templates**:
    - **Deployment**: Specifies the application container settings.
    - **Service**: Defines the Kubernetes service for networking.
    - **ConfigMap**: Used for configuration management.

- **Customization**:
  You can override the default `values.yaml` by providing your own custom configurations. Example:

  ```bash
  helm install mediator ./mediator-charts --values custom-values.yaml
  ```

  - **Mandatory values** to override in `custom-values.yaml`:
    - `MONGO_DBN`
    - `MONGO_URI`
    - `SERVER_LOCAL_PORT`
    - `SERVER_PUBLIC_DOMAIN`

## **5. Deployment Guide**

Follow these steps to deploy the application:

1. Clone the repository:
   ```bash
   git clone https://github.com/adorsys/didcomm-mediator-rs.git
   ```
   
2. Install dependencies:
   ```bash
   helm dependency update mediator-charts
   ```

3. Deploy using Helm:
   ```bash
   helm install mediator mediator-charts --namespace didcomm-mediator
   ```

4. Verify the deployment status:
   ```bash
   kubectl get pods -n didcomm-mediator
   kubectl get services -n didcomm-mediator
   ```

---

### **5.1 Notes on Namespaces**

- It is crucial to use the correct namespace when deploying to Kubernetes. Ensure that the namespace (`didcomm-mediator` in this case) exists, or create it manually:
  ```bash
  kubectl create namespace didcomm-mediator
  ```

---

### **5.2 Rollback Instructions**

To roll back to a previous release, use the following command:

```bash
helm rollback mediator <revision>
```

You can find the available revisions by running:

```bash
helm history mediator
```

---

## **6. Accessing the Application**

### **6.1 Port Forwarding**

To forward the application service ports locally for testing, use:

```bash
kubectl port-forward service/<service-name> 8080:<target-port>
```

### **6.2 Ingress/LoadBalancer Details**

If the application is exposed via Ingress or LoadBalancer, follow these steps to access it:
- Verify the external IP or URL:
  ```bash
  kubectl get ingress -n didcomm-mediator
  ```

---

## **7. Monitoring and Debugging**

### **7.1 Logs**

To fetch logs for a specific pod:

```bash
kubectl logs <pod-name> -n didcomm-mediator
```

Use `kubectl get pods -n didcomm-mediator` to find the pod names.

### **7.2 Monitoring Tools**

- **Prometheus/Grafana**: These tools can be set up to monitor your application's performance.
- **ELK Stack**: If configured, you can use ELK for centralized logging and monitoring.
  
**Steps to configure and access monitoring dashboards**:
- Install the monitoring tools (e.g., Prometheus, Grafana).
- Access Grafana dashboards using the provided URLs or port forwarding.

---

## **8. Additional Notes**

- **Helm Chart Upgrades**: To upgrade your release with new changes from the chart repository:
  ```bash
  helm upgrade mediator mediator-charts --namespace didcomm-mediator
  ```

- **Custom Values**: If you have custom values that should be applied to the Helm chart, always include them with the `--values` flag as shown earlier.
