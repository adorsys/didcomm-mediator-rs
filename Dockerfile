ARG APP_NAME=didcomm-mediator

# Use buildx's automatic platform detection
FROM --platform=$BUILDPLATFORM blackdex/rust-musl:x86_64-musl AS builder-amd64
FROM --platform=$BUILDPLATFORM blackdex/rust-musl:aarch64-musl AS builder-arm64

# Select the appropriate builder based on target platform
FROM builder-${TARGETARCH} AS builder
ARG APP_NAME
ARG TARGETPLATFORM
ARG TARGETARCH
WORKDIR /app

# Set the Rust target and build the application
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target,id=target-cache-${TARGETPLATFORM} \
    --mount=type=cache,target=/root/.cargo/registry,id=registry-cache-${TARGETPLATFORM} \
    case "$TARGETARCH" in \
        amd64) RUST_TARGET="x86_64-unknown-linux-musl" ;; \
        arm64) RUST_TARGET="aarch64-unknown-linux-musl" ;; \
        *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
    esac; \
    cargo build --locked --release --target=${RUST_TARGET}; \
    mv target/${RUST_TARGET}/release/${APP_NAME} .

FROM gcr.io/distroless/static-debian12 AS runtime
ARG APP_NAME
COPY --from=builder --chown=nonroot:nonroot /app/${APP_NAME} /app/${APP_NAME}
EXPOSE 3000
ENTRYPOINT ["/app/didcomm-mediator"]