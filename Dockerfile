FROM rust:1.94 AS builder

WORKDIR /app
COPY ./src ./src
COPY Cargo.toml Cargo.toml

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/manager /app/manager

CMD ["./manager"]
