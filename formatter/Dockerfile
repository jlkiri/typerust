FROM rust:latest as builder

WORKDIR /formatter

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Workaround to avoid rebuilding dependencies every time
COPY Cargo.toml Cargo.toml
RUN mkdir -p src
RUN touch src/lib.rs
RUN rustup target add wasm32-unknown-unknown
RUN cargo build --release --target wasm32-unknown-unknown
RUN rm -f target/wasm32-unknown-unknown/deps/formatter*

COPY . .
RUN wasm-pack build --release --target web

FROM scratch

COPY --from=builder /formatter/pkg .
