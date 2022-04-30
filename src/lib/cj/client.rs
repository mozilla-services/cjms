use crate::telemetry::LogKey;
use crate::{error, info};
use crate::{models::subscriptions::Subscription, settings::Settings};
use rand::{thread_rng, Rng};
use reqwest::{Client, Error, Response, Url};
use serde::Deserialize;
use serde_json::{json, Value};
use time::Duration;
use time::OffsetDateTime;

use super::country_codes::get_iso_code_3_from_iso_code_2;

pub struct CJClient {
    client: reqwest::Client,
    cj_cid: String,
    cj_type: String,
    cj_signature: String,
    commission_detail_endpoint: Url,
    commission_detail_api_token: String,
    s2s_endpoint: Url,
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

pub fn convert_plan_amount_to_decimal(plan_amount: i32) -> f32 {
    plan_amount as f32 / 100.0
}

impl CJClient {
    pub fn new(
        settings: &Settings,
        s2s_endpoint: Option<&str>,
        commission_detail_endpoint: Option<&str>,
    ) -> CJClient {
        let s2s_endpoint = s2s_endpoint.unwrap_or("https://www.emjcd.com/u");
        let commission_detail_endpoint =
            commission_detail_endpoint.unwrap_or("https://commissions.api.cj.com/query");
        CJClient {
            client: Client::new(),
            cj_cid: settings.cj_cid.clone(),
            cj_type: settings.cj_type.clone(),
            cj_signature: settings.cj_signature.clone(),
            commission_detail_endpoint: Url::parse(commission_detail_endpoint)
                .expect("Could not parse commission_detail_endpoint"),
            commission_detail_api_token: settings.cj_api_access_token.clone(),
            s2s_endpoint: Url::parse(s2s_endpoint).expect("Could not parse s2s_endpoint"),
        }
    }

    fn random_minutes(&self) -> Duration {
        let mut rng = thread_rng();
        let minutes = rng.gen_range(15..=60);
        Duration::minutes(minutes)
    }

    pub async fn report_subscription(&self, sub: &Subscription) -> Result<Response, Error> {
        let mut url_for_sub = self.s2s_endpoint.clone();
        // Note this must be in the future or will fail CJ side
        let randomized_event_time = sub.subscription_created + self.random_minutes();
        let event_time = randomized_event_time.format("%FT%H:%M:00.000Z");
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
                &format!("{}", convert_plan_amount_to_decimal(sub.plan_amount)),
            )
            .append_pair("QTY1", &format!("{}", sub.quantity))
            .append_pair(
                "CUST_COUNTRY",
                get_iso_code_3_from_iso_code_2(sub.country.as_ref().unwrap_or(&String::from(""))),
            );
        self.client.get(url_for_sub).send().await
    }

    pub async fn query_comission_detail_api_between_dates(
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
            forAdvertisers: ["123456"],
            sincePostingDate:"{}",
            beforePostingDate:"{}",
        ) {{
            count
            records {{
                original
                orderId
                correctionReason
                saleAmountPubCurrency
                items {{
                    sku
                }}
            }}
        }}}}"#,
            since, before
        );
        // Make request
        let resp = self
            .client
            .post(self.commission_detail_endpoint.clone())
            .header(
                "Authorization",
                format!("Bearer {}", self.commission_detail_api_token),
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
                error!(
                    LogKey::VerifyReportsFailedDeserialization,
                    error = e,
                    "Could not deserialize data from CJ call."
                );
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
                    error!(
                        LogKey::VerifyReportsFailed,
                        error_json = error.to_string().as_str(),
                        "Got no data from CJ."
                    );
                    panic!("Got no data from CJ.");
                }
                None => {
                    error!(
                        LogKey::VerifyReportsFailedUnknown,
                        "Got no data and no errors from CJ."
                    );
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
        assert_eq!(convert_plan_amount_to_decimal(999), 9.99);
        assert_eq!(convert_plan_amount_to_decimal(5988), 59.88);
    }
}
