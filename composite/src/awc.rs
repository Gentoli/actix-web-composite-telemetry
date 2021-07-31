use awc::ClientRequest;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub use actix_web_opentelemetry::{ClientExt, InstrumentedClientRequest};

pub trait TracingClientExt {
    fn with_span(self, span: Span) -> InstrumentedClientRequest;

    fn instrument_current(self) -> InstrumentedClientRequest
    where
        Self: Sized,
    {
        self.with_span(Span::current())
    }
}

impl TracingClientExt for ClientRequest {
    fn with_span(self, span: Span) -> InstrumentedClientRequest {
        self.trace_request_with_context(span.context())
    }
}
