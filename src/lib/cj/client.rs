use crate::{info, models::subscriptions::Subscription, settings::Settings, telemetry::LogKey};
use rand::{thread_rng, Rng};
use reqwest::{Client, Error, Response, Url};
use secrecy::{ExposeSecret, Secret};
use serde::{de, Deserialize, Deserializer};
use serde_json::{json, Value};
use time::{Duration, OffsetDateTime};

use super::country_codes::get_iso_code_3_from_iso_code_2;

pub struct CJClient {
    advertiser_id: String,
    client: reqwest::Client,
    cj_cid: String,
    cj_type: String,
    cj_signature: String,
    commission_detail_endpoint: Url,
    commission_detail_api_token: Secret<String>,
    s2s_endpoint: Url,
    random_minutes: Duration,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CommissionDetailItem {
    pub sku: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionDetailRecord {
    pub original: bool,
    pub order_id: String,
    pub correction_reason: Option<String>,
    pub coupon: Option<String>,
    #[serde(deserialize_with = "f32_from_str")]
    pub sale_amount_pub_currency: f32,
    pub items: Vec<CommissionDetailItem>,
}

#[derive(Debug, Deserialize)]
pub struct CommissionDetailRecordSet {
    pub count: usize,
    pub records: Vec<CommissionDetailRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertiserCommissions {
    pub advertiser_commissions: CommissionDetailRecordSet,
}

#[derive(Debug, Deserialize)]
struct CommissionDetailQueryResponse {
    data: Option<AdvertiserCommissions>,
    errors: Option<Value>,
}

pub fn convert_amount_to_decimal(plan_amount: i32) -> f32 {
    plan_amount as f32 / 100.0
}

fn f32_from_str<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<f32>().map_err(de::Error::custom)
}

fn get_random_minutes() -> Duration {
    let mut rng = thread_rng();
    let minutes = rng.gen_range(15..=60);
    Duration::minutes(minutes)
}

impl CJClient {
    pub fn new(
        settings: &Settings,
        s2s_endpoint: Option<&str>,
        commission_detail_endpoint: Option<&str>,
        random_minutes: Option<Duration>,
    ) -> CJClient {
        let s2s_endpoint = s2s_endpoint.unwrap_or("https://www.emjcd.com/u");
        let commission_detail_endpoint =
            commission_detail_endpoint.unwrap_or("https://commissions.api.cj.com/query");
        CJClient {
            advertiser_id: settings.cj_sftp_user.clone(),
            client: Client::new(),
            cj_cid: settings.cj_cid.clone(),
            cj_type: settings.cj_type.clone(),
            cj_signature: settings.cj_signature.clone(),
            commission_detail_endpoint: Url::parse(commission_detail_endpoint)
                .expect("Could not parse commission_detail_endpoint"),
            commission_detail_api_token: settings.cj_api_access_token.clone(),
            s2s_endpoint: Url::parse(s2s_endpoint).expect("Could not parse s2s_endpoint"),
            random_minutes: random_minutes.unwrap_or_else(get_random_minutes),
        }
    }

    fn randomize_and_format_event_time(&self, original_event_time: OffsetDateTime) -> String {
        // Note this must be in the future or will fail CJ side
        // We add a random number of minutes and remove seconds and microseconds to enhance privacy
        let randomized_event_time = original_event_time + self.random_minutes;
        randomized_event_time.format("%FT%H:%M:00.000Z")
    }

    fn get_url_for_sub(&self, sub: &Subscription) -> Url {
        let event_time = self.randomize_and_format_event_time(sub.subscription_created);
        let mut url_for_sub = self.s2s_endpoint.clone();
        url_for_sub
            .query_pairs_mut()
            .append_pair("CID", &self.cj_cid)
            .append_pair("TYPE", &self.cj_type)
            .append_pair("SIGNATURE", &self.cj_signature)
            .append_pair("METHOD", "S2S")
            .append_pair(
                "CJEVENT",
                sub.cj_event_value.as_ref().unwrap_or(&String::from("n/a")),
            )
            .append_pair("EVENTTIME", &event_time)
            .append_pair("OID", &sub.id.to_string())
            .append_pair("CURRENCY", &sub.plan_currency)
            .append_pair("ITEM1", &sub.plan_id)
            .append_pair(
                "AMT1",
                &format!("{}", convert_amount_to_decimal(sub.plan_amount)),
            )
            .append_pair("QTY1", &format!("{}", sub.quantity))
            .append_pair(
                "CUST_COUNTRY",
                get_iso_code_3_from_iso_code_2(sub.country.as_ref().unwrap_or(&String::from(""))),
            )
            .append_pair("COUPON", sub.coupons.as_ref().unwrap_or(&String::from("")));
        url_for_sub
    }

    pub async fn report_subscription(&self, sub: &Subscription) -> Result<Response, Error> {
        let url_for_sub = self.get_url_for_sub(sub);
        self.client.get(url_for_sub).send().await
    }

    pub async fn query_commission_detail_api_between_dates(
        &self,
        min: OffsetDateTime,
        max: OffsetDateTime,
    ) -> CommissionDetailRecordSet {
        // We format to the beginning of the day of min and the beginning of the next day of max
        let format_string = "%FT00:00:00Z";
        let since = min.format(format_string);
        let before = (max + Duration::days(1)).format(format_string);
        // Format query
        let query = format!(
            r#"{{
        advertiserCommissions(
            forAdvertisers: ["{}"],
            sincePostingDate:"{}",
            beforePostingDate:"{}",
        ) {{
            count
            records {{
                original
                orderId
                correctionReason
                coupon
                saleAmountPubCurrency
                items {{
                    sku
                }}
            }}
        }}}}"#,
            self.advertiser_id, since, before
        );
        info!(
            LogKey::VerifyReportsQuery,
            query = query.as_str(),
            "Query sent to CommissionDetail API"
        );
        // Make request
        let resp = self
            .client
            .post(self.commission_detail_endpoint.clone())
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.commission_detail_api_token.expose_secret()
                ),
            )
            .json(&json!({ "query": query }))
            .send()
            .await
            .expect("Call to CJ failed.");
        if resp.status() != 200 {
            panic!("CJ did not return a 200. {:?}", resp)
        }
        // Parse and handle the response
        let query_result: CommissionDetailQueryResponse = match resp.json().await {
            Ok(data) => data,
            Err(e) => {
                panic!("Could not deserialize data from CJ call. {}", e);
            }
        };
        match query_result.data {
            Some(data) => {
                info!(
                    LogKey::VerifyReports,
                    "Successfully received data from CommissionDetail API."
                );
                data.advertiser_commissions
            }
            None => match query_result.errors {
                Some(error) => {
                    panic!("Got no data from CJ. {}", error);
                }
                None => {
                    panic!("Got no data and no errors from CJ.");
                }
            },
        }
    }
}

