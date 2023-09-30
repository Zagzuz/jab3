# syntax=docker/dockerfile:1
FROM rust:bookworm AS builder
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev && apt-get install -y ca-certificates
COPY --from=builder ./target/release/jab3 ./target/release/jab3
CMD ["/target/release/jab3"]
