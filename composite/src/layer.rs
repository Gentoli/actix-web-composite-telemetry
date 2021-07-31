use crate::tracing::level_filters::LevelFilter;
use crate::tracing::span::{Attributes, Record};
use crate::tracing::subscriber::Interest;
use crate::tracing::{Event, Id, Level, Metadata};
use std::marker::PhantomData;
use tracing::Subscriber;
use tracing_log::NormalizeEvent;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

pub struct EventLayer<S: Subscriber, L: Layer<S>> {
    inner: L,
    level: Level,
    sub: PhantomData<S>,
}

impl<S: Subscriber, L: Layer<S>> EventLayer<S, L> {
    pub fn new(inner: L, level: Level) -> Self {
        Self {
            inner,
            level,
            sub: Default::default(),
        }
    }
}

impl<S: Subscriber, L: Layer<S>> Layer<S> for EventLayer<S, L> {
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        self.inner.register_callsite(metadata)
    }

    fn enabled(&self, metadata: &Metadata<'_>, ctx: Context<'_, S>) -> bool {
        self.inner.enabled(metadata, ctx)
    }

    fn new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        self.inner.new_span(attrs, id, ctx)
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        self.inner.max_level_hint()
    }

    fn on_record(&self, _span: &Id, _values: &Record<'_>, _ctx: Context<'_, S>) {
        self.inner.on_record(_span, _values, _ctx)
    }

    fn on_follows_from(&self, _span: &Id, _follows: &Id, _ctx: Context<'_, S>) {
        self.inner.on_follows_from(_span, _follows, _ctx)
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let normalized_meta = event.normalized_metadata();
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        if meta.level() <= &self.level {
            self.inner.on_event(event, _ctx)
        }
    }

    fn on_enter(&self, _id: &Id, _ctx: Context<'_, S>) {
        self.inner.on_enter(_id, _ctx)
    }

    fn on_exit(&self, _id: &Id, _ctx: Context<'_, S>) {
        self.inner.on_exit(_id, _ctx)
    }

    fn on_close(&self, _id: Id, _ctx: Context<'_, S>) {
        self.inner.on_close(_id, _ctx)
    }

    fn on_id_change(&self, _old: &Id, _new: &Id, _ctx: Context<'_, S>) {
        self.inner.on_id_change(_old, _new, _ctx)
    }
}
