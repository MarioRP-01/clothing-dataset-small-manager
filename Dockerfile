# Build stage
FROM rust:1.68-alpine as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apk add openssl-dev musl-dev
RUN update-ca-certificates

# Create App User
ENV USER=dataset-manager
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /app
COPY . /app

RUN cargo build --release

# Prod stage
FROM alpine:latest

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

COPY --from=builder /app/target/release/dataset-manager /usr/local/bin/

USER dataset-manager:dataset-manager

CMD ["dataset-manager", "-d", "/data"]
