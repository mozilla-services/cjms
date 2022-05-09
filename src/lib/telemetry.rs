use cadence::{MetricClient, StatsdClient, UdpMetricSink};
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
    VerifyReportsNoCount,
    VerifyReportsQuery,
    VerifyReportsRefundFound,
    VerifyReportsRefundNotFound,
    VerifyReportsRefundMatched,
    VerifyReportsRefundNotMatched,
    VerifyReportsRefundUpdateFailed,
    VerifyReportsRefundUpdated,
    VerifyReportsSubscriptionMatched,
    VerifyRefundsSubscriptionMissingFromDatabase,
    VerifyReportsSubscriptionNotFound,
    VerifyReportsSubscriptionNotMatched,
    VerifyReportsSubscriptionFound,
    VerifyReportsSubscriptionUpdateFailed,
    VerifyReportsSubscriptionUpdated,
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
    pub fn add_suffix(&self, suffix: &str) -> LogKey {
        let mut s = self.to_string();
        s.push_str("-");
        s.push_str(suffix);

        match LogKey::from_str(&*s) {
            Ok(v) => v,
            Err(_) => *self,
        }
    }
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
        crate::info!($trace_type.to_string().as_str(), $($arg)*);
        $statsd_client.incr(&$trace_type);
    }
}

/// TODO doc
#[macro_export]
macro_rules! error_and_incr {
    ( $statsd_client:expr, $trace_type:expr, $($arg:tt)+ ) => {
        crate::error!($trace_type.to_string().as_str(), $($arg)*);
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
        let expected = LogKey::TestEnding;
        let actual = LogKey::Test.add_suffix("ending");
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_log_key_with_invalid_suffix() {
        let expected = LogKey::Test;
        let actual = LogKey::Test.add_suffix("this-wont-work");
        assert_eq!(expected, actual);
    }
}
