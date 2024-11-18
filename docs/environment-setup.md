# DIDComm Mediator Integration Environment Setup

This guide provides step-by-step instructions to set up the integration environment for deploying, testing, and interacting with the Rust-based DIDComm mediator server.

## Prerequisites
Before you begin, ensure the following are installed on your system:
- Docker: version 20+([install docker](https://docs.docker.com/engine/install/debian/))
- Rust: latest stable version ([install rust](https://www.rust-lang.org/tools/install))
- MongoDB: latest stable version ([install mongodb](https://www.mongodb.com/docs/manual/tutorial/install-mongodb-on-ubuntu/))

This documentation assumes you have clone the [mediator](https://github.com/adorsys/didcomm-mediator-rs) and are in the root of the project

## step 1: Setup the environment variables
Modify the ```.env``` file with the right values for the variables

## Step 2: Start The Environment

```sh
 docker-compose up -d
```
## Step 3: Test Connectivity
Use a tool like Postman or curl to verify that the server is running and responding.
```sh 
curl -X GET http://0.0.0.0:8080/ \
-H "Content-Type: application/json" \
```

## Step 4: Logging And Monitoring
to monitor the logs run the command
```sh 
docker logs -f didcomm-mediator
```

## Step 5: Cleanup
To Stop and remove the environment
```sh
docker-compose down
```