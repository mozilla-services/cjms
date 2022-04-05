use cadence::{StatsdClient, UdpMetricSink};
use sentry::ClientInitGuard;
use std::borrow::Cow;
use std::net::UdpSocket;
use tracing::subscriber::set_global_default;
use tracing_actix_web_mozlog::{JsonStorageLayer, MozLogFormatLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use crate::settings::Settings;
use crate::version::{read_version, VERSION_FILE};

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

pub fn init_sentry(settings: &Settings) -> ClientInitGuard {
    let version_data = read_version(VERSION_FILE);

    sentry::init((
        settings.sentry_dsn.clone(),
        sentry::ClientOptions {
            environment: Some(Cow::from(settings.environment.clone())),
            release: Some(Cow::from(version_data.version)),
            /// `sample_rate` defines the sample rate of error events (i.e. panics and error
            /// log messages). Should always be 1.0.
            sample_rate: 1.0,
            /// `traces_sample_rate` defines the sample rate of "transactional"
            /// events that are used for performance insights but are not
            /// directly related to error handling.
            traces_sample_rate: 0.3,
            ..Default::default()
        },
    ))
}

pub fn create_statsd_client(settings: &Settings) -> StatsdClient {
    // TODO investigate non-blocking version
    let host = (
        settings.statsd_host.clone(),
        settings.statsd_port.parse::<u16>().unwrap(),
    );
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = UdpMetricSink::from(host, socket).unwrap();

    StatsdClient::from_sink("cjms", sink)
}
