FROM rust:1.73.0-slim-buster
WORKDIR /usr/src/audible-news-rs
COPY . .
RUN apt-get update && apt-get install -y pkg-config make g++ libssl-dev
RUN cargo install --path .
CMD ["audible-news-rs"]