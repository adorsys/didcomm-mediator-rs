FROM rust:latest as builder

WORKDIR /app

COPY . .

# Build the server
RUN cargo build --release

# Use a minimal image for running the server
FROM ubuntu

RUN apt update && apt install -y libpq5 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Set the storage directory path
ENV STORAGE_DIRPATH="crates/generic-server/target/storage"

# Copy the built binary
COPY --from=builder /app/target/release/didcomm-mediator /usr/local/bin/didcomm-mediator

# Expose the necessary port
EXPOSE 8080

# Run the server
CMD ["didcomm-mediator"]
