FROM rust:latest

WORKDIR /app/

RUN cargo install cargo-watch

CMD ["cargo", "watch", "--why", "-x", "build"]