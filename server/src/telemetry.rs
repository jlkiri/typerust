use opentelemetry::sdk::export::trace::stdout;
use opentelemetry_otlp::WithExportConfig;
use tonic::{metadata::MetadataMap, transport::ClientTlsConfig};
use tracing::Subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;
use url::Url;

pub fn create_stdout_subscriber() -> impl Subscriber {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let trace_config =
        opentelemetry::sdk::trace::config().with_resource(opentelemetry::sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "typerust"),
        ]));

    let tracer = stdout::new_pipeline()
        .with_trace_config(trace_config)
        .with_pretty_print(true)
        .install_simple();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(telemetry)
}

pub fn create_otel_subscriber(
    honeycomb_api_token: String,
    otlp_export_url: String,
) -> impl Subscriber {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let mut metadata = MetadataMap::with_capacity(3);
    metadata.insert("x-honeycomb-team", honeycomb_api_token.parse().unwrap());

    let endpoint = Url::parse(&otlp_export_url).expect("endpoint is not a valid url");
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_metadata(metadata)
        .with_endpoint(endpoint.as_str())
        .with_tls_config(
            ClientTlsConfig::new().domain_name(
                endpoint
                    .host_str()
                    .expect("the specified endpoint should have a valid host"),
            ),
        );

    let trace_config =
        opentelemetry::sdk::trace::config().with_resource(opentelemetry::sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "typerust"),
        ]));

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(trace_config)
        .with_exporter(otlp_exporter)
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("failed to create otel pipeline");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    Registry::default().with(telemetry).with(filter)
}
