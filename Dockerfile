FROM rust:latest as builder

WORKDIR /server

# Workaround to avoid rebuilding dependencies every time
COPY server/Cargo.toml Cargo.toml
RUN mkdir -p src
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/typerust*

COPY server .
RUN cargo install --path .


FROM rust:1.61-slim-bullseye
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates
RUN rustup target add wasm32-wasi

COPY --from=builder /usr/local/cargo/bin/typerust .
COPY templates templates

EXPOSE 8080

ENTRYPOINT ["./typerust"]
