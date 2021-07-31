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

pub fn configure() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let filter = EnvFilter::from_default_env()
        .add_directive("debug".parse().unwrap())
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap());

    let collector = tracing_subscriber::fmt::layer(); //.json();

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("janus-server")
        .install_simple()
        .expect("jaeger");
    // let std_tracer = stdout::new_pipeline()
    //     .with_trace_config(Config::default().with_sampler(Sampler::AlwaysOn));
    //     .install_simple();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(EventLayer::new(collector, Level::DEBUG))
        .with(filter)
        .init();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
