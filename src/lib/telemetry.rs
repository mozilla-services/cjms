use cadence::{MetricClient, StatsdClient, UdpMetricSink};
use sentry::ClientInitGuard;
use sentry_tracing::EventFilter;
use std::borrow::Cow;
use std::net::UdpSocket;
use std::panic::RefUnwindSafe;
use std::sync::Arc;
use strum_macros::Display as EnumToString;
use time::Duration;
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
    BigQuery,
    CheckRefunds,
    CheckSubscriptions,
    Cleanup,
    CorrectionsReport,
    ReportSubscriptions,
    RequestLogTest,
    StatsDError,
    Test, // For test cases
    WebApp,

    // TODO sort
    CorrectionsSubscriptionFetch,
    CorrectionsSubscriptionFetchFailed,
    BatchRefundsUpdate,
    BatchRefundsUpdateFailed,
    CheckRefundsDeserializeBigQuery,
    CheckRefundsDeserializeBigQueryFailed,
    CheckRefundsSubscriptionMissingFromDatabase,
    CheckRefundsRefundDataChanged,
    CheckRefundsRefundDataUnchanged,
    CheckRefundsRefundUpdate,
    CheckRefundsRefundUpdateFailed,
    CheckRefundsRefundCreate,
    CheckRefundsRefundCreateDuplicateKeyViolation,
    CheckRefundsRefundCreateDatabaseError,
    CheckRefundsRefundCreateFailed,
    CheckRefundsRefundFetchFailed,
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

/// TODO doc
#[macro_export]
macro_rules! info {
    ( $trace_type:expr, $($arg:tt)+ ) => {
        tracing::info!(r#type = $trace_type.to_string().as_str(), $($arg)*)
    }
}

/// TODO doc
#[macro_export]
macro_rules! error {
    ( $trace_type:expr, error = $error:expr, $($arg:tt)+ ) => {
        tracing::error!(
            r#type = $trace_type.to_string().as_str(),
            error = format!("{:?}", $error).as_str(),
            $($arg)*)
    };
    ( $trace_type:expr, $($arg:tt)+ ) => {
        tracing::error!(
            r#type = $trace_type.to_string().as_str(),
            $($arg)*)
    };
}

/// TODO doc
#[macro_export]
macro_rules! info_and_incr {
    ( $statsd_client:expr, $trace_type:expr, $($arg:tt)+ ) => {
        info!($trace_type.to_string().as_str(), $($arg)*);
        $statsd_client.incr(&$trace_type, None);
    }
}

/// TODO doc
#[macro_export]
macro_rules! error_and_incr {
    ( $statsd_client:expr, $trace_type:expr, $($arg:tt)+ ) => {
        error!($trace_type.to_string().as_str(), $($arg)*);
        $statsd_client.incr(&$trace_type, None);
    }
}

#[derive(Clone)]
pub struct StatsD {
    client: Arc<dyn MetricClient + Send + Sync + RefUnwindSafe>,
}

impl StatsD {
    pub fn new(settings: &Settings) -> Self {
        let host = (settings.statsd_host.clone(), settings.statsd_port);
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let sink = UdpMetricSink::from(host, socket).unwrap();

        StatsD {
            client: Arc::new(StatsdClient::from_sink("cjms", sink)),
        }
    }
    pub fn incr(&self, key: &TraceType, suffix: Option<&str>) {
        let tag = match suffix {
            Some(s) => format!("{}-{}", key, s.to_lowercase()),
            None => key.to_string(),
        };
        self.client
            .incr(&tag)
            .map_err(|e| {
                error!(
                    TraceType::StatsDError,
                    error = e,
                    tag = tag.as_str(),
                    "Could not increment statsd tag"
                );
            })
            .ok();
    }
    pub fn gauge(&self, key: &TraceType, suffix: Option<&str>, v: usize) {
        let tag = match suffix {
            Some(s) => format!("{}-{}", key, s.to_lowercase()),
            None => key.to_string(),
        };
        let v = v as u64;
        self.client
            .gauge(&tag, v)
            .map_err(|e| {
                error!(
                    TraceType::StatsDError,
                    error = e,
                    value = v,
                    tag = tag.as_str(),
                    "Could not record value for statsd tag"
                );
            })
            .ok();
    }
    pub fn time(&self, key: &TraceType, suffix: Option<&str>, t: Duration) {
        let tag = match suffix {
            Some(s) => format!("{}-{}", key, s.to_lowercase()),
            None => key.to_string(),
        };
        let milliseconds = t.whole_milliseconds();
        self.client
            .time(&tag, milliseconds as u64)
            .map_err(|e| {
                error!(
                    TraceType::StatsDError,
                    error = e,
                    time = format!("{:?}", t).as_str(),
                    tag = tag.as_str(),
                    "Could not record time for statsd tag"
                );
            })
            .ok();
    }
}
