use crate::{models::subscriptions::Subscription, settings::Settings};
use reqwest::{Client, Error, Response, Url};

use super::country_codes::get_iso_code_3_from_iso_code_2;

pub struct CJS2SClient {
    url: Url,
    client: reqwest::Client,
    cj_cid: String,
    cj_type: String,
    cj_signature: String,
}

impl CJS2SClient {
    pub fn new(settings: &Settings, cj_endpoint: Option<&str>) -> CJS2SClient {
        let cj_endpoint = cj_endpoint.unwrap_or("https://www.emjcd.com/u");
        CJS2SClient {
            url: Url::parse(cj_endpoint).expect("Could not parse cj_endpoint"),
            client: Client::new(),
            cj_cid: settings.cj_cid.clone(),
            cj_type: settings.cj_type.clone(),
            cj_signature: settings.cj_signature.clone(),
        }
    }

    pub async fn report_subscription(&self, sub: &Subscription) -> Result<Response, Error> {
        let mut url_for_sub = self.url.clone();
        let event_time = sub.subscription_created.format("%FT%H:00:00.000Z");
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
            .append_pair("AMT1", &format!("{}", sub.plan_amount))
            .append_pair("QTY1", &format!("{}", sub.quantity))
            .append_pair(
                "CUST_COUNTRY",
                get_iso_code_3_from_iso_code_2(sub.country.as_ref().unwrap_or(&String::from(""))),
            );
        self.client.get(url_for_sub).send().await
    }
}
