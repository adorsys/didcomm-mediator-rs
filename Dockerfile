FROM rust:bookworm AS builder

WORKDIR /usr/src/app

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo,from=rust:bookworm,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/didcomm-mediator ./didcomm-mediator

FROM debian:bookworm-slim

RUN apt-get update && apt install -y \
    openssl && apt-get clean && \
    useradd -ms /bin/bash app

USER app
WORKDIR /app

COPY --from=builder /usr/src/app/didcomm-mediator /app/didcomm-mediator

EXPOSE 8080

CMD ["./didcomm-mediator", "/app/"]