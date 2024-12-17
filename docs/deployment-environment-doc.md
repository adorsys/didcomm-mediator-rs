
# **Application Deployment Documentation**

## **1. Prerequisites**
- **Tools and Software Required**:
  - Helm version (e.g., `Helm 3.x`)
  - Kubernetes version (e.g., `1.25+`)
  - Minikube/Cluster setup
  - Other dependencies (e.g., Docker, kubectl, etc.)
- **Environment Setup**:
  - Access to the Kubernetes cluster
  - Required configurations or credentials


## **3. Helm Chart Structure**
- **Chart Overview**:  
  - Structure of the Helm chart (values.yaml, templates, etc.).
  - Purpose of critical templates (e.g., Deployment, Service, ConfigMap).  
  - Default vs. custom configurations.  
- **Customization**:  
  - How to override values.yaml using custom configurations.  
    Example:
    ```bash
    helm install mediator ./mediator-charts --values custom-values.yaml
    ```
  - mandatory values are;
  
  **MONGO_DBN**,
  **MONGO_URI**, 
  **SERVER_LOCAL_PORT** and 
  **SERVER_PUBLIC_DOMAIN**


## **4. Deployment Guide**
- **Steps to Deploy**:
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
  4. Verify deployment status:
     ```bash
     kubectl get pods -n didcomm-mediator
     kubectl get services -n didcomm-mediator
     ```
- **Notes on Namespaces**:
  - Importance of creating and using the correct namespace.
- **Rollback Instructions**:
  - How to roll back to a previous release:
    ```bash
    helm rollback my-app <revision>
    ```

---

## **5. Accessing the Application**
- **Port Forwarding**:
  - Steps to forward the service ports locally for testing:
    ```bash
    kubectl port-forward service/<service-name> 8080:<target-port>
    ```
- **Ingress/LoadBalancer Details**:
  - Steps to access the application if exposed via Ingress or LoadBalancer.

## **7. Monitoring and Debugging**
- **Logs**:
  - How to fetch logs for debugging:
    ```bash
    kubectl logs <pod-name> -n didcomm-mediator
    ```
- **Monitoring Tools**:
  - Mention tools used (e.g., Prometheus, Grafana, ELK Stack).
  - Steps to configure and access monitoring dashboards.

