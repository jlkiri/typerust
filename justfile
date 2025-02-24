set dotenv-load := true
set shell := ["bash", "-uc"]

RUST_LOG := env("RUST_LOG", "debug")
IP_ADDR := env("IP_ADDR", "0.0.0.0:8080")
LOCAL_LOG_ONLY := env("LOCAL_LOG_ONLY", "true")
OTLP_EXPORT_URL := env("OTLP_EXPORT_URL", "")
HONEYCOMB_API_TOKEN := env("HONEYCOMB_API_TOKEN", "")

build-formatter:
    docker buildx build --file formatter/Dockerfile formatter --output formatter/pkg

frontend-install-deps:
    cd frontend && npm ci

build-frontend: build-formatter
    rm -rf server/public/*
    mkdir -p server/public
    cd frontend && npm run build
    cp -r frontend/dist/* server/public
    cp -r md/* server/public

build-frontend-ci: build-formatter frontend-install-deps build-frontend

build: build-frontend
    cargo build --manifest-path server/Cargo.toml

check:
    cargo check --manifest-path server/Cargo.toml

test:
    cargo test --manifest-path server/Cargo.toml

run-local: build
    IP_ADDR={{IP_ADDR}} LOCAL_LOG_ONLY={{LOCAL_LOG_ONLY}} OTLP_EXPORT_URL={{OTLP_EXPORT_URL}} HONEYCOMB_API_TOKEN={{HONEYCOMB_API_TOKEN}} RUST_LOG={{RUST_LOG}},typerust=debug cargo run --manifest-path server/Cargo.toml

build-image: build-frontend
    docker buildx build --tag typerust .

run: stop
    docker run --rm \
        --env RUST_LOG={{RUST_LOG}} \
        --env IP_ADDR={{IP_ADDR}} \
        --env LOCAL_LOG_ONLY={{LOCAL_LOG_ONLY}} \
        --env OTLP_EXPORT_URL={{OTLP_EXPORT_URL}} \
        --env HONEYCOMB_API_TOKEN={{HONEYCOMB_API_TOKEN}} \
        --detach --name playground \
        --publish 8080:8080 typerust

stop:
    docker stop playground || exit 0

deploy-dev:
    fly deploy --config fly.dev.toml

deploy-prod:
    fly deploy --config fly.production.toml

load-test:
    oha -c 100 -n 400 --disable-keepalive --method POST -d 'fn main() { println!("Hello, world!"); }' http://localhost:8080/api/run
