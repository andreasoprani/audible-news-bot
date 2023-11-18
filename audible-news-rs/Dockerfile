FROM rust:1.73.0-slim-buster AS builder
WORKDIR /usr/src/audible-news-rs
COPY . .
RUN apt-get update && apt-get install -y pkg-config make g++ libssl-dev
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates
COPY --from=builder /usr/src/audible-news-rs/data /data
COPY --from=builder /usr/local/cargo/bin/audible-news-rs /usr/local/bin/audible-news-rs
CMD ["audible-news-rs"]