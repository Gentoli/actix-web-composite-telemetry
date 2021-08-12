pub mod awc;
mod layer;

use crate::layer::EventLayer;

use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub use actix_web_opentelemetry::RequestTracing;
pub use opentelemetry::trace::TraceContextExt;
pub use opentelemetry::Context;
pub use tracing;
pub use tracing_actix_web::{RootSpan, TracingLogger};
pub use tracing_attributes::instrument;
pub use tracing_opentelemetry::OpenTelemetrySpanExt;

use tracing_subscriber::fmt::format::FmtSpan;

pub async fn configure() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new("debug"))
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap());

    let collector = tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE);
    #[cfg(feature = "json_log")]
    let collector = collector.json();

    #[cfg(feature = "jaeger")]
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("janus-server")
        .install_simple()
        .expect("jaeger");

    #[cfg(feature = "stackdriver")]
    let tracer = {
        use opentelemetry::sdk::trace;
        use opentelemetry::trace::TracerProvider;
        use opentelemetry_stackdriver::tokio_adapter::TokioSpawner;
        use opentelemetry_stackdriver::GcpAuthorizer;
        use std::time::Duration;

        let authorizer = GcpAuthorizer::new()
            .await
            .expect("google service account creds");
        let handle = tokio::runtime::Handle::current();
        let spawner = TokioSpawner::new(handle);
        let exporter = opentelemetry_stackdriver::StackDriverExporter::connect(
            authorizer,
            &spawner,
            Some(Duration::from_secs(10)),
            10,
        )
        .await
        .expect("failed to start stackdriver exporter");
        let provider = trace::TracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();
        let tracer = provider.get_tracer("janus", option_env!("SHORT_SHA"));

        let _ = global::set_tracer_provider(provider);

        tracer
    };

    #[cfg(feature = "std_tracer")]
    let tracer = stdout::new_pipeline()
        .with_trace_config(Config::default().with_sampler(Sampler::AlwaysOn))
        .install_simple();

    #[cfg(feature = "trace_output")]
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let registry = tracing_subscriber::registry()
        .with(EventLayer::new(collector, Level::DEBUG))
        .with(env_filter);

    #[cfg(feature = "trace_output")]
    let registry = registry.with(telemetry);

    registry.init();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
