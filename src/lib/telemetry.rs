use cadence::{MetricClient, StatsdClient, UdpMetricSink};
use secrecy::ExposeSecret;
use sentry::ClientInitGuard;
use sentry_tracing::EventFilter;
use std::borrow::Cow;
use std::net::UdpSocket;
use std::panic::RefUnwindSafe;
use std::str::FromStr;
use std::sync::Arc;
use strum_macros::Display as EnumToString;
use strum_macros::EnumString;
use time::Duration;
use tracing::subscriber::set_global_default;
use tracing_actix_web_mozlog::{JsonStorageLayer, MozLogFormatLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use crate::settings::Settings;
use crate::version::{read_version, VERSION_FILE};

#[derive(Debug, EnumToString, EnumString, PartialEq, Eq, Clone, Copy)]
#[strum(serialize_all = "kebab_case")]
pub enum LogKey {
    AicRecordCreate,
    AicRecordCreateFailed,
    AicRecordUpdate,
    AicRecordUpdateFailed,
    AicRecordUpdateFailedNotFound,
    BatchRefunds,
    BatchRefundsEnding,
    BatchRefundsNNotReported,
    BatchRefundsStarting,
    BatchRefundsTimer,
    BatchRefundsUpdate,
    BatchRefundsUpdateFailed,
    BigQuery,
    CheckRefunds,
    CheckRefundsBytesFromBq,
    CheckRefundsDeserializeBigQuery,
    CheckRefundsDeserializeBigQueryFailed,
    CheckRefundsEnding,
    CheckRefundsNFromBq,
    CheckRefundsRefundCreate,
    CheckRefundsRefundCreateDatabaseError,
    CheckRefundsRefundCreateDuplicateKeyViolation,
    CheckRefundsRefundCreateFailed,
    CheckRefundsRefundDataChanged,
    CheckRefundsRefundDataUnchanged,
    CheckRefundsRefundFetchFailed,
    CheckRefundsRefundUpdate,
    CheckRefundsRefundUpdateFailed,
    CheckRefundsStarting,
    CheckRefundsSubscriptionMissingFromDatabase,
    CheckRefundsTimer,
    CheckRefundsTotalNFromBq,
    CheckSubscriptions,
    CheckSubscriptionsAicArchive,
    CheckSubscriptionsAicArchiveFailed,
    CheckSubscriptionsAicFetch,
    CheckSubscriptionsAicFetchFailed,
    CheckSubscriptionsAicFetchFromArchive,
    CheckSubscriptionsBytesFromBq,
    CheckSubscriptionsDeserializeBigQuery,
    CheckSubscriptionsDeserializeBigQueryFailed,
    CheckSubscriptionsEnding,
    CheckSubscriptionsNFromBq,
    CheckSubscriptionsStarting,
    CheckSubscriptionsSubscriptionCreate,
    CheckSubscriptionsSubscriptionCreateDatabaseError,
    CheckSubscriptionsSubscriptionCreateDuplicateKeyViolation,
    CheckSubscriptionsSubscriptionCreateFailed,
    CheckSubscriptionsTimer,
    CheckSubscriptionsTotalNFromBq,
    Cleanup,
    CleanupAicArchive,
    CleanupAicArchiveFailed,
    CleanupEnding,
    CleanupStarting,
    CleanupTimer,
    CorrectionsReport,
    CorrectionsReportByDayAccessed,
    CorrectionsReportTodayAccessed,
    CorrectionsSubscriptionFetch,
    CorrectionsSubscriptionFetchFailed,
    RequestAicCreate,
    RequestAicUpdate,
    ReportSubscriptionMarkNotReported,
    ReportSubscriptionMarkNotReportedFailed,
    ReportSubscriptionMarkWillNotReport,
    ReportSubscriptionMarkWillNotReportFailed,
    ReportSubscriptionReportToCj,
    ReportSubscriptionReportToCjButCouldNotMarkReported,
    ReportSubscriptionReportToCjFailed,
    ReportSubscriptions,
    ReportSubscriptionsAicExpiredBeforeSubscriptionCreated,
    ReportSubscriptionsEnding,
    ReportSubscriptionsNNotReported,
    ReportSubscriptionsStarting,
    ReportSubscriptionsSubscriptionHasNoAicExpiry,
    ReportSubscriptionsTimer,
    RequestLogTest,
    StatsDError,
    StatusHistoryDeserializeError,
    VerifyReports,
    VerifyReportsCount,
    VerifyReportsEnding,
    VerifyReportsNoCount,
    VerifyReportsQuery,
    VerifyReportsRefundFound,
    VerifyReportsRefundNotFound,
    VerifyReportsRefundMatched,
    VerifyReportsRefundNotMatched,
    VerifyReportsRefundUpdateFailed,
    VerifyReportsRefundUpdated,
    VerifyReportsStarting,
    VerifyReportsSubscriptionMatched,
    VerifyRefundsSubscriptionMissingFromDatabase,
    VerifyReportsSubscriptionNotFound,
    VerifyReportsSubscriptionNotMatched,
    VerifyReportsSubscriptionFound,
    VerifyReportsSubscriptionUpdateFailed,
    VerifyReportsSubscriptionUpdated,
    VerifyReportsTimer,
    VerifyReportsTooManyRecords,
    WebApp,
    WebAppEnding,
    WebAppStarting,
    WebAppTimer,

    // For test cases
    Test,
    TestEnding,
    TestErrorIncr,
    TestGauge,
    TestIncr,
    TestInfoIncr,
    TestStarting,
    TestTime,
    TestTimer,
}

impl LogKey {
    /// Use a string-based suffix to produce a related enum value.
    ///
    /// This is useful, for example, in situations where an abstract function is
    /// used in different contexts, but needs to log to different keys depending
    /// on the context. In this situation the abstract function can accept a
    /// "base key" as an argument, and the function can produce new Enum values
    /// by adding a known suffix to the base value.
    ///
    /// Note that the value produced by the method must be a valid Enum value.
    /// If the stringified value is invalid, the original enum value will be
    /// returned.
    ///
    /// # Examples
    ///
    /// ```
    /// LogKey::Cleanup.add_suffix("starting"); // returns LogKey::CleanupStarting
    /// LogKey::Cleanup.add_suffix("invalid"); // produces an invalid enum value; returns LogKey::Cleanup
    /// ```
    pub fn add_suffix(&self, suffix: &str) -> LogKey {
        let s = self.to_string() + "-" + suffix;

        match LogKey::from_str(&s) {
            Ok(v) => v,
            Err(_) => *self,
        }
    }
}

/// Creates a tracing subscriber and sets it as the global default.
/// For app initialization purposes.
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

/// Initialize a connection to Sentry.
/// For app initialization purposes.
pub fn init_sentry(settings: &Settings) -> ClientInitGuard {
    let version_data = read_version(VERSION_FILE);

    sentry::init((
        settings.sentry_dsn.expose_secret().clone(),
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

/// Create an info-level log trace.
///
/// The macro expects a `LogKey` enum value as its first argument. Aside from
/// this, the macro behaves exactly like the `tracing::info!` macro that it wraps.
///
/// # Examples
///
/// ```
/// info!(LogKey::CleanupAicArchive, key = "value", "Some log message")
/// ```
#[macro_export]
macro_rules! info {
    ( $trace_type:expr, $($arg:tt)+ ) => {
        tracing::info!(r#type = $trace_type.to_string().as_str(), $($arg)*)
    }
}

/// Create an error-level log trace.
///
/// The macro expects a `LogKey` enum value as its first argument. If the second
/// argument is a keyword argument with the name `error`, it will be assumed
/// that the argument an Error that implements Debug, which will be parsed and
/// formatted before being passed to the macro. Aside from these differences,
/// the macro behaves exactly like the `tracing::error!` macro that it wraps.
///
/// Note that the `error` keyword argument must be the second argument, directly
/// after the trace type. If it is not passed as the second argument, it will be
/// interpreted as a normal keyword argument and will not be formatted.
///
/// # Examples
///
/// ```
/// use std::error::Error;
///
/// error!(LogKey::CleanupAicArchiveFailed, key = "value", "Some log message")
///
/// error!(LogKey::CleanupAicArchiveFailed, error = Error, key = "value", "Some log message")
///
/// ```
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

/// Create an info-level log trace and increment a statsd counter with the same
/// name.
///
/// The macro expects a `StatsD` client as its first argument, followed by a
/// `LogKey` enum value. The name of the log trace and the statsd counter will
/// be the stringified form of the LogKey enum value. Aside from this, the macro
/// behaves exactly like the `info!` macro.
///
/// # Examples
///
/// ```
/// info_and_incr!(
///     StatsD::new(&settings),
///     LogKey::CleanupAicArchive,
///     key = "value",
///     "Some log message"
/// )
/// ```
#[macro_export]
macro_rules! info_and_incr {
    ( $statsd_client:expr, $trace_type:expr, $($arg:tt)+ ) => {
        $crate::info!($trace_type.to_string().as_str(), $($arg)*);
        $statsd_client.incr(&$trace_type);
    }
}

/// Create an error-level log trace and increment a statsd counter with the same
/// name.
///
/// The macro expects a `StatsD` client as its first argument, followed by a
/// `LogKey` enum value. Aside from this, the macro behaves exactly like the
/// `error!` macro.
///
/// # Examples
///
/// ```
/// use std::error::Error;
///
/// let statsd = StatsD::new(&settings);
///
/// error_and_incr!(
///     statsd,
///     LogKey::CleanupAicArchiveFailed,
///     key = "value",
///     "Some log message"
/// )
///
/// error_and_incr!(
///     statsd,
///     Error,
///     LogKey::CleanupAicArchiveFailed,
///     key = "value",
///     "Some log message"
/// )
/// ```
#[macro_export]
macro_rules! error_and_incr {
    ( $statsd_client:expr, $trace_type:expr, $($arg:tt)+ ) => {
        $crate::error!($trace_type.to_string().as_str(), $($arg)*);
        $statsd_client.incr(&$trace_type);
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
    pub fn incr(&self, key: &LogKey) {
        let k = key.to_string();
        self.client
            .incr(k.as_str())
            .map_err(|e| {
                error!(
                    LogKey::StatsDError,
                    error = e,
                    key = k.as_str(),
                    "Could not increment statsd tag"
                );
            })
            .ok();
    }
    pub fn gauge(&self, key: &LogKey, v: usize) {
        let k = key.to_string();
        let v = v as u64;
        self.client
            .gauge(k.as_str(), v)
            .map_err(|e| {
                error!(
                    LogKey::StatsDError,
                    error = e,
                    key = k.as_str(),
                    value = v,
                    "Could not record value for statsd tag"
                );
            })
            .ok();
    }
    pub fn time(&self, key: &LogKey, t: Duration) {
        let k = key.to_string();
        let milliseconds = t.whole_milliseconds();
        self.client
            .time(k.as_str(), milliseconds as u64)
            .map_err(|e| {
                error!(
                    LogKey::StatsDError,
                    error = e,
                    key = k.as_str(),
                    time = format!("{:?}", t).as_str(),
                    "Could not record time for statsd tag"
                );
            })
            .ok();
    }
}

#[cfg(test)]
pub mod test_telemetry {
    use super::*;

    #[test]
    fn get_log_key_with_valid_suffix() {
        let expected = LogKey::TestStarting;
        let actual = LogKey::Test.add_suffix("starting");
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_log_key_with_invalid_suffix() {
        let expected = LogKey::Test;
        let actual = LogKey::Test.add_suffix("this-wont-work");
        assert_eq!(expected, actual);
    }
}
