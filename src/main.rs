#![allow(unused)]

use std::fmt::Debug;
use std::marker::PhantomData;

use tracing::{
    event,
    field::{Field, Value, Visit},
    info_span, span,
    span::{Attributes, Record},
    Event, Id, Level, Subscriber,
};
use tracing_subscriber::{
    field::RecordFields,
    layer::{Context, Layer},
    registry::{LookupSpan, Registry},
};

fn main() {
    let subscriber = BooLog::new().with_subscriber(Registry::default());
    tracing::subscriber::set_global_default(subscriber).expect(":'(");

    let span = info_span!("my_span", foo = 3, bar = 0);
    span.record("bar", &"v1");
    span.in_scope(|| {
        let span = info_span!("my_span2", baz = 3);
        span.in_scope(|| {
            event!(Level::INFO, answer = 42, "another event");
        });
        event!(Level::INFO, answer = 42, "an event");
    });

    event!(Level::TRACE, "span-less event?");
}

pub struct BooLog<S> {
    _subscriber: PhantomData<S>,
}

impl<S> BooLog<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn new() -> Self {
        BooLog {
            _subscriber: PhantomData,
        }
    }

    fn parent_span(&self, attrs: &Attributes<'_>, ctx: &Context<'_, S>) -> Option<Id> {
        attrs
            .parent()
            .cloned()
            .or_else(|| ctx.current_span().id().cloned())
    }

    fn parent_event(&self, event: &Event<'_>, ctx: &Context<'_, S>) -> Option<Id> {
        event
            .parent()
            .cloned()
            .or_else(|| ctx.current_span().id().cloned())
    }
}

impl<S> Layer<S> for BooLog<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        println!(
            "Created span [{:?}] {}, from parent: {:?}",
            span.id(),
            span.metadata().name(),
            self.parent_span(attrs, &ctx),
        );
        print_fields("  new_span", attrs);
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        print_fields("on_record", values);
    }

    fn on_follows_from(&self, id: &Id, follows: &Id, ctx: Context<'_, S>) {
        panic!("PrintVisitor::on_follows_from");
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        println!("Event under span {:?}", self.parent_event(event, &ctx));
        print_fields("on_event", event);
    }
}

fn print_fields<R>(msg: &'static str, r: R)
where
    R: RecordFields,
{
    let mut visitor = PrintVisitor(msg);
    r.record(&mut visitor);
}

struct PrintVisitor(&'static str);

impl Visit for PrintVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        println!(
            "[{}] PrintVisitor::record_debug() - {} = {:?}",
            self.0, field, value
        );
    }
}
