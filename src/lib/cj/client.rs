use crate::{models::subscriptions::Subscription, settings::Settings};
use rand::{thread_rng, Rng};
use reqwest::{Client, Error, Response, Url};
use time::Duration;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::country_codes::get_iso_code_3_from_iso_code_2;

pub struct CJClient {
    client: reqwest::Client,
    cj_cid: String,
    cj_type: String,
    cj_signature: String,
    commission_detail_endpoint: Url,
    s2s_endpoint: Url,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommissionDetailItem {
    sku: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionDetailRecord {
    original: bool,
    order_id: String,
    correction_reason: Option<String>,
    sale_amount_pub_currency: f64,
    items: Vec<CommissionDetailItem>,
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
            .append_pair("AMT1", &format!("{}", sub.plan_amount as f32 / 100.0))
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
    ) -> Result<Response, Error> {
        let query = format!(
            r#"
advertiserCommissions(
    forAdvertisers: ["123456"],
    sincePostingDate:"{}",
    beforePostingDate:"{}",
) {{
    count
}})
"#,
            min, max
        );
        self.client
            .post(self.commission_detail_endpoint.clone())
            .json(&query)
            .send()
            .await
    }
}
