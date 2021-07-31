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

pub fn configure() {
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
