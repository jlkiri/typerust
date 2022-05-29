mod error;
mod handler;
mod static_server;
mod telemetry;
mod wasm;

use crate::error::SandboxError;
use axum::{
    error_handling::HandleErrorLayer,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Extension, Json, Router,
};
use dotenv::dotenv;
use handler::HandlerResponse;
use serde::Deserialize;
use std::net::SocketAddr;
use std::{convert::Infallible, sync::Arc};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{instrument, subscriber::set_global_default, Level};
use wasm::create_interruptable_engine;

const MAX_AGE_ONE_HOUR: HeaderValue = HeaderValue::from_static("public, max-age=3600");
const MAX_AGE_ONE_YEAR: HeaderValue = HeaderValue::from_static("public, max-age=31536000");

#[derive(Deserialize, Debug)]
struct EnvConfig {
    ip_addr: SocketAddr,
    otlp_export_url: String,
    honeycomb_api_token: String,
    local_log_only: bool,
}

pub struct State {
    engine: wasmtime::Engine,
}

impl IntoResponse for SandboxError {
    fn into_response(self) -> Response {
        self.to_string().into_response()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let env = envy::from_env::<EnvConfig>().unwrap_or_else(|error| panic!("{:#?}", error));

    if env.local_log_only {
        println!("installing stdout subscriber...");
        let subscriber = telemetry::create_stdout_subscriber();
        set_global_default(subscriber).expect("failed to set global subscriber");
    } else {
        let subscriber =
            telemetry::create_otel_subscriber(env.honeycomb_api_token, env.otlp_export_url);
        set_global_default(subscriber).expect("failed to set global subscriber");
    };

    let engine = create_interruptable_engine();
    let state = Arc::new(State { engine });

    let static_service = static_server::file_service(MAX_AGE_ONE_HOUR, MAX_AGE_ONE_YEAR);

    let app = Router::new()
        .fallback(static_service)
        .route("/api/run", post(run))
        .route("/api/build", post(build))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(Extension(state))
        .layer(HandleErrorLayer::new(handle_error))
        .layer(middleware::from_fn(uncache_404));

    serve(app, env.ip_addr)
        .await
        .expect("server crashed unexpectedly");
}

async fn uncache_404<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let mut res = next.run(req).await;
    if res.status() == StatusCode::NOT_FOUND {
        res.headers_mut()
            .remove(static_server::CDN_CACHE_CONTROL_HEADER);
        res.headers_mut().remove(header::CACHE_CONTROL);
        return (StatusCode::NOT_FOUND, "404 - nothing to see here").into_response();
    }
    res
}

#[instrument(skip_all, name = "Invoke build handler", fields(
    service.name = "typerust"
))]
async fn build(code: String) -> impl IntoResponse {
    let response = handler::build(code).await?;
    Ok::<_, SandboxError>(Json(response))
}

#[instrument(skip_all, name = "Invoke run handler", fields(
    service.name = "typerust"
))]
async fn run(code: String, Extension(state): Extension<Arc<State>>) -> impl IntoResponse {
    match handler::run(code, state).await {
        Err(SandboxError::Internal(_)) => {
            tracing::error!("unexpected internal error");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(SandboxError::Timeout) => {
            let response = HandlerResponse::Error(
                "RUNTIME ERROR: Your code took too long to execute and was interrupted".into(),
            );
            Ok(Json(response))
        }
        Err(_) => {
            tracing::error!("memory limit or unallowed filesystem/network access");
            let response = HandlerResponse::Error(
                "RUNTIME ERROR: This could happen for the following reasons: 
a) you tried to access filesystem and/or network which is not allowed in this playground or 
b) your code exceeded memory limit and was interrupted."
                    .into(),
            );
            Ok(Json(response))
        }
        Ok(resp) => Ok(Json(resp)),
    }
}

async fn handle_error(_: Infallible) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal server error".to_string(),
    )
}

async fn serve(app: Router, addr: SocketAddr) -> Result<(), anyhow::Error> {
    println!("listening on {addr}...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
