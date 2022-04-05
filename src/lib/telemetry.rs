use sentry::ClientInitGuard;
use std::borrow::Cow;
use tracing::subscriber::set_global_default;
use tracing_actix_web_mozlog::{JsonStorageLayer, MozLogFormatLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Creates a tracing subscriber and sets it as the global default.
pub fn init_tracing<Sink>(service_name: &str, log_level: &str, sink: Sink)
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::new(log_level);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(MozLogFormatLayer::new(service_name, sink))
        .with(sentry_tracing::layer());

    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn init_sentry(dsn: &str) -> ClientInitGuard {
    sentry::init((
        dsn,
        sentry::ClientOptions {
            // TODO pull from version.yaml
            release: sentry::release_name!(),
            // TODO doc
            sample_rate: 1.0,
            // TODO doc
            traces_sample_rate: 0.3,
            // TODO how to vary this?
            environment: Some(Cow::from("local")),
            ..Default::default()
        },
    ))
}
