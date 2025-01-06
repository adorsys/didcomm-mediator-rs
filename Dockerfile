# Stage 1: Builder
FROM rust:1.80 as builder  

WORKDIR /app

# Copy Cargo files and crates (this helps with caching dependencies)
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY crates ./crates

# Pre-cache dependencies and build the project
RUN cargo fetch && cargo build --release

# Dynamically mount source code during build for easy updates
VOLUME /app

# Ensure the binary is built and available
CMD cargo build --release && \
    cp target/release/didcomm-mediator /usr/local/bin/didcomm-mediator

# Stage 2: Runtime
FROM ubuntu:22.04  

# Install necessary runtime dependencies
RUN apt update && apt install -y libpq5 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Set the storage directory path
ENV STORAGE_DIRPATH="./storage"

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/didcomm-mediator /usr/local/bin/didcomm-mediator

# Copy runtime configurations
COPY .env .env

# Expose the port the app will listen on
EXPOSE 3000

# Set the entrypoint for the application
ENTRYPOINT ["didcomm-mediator"]
