ARG RUST_VERSION=1.83.0
ARG APP_NAME=didcomm-mediator

# Build stage
FROM rust:${RUST_VERSION}-alpine AS build

# Arguments and working directory
ARG APP_NAME
WORKDIR /app

# Install required dependencies
RUN apk add --no-cache clang lld musl-dev gcc g++ git build-base pkgconf openssl-dev openssl-libs-static

# Add MUSL target
RUN rustup target add x86_64-unknown-linux-musl

# Build the application
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
cargo build --locked --release --target=x86_64-unknown-linux-musl && \
cp ./target/x86_64-unknown-linux-musl/release/${APP_NAME} /bin/server

# Final stage
FROM alpine:3.18 AS final

# Create a non-root user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the compiled binary
COPY --from=build /bin/server /bin/

# Expose application port
EXPOSE 3000

# Command to run the application
CMD ["/bin/server"]