#[cfg(test)]
pub mod test_telemetry {
    use super::*;

    #[test]
    fn test_convert_plan_amount_to_decimal() {
        assert_eq!(convert_amount_to_decimal(999), 9.99);
        assert_eq!(convert_amount_to_decimal(5988), 59.88);
    }
}

#[cfg(test)]
mod tests {

    use time::{date, time, PrimitiveDateTime};

    use super::*;
    use crate::{
        models::subscriptions::test_subscriptions::make_fake_sub, test_utils::empty_settings,
    };

    #[test]
    fn commission_detail_record_parses_f32() {
        let json = json!({
            "original": true,
            "orderId": "abc123",
            "saleAmountPubCurrency": "9.99",
            "items": vec![json!({"sku": "abc123"})],
        });
        let result = serde_json::from_value::<CommissionDetailRecord>(json).unwrap();
        assert_eq!(result.sale_amount_pub_currency, 9.99f32)
    }

    #[test]
    fn commission_detail_record_parses_f32_err_on_invalid_value() {
        let json = json!({
            "original": true,
            "orderId": "abc123",
            "saleAmountPubCurrency": "notgood",
            "items": vec![json!({"sku": "abc123"})],
        });
        let result = serde_json::from_value::<CommissionDetailRecord>(json);
        assert!(result.is_err());
    }

    #[test]
    fn random_minutes_should_be_set_on_cjclient_if_passed() {
        // This is used for tests settings
        let settings = empty_settings();
        let cj = CJClient::new(&settings, None, None, Some(Duration::minutes(88)));
        assert_eq!(cj.random_minutes.whole_minutes(), 88);
    }

    #[test]
    fn random_minutes_should_be_greated_than_15_and_less_than_60_if_on_cjclient_if_not_passed() {
        let settings = empty_settings();
        for _ in 0..10 {
            let cj = CJClient::new(&settings, None, None, None);
            let minutes = cj.random_minutes.whole_minutes();
            assert!(
                minutes >= 15,
                "minutes was {} - should be greater than 15",
                minutes
            );
            assert!(
                minutes <= 60,
                "minutes was {} - should be less than 60",
                minutes
            );
        }
    }

    #[test]
    fn randomize_and_format_event_time_adds_minutes_and_formats_string_correctly() {
        let settings = empty_settings();
        let cj = CJClient::new(&settings, None, None, Some(Duration::minutes(9)));
        let event_time =
            PrimitiveDateTime::new(date!(2019 - 01 - 01), time!(11:11:11.111111)).assume_utc();
        let result = cj.randomize_and_format_event_time(event_time);
        assert_eq!(result, "2019-01-01T11:20:00.000Z");
    }

    #[test]
    fn event_time_in_url_should_by_randomized_by_duration() {
        let mut sub = make_fake_sub();
        sub.subscription_created =
            PrimitiveDateTime::new(date!(2021 - 12 - 31), time!(23:59:59.999999)).assume_utc();
        let settings = empty_settings();
        let cj = CJClient::new(&settings, None, None, Some(Duration::minutes(44)));
        let url = cj.get_url_for_sub(&sub);
        for (key, value) in url.query_pairs() {
            if key == "EVENTTIME" {
                assert_eq!(value, "2022-01-01T00:43:00.000Z");
            }
        }
    }
}
