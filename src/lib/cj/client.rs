use crate::{models::subscriptions::Subscription, settings::Settings};
use rand::{thread_rng, Rng};
use reqwest::{Client, Error, Response, Url};
use time::{Duration, OffsetDateTime};

use super::country_codes::get_iso_code_3_from_iso_code_2;

pub struct CJS2SClient {
    url: Url,
    client: reqwest::Client,
    cj_cid: String,
    cj_type: String,
    cj_signature: String,
    random_minutes: Duration,
}

fn get_random_minutes() -> Duration {
    let mut rng = thread_rng();
    let minutes = rng.gen_range(15..=60);
    Duration::minutes(minutes)
}

impl CJS2SClient {
    pub fn new(
        settings: &Settings,
        cj_endpoint: Option<&str>,
        random_minutes: Option<Duration>,
    ) -> CJS2SClient {
        let cj_endpoint = cj_endpoint.unwrap_or("https://www.emjcd.com/u");
        CJS2SClient {
            url: Url::parse(cj_endpoint).expect("Could not parse cj_endpoint"),
            client: Client::new(),
            cj_cid: settings.cj_cid.clone(),
            cj_type: settings.cj_type.clone(),
            cj_signature: settings.cj_signature.clone(),
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

        let mut url_for_sub = self.url.clone();
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
        url_for_sub
    }

    pub async fn report_subscription(&self, sub: &Subscription) -> Result<Response, Error> {
        let url_for_sub = self.get_url_for_sub(sub);
        self.client.get(url_for_sub).send().await
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
    fn random_minutes_should_be_set_on_cjclient_if_passed() {
        // This is used for tests settings
        let settings = empty_settings();
        let cj = CJS2SClient::new(&settings, None, Some(Duration::minutes(88)));
        assert_eq!(cj.random_minutes.whole_minutes(), 88);
    }

    #[test]
    fn random_minutes_should_be_greated_than_15_and_less_than_60_if_on_cjclient_if_not_passed() {
        let settings = empty_settings();
        for _ in 0..10 {
            let cj = CJS2SClient::new(&settings, None, None);
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
        let cj = CJS2SClient::new(&settings, None, Some(Duration::minutes(9)));
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
        let cj = CJS2SClient::new(&settings, None, Some(Duration::minutes(44)));
        let url = cj.get_url_for_sub(&sub);
        for (key, value) in url.query_pairs() {
            if key == "EVENTTIME" {
                assert_eq!(value, "2022-01-01T00:43:00.000Z");
            }
        }
    }
}
