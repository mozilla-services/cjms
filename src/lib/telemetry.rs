use cadence::{CountedExt, StatsdClient, UdpMetricSink, Timed};
use sentry::ClientInitGuard;
use sentry_tracing::EventFilter;
use time::Duration;
use std::borrow::Cow;
use std::net::UdpSocket;
use strum_macros::Display as EnumToString;
use tracing::subscriber::set_global_default;
use tracing_actix_web_mozlog::{JsonStorageLayer, MozLogFormatLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use crate::settings::Settings;
use crate::version::{read_version, VERSION_FILE};

// TODO - Rename to something more generic e.g. LoggingKey
#[derive(Debug, EnumToString, PartialEq, Eq)]
#[strum(serialize_all = "kebab_case")]
pub enum TraceType {
    AicRecordCreate,
    AicRecordCreateFailed,
    BatchRefunds,
    CheckRefunds,
    CheckSubscriptions,
    Cleanup,
    ReportSubscriptions,
    RequestErrorLogTest,
    RequestIndexSuccess,
    StatsDError,
    Test, // For test cases
    WebApp,
}

/// Creates a tracing subscriber and sets it as the global default.
pub fn init_tracing<Sink>(service_name: &str, log_level: &str, sink: Sink)
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // Filter out any events that are below `log_level`.
    let env_filter = EnvFilter::new(log_level);

    // Prevent the subscriber from sending any events to Sentry that are below
    // ERROR. This is separate from the EnvFilter, which is responsible for the
    // log output itself. This is necessary to respect Sentry API call limits
    // set by SRE.
    let sentry_layer = sentry_tracing::layer().event_filter(|md| match md.level() {
        &tracing::Level::ERROR => EventFilter::Event,
        _ => EventFilter::Ignore,
    });

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(MozLogFormatLayer::new(service_name, sink))
        .with(sentry_layer);

    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn init_sentry(settings: &Settings) -> ClientInitGuard {
    let version_data = read_version(VERSION_FILE);

    sentry::init((
        settings.sentry_dsn.clone(),
        sentry::ClientOptions {
            environment: Some(Cow::from(settings.environment.clone())),
            // Suppress breadcrumbs.
            max_breadcrumbs: 0,
            release: Some(Cow::from(version_data.version)),
            // `sample_rate` defines the sample rate of error events (i.e. panics and error
            // log messages). Should always be 1.0.
            sample_rate: 1.0,
            // `traces_sample_rate` defines the sample rate of "transactional"
            // events that are used for performance insights. We don't want any
            // of this, so we set to zero.
            traces_sample_rate: 0.0,
            ..Default::default()
        },
    ))
}

pub fn info(trace_type: &TraceType, message: &str) {
    tracing::info!(r#type = trace_type.to_string().as_str(), message);
}

pub fn error(trace_type: &TraceType, message: &str, error: Option<Box<dyn std::error::Error>>) {
    match error {
        Some(err) => tracing::error!(
            r#type = trace_type.to_string().as_str(),
            "Message: '{}'. Original error: {:?}",
            message,
            err
        ),
        None => tracing::error!(r#type = trace_type.to_string().as_str(), message),
    };
}

pub struct StatsD {
    client: StatsdClient,
}

impl StatsD {
    pub fn new(settings: &Settings) -> Self {
        let host = (settings.statsd_host.clone(), settings.statsd_port);
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let sink = UdpMetricSink::from(host, socket).unwrap();

        StatsD {
            client: StatsdClient::from_sink("cjms", sink),
        }
    }
    pub fn incr(&self, key: &TraceType, suffix: &str) {
        let tag = format!("{}-{}", key, suffix.to_lowercase());
        self.client
            .incr(&tag)
            .map_err(|e| {
                error(
                    &TraceType::StatsDError,
                    &format!("Could not increment statsd tag {}", tag),
                    Some(Box::new(e)),
                );
            })
            .ok();
    }
    pub fn time(&self, key: &TraceType, suffix: &str, t: Duration) {
        let tag = format!("{}-{}", key, suffix.to_lowercase());
        let milliseconds = t.whole_milliseconds();
        self.client
            .time(&tag, milliseconds as u64)
            .map_err(|e| {
                error(
                    &TraceType::StatsDError,
                    &format!("Could not record time {:?} for statsd tag {}", t, tag),
                    Some(Box::new(e)),
                );
            })
            .ok();
    }
}
