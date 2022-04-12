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

pub async fn configure(service_name: &'static str) {
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
        .with_service_name(service_name)
        .install_simple()
        .expect("jaeger");

    #[cfg(feature = "stackdriver")]
    let tracer = {
        use opentelemetry::sdk::trace;
        use opentelemetry::trace::TracerProvider;
        use opentelemetry_stackdriver::GcpAuthorizer;

        let authorizer = GcpAuthorizer::new()
            .await
            .expect("google service account creds");
        let (exporter, fut) = opentelemetry_stackdriver::StackDriverExporter::builder()
            .build(authorizer)
            .await
            .expect("failed to start stackdriver exporter");

        let _ = tokio::spawn(fut);

        let provider = trace::TracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();
        let tracer = provider.versioned_tracer(
            service_name,
            option_env!("SHORT_SHA"),
            Some("https://opentelemetry.io/schema/1.0.0")
        );

        let _ = global::set_tracer_provider(provider);

        tracer
    };

    #[cfg(feature = "std_tracer")]
    let tracer = {
        use opentelemetry::sdk::export::trace::stdout;
        use opentelemetry::sdk::trace::{Config, Sampler};

        stdout::new_pipeline()
            .with_trace_config(Config::default().with_sampler(Sampler::AlwaysOn))
            .install_simple()
    };

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
