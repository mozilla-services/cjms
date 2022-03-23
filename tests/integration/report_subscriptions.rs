use lib::{
    cj::{client::CJS2SClient, country_codes::get_iso_code_3_from_iso_code_2},
    models::subscriptions::SubscriptionModel,
    report_subscriptions::report_subscriptions_to_cj,
    settings::{get_settings, Settings},
};
use serde_json::json;
use time::{Duration, OffsetDateTime};
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockBuilder, MockServer, ResponseTemplate,
};

use crate::{models::subscriptions::make_fake_sub, utils::get_db_pool};

fn when_sending_to_cj(settings: &Settings) -> MockBuilder {
    Mock::given(path("/"))
        .and(method("GET"))
        .and(query_param("CID", &settings.cj_cid))
        .and(query_param("TYPE", &settings.cj_type))
        .and(query_param("SIGNATURE", &settings.cj_signature))
        .and(query_param("METHOD", "S2S"))
}

#[tokio::test]
async fn report_subscriptions() {
    // SETUP

    let settings = get_settings();
    let db_pool = get_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };

    // Sub 1 - should be reported
    let mut sub_1 = make_fake_sub();
    sub_1.flow_id = "1".to_string();
    sub_1.status = Some(String::from("not_reported"));
    sub_1.status_history = Some(json!([{
        "status": "not_reported",
        "t": OffsetDateTime::now_utc() - Duration::seconds(10)
    }]));
    sub_1.subscription_created =
        OffsetDateTime::parse("2021-12-25 14:22:33 +0000", "%F %T %z").unwrap();
    sub_1.aic_expires = Some(OffsetDateTime::now_utc() + Duration::days(10));
    // Sub 2 - should be "will_not_report" (because aic_expires before subscription created)
    let mut sub_2 = make_fake_sub();
    sub_2.flow_id = "2".to_string();
    sub_2.status = Some(String::from("not_reported"));
    sub_2.status_history = Some(json!([{
        "status": "not_reported",
        "t": OffsetDateTime::now_utc() - Duration::seconds(20)
    }]));
    sub_2.subscription_created = OffsetDateTime::now_utc() - Duration::days(1);
    sub_2.aic_expires = Some(OffsetDateTime::now_utc() - Duration::days(3));
    // Sub 3 - should be reported but will fail because mock cj fails
    let mut sub_3 = make_fake_sub();
    sub_3.flow_id = "3".to_string();
    sub_3.status = Some(String::from("not_reported"));
    sub_3.status_history = Some(json!([{
        "status": "not_reported",
        "t": OffsetDateTime::now_utc() - Duration::seconds(20)
    }]));
    sub_3.subscription_created = OffsetDateTime::now_utc() - Duration::days(5);
    sub_3.aic_expires = Some(OffsetDateTime::now_utc() + Duration::days(10));
    // Sub 4 - should be reported (no country)
    let mut sub_4 = make_fake_sub();
    sub_4.flow_id = "4".to_string();
    sub_4.status = Some(String::from("not_reported"));
    sub_4.status_history = Some(json!([{
        "status": "not_reported",
        "t": OffsetDateTime::now_utc() - Duration::seconds(10)
    }]));
    sub_4.subscription_created = OffsetDateTime::now_utc() - Duration::days(5);
    sub_4.aic_expires = Some(OffsetDateTime::now_utc() + Duration::days(10));
    sub_4.country = None;

    for sub in [&sub_1, &sub_2, &sub_3, &sub_4] {
        sub_model
            .create_from_sub(sub)
            .await
            .expect("Failed to create sub.");
    }

    let mock_cj = MockServer::start().await;
    when_sending_to_cj(&settings)
        .and(query_param("CJEVENT", sub_1.cj_event_value.unwrap()))
        .and(query_param("EVENTTIME", "2021-12-25T14:00:00.000Z"))
        .and(query_param("OID", sub_1.id.to_string()))
        .and(query_param("CURRENCY", sub_1.plan_currency))
        .and(query_param("ITEM1", sub_1.plan_id))
        .and(query_param("AMT1", sub_1.plan_amount.to_string()))
        .and(query_param("QTY1", sub_1.quantity.to_string()))
        .and(query_param(
            "CUST_COUNTRY",
            get_iso_code_3_from_iso_code_2(sub_1.country.as_ref().unwrap()),
        ))
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&mock_cj)
        .await;
    when_sending_to_cj(&settings)
        .and(query_param("CJEVENT", sub_3.cj_event_value.unwrap()))
        .and(query_param(
            "EVENTTIME",
            sub_3.subscription_created.format("%FT%H:00:00.000Z"),
        ))
        .and(query_param("OID", sub_3.id.to_string()))
        .and(query_param("CURRENCY", sub_3.plan_currency))
        .and(query_param("ITEM1", sub_3.plan_id))
        .and(query_param("AMT1", format!("{}", sub_3.plan_amount)))
        .and(query_param("QTY1", format!("{}", sub_3.quantity)))
        .and(query_param(
            "CUST_COUNTRY",
            get_iso_code_3_from_iso_code_2(sub_2.country.as_ref().unwrap()),
        ))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&mock_cj)
        .await;
    when_sending_to_cj(&settings)
        .and(query_param("CJEVENT", sub_4.cj_event_value.unwrap()))
        .and(query_param(
            "EVENTTIME",
            sub_4.subscription_created.format("%FT%H:00:00Z"),
        ))
        .and(query_param("OID", sub_4.id.to_string()))
        .and(query_param("CURRENCY", sub_4.plan_currency))
        .and(query_param("ITEM1", sub_4.plan_id))
        .and(query_param("AMT1", format!("{}", sub_4.plan_amount)))
        .and(query_param("QTY1", format!("{}", sub_4.quantity)))
        .and(query_param("CUST_COUNTRY", "N/A"))
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&mock_cj)
        .await;
    let mock_cj_client = CJS2SClient::new(&settings, Some(&mock_cj.uri()));

    // GO
    report_subscriptions_to_cj(&db_pool, mock_cj_client).await;

    // ASSERT

    let sub_1_updated = sub_model
        .fetch_one_by_id(&sub_1.id)
        .await
        .expect("Could not get sub");
    let sub_2_updated = sub_model
        .fetch_one_by_id(&sub_2.id)
        .await
        .expect("Could not get sub");
    let sub_3_updated = sub_model
        .fetch_one_by_id(&sub_3.id)
        .await
        .expect("Could not get sub");
    let sub_4_updated = sub_model
        .fetch_one_by_id(&sub_4.id)
        .await
        .expect("Could not get sub");

    assert_eq!(sub_1_updated.status.unwrap(), "reported");
    assert_eq!(sub_2_updated.status.unwrap(), "will_not_report");
    assert_eq!(sub_3_updated.status.unwrap(), "not_reported");
    assert_eq!(sub_4_updated.status.unwrap(), "reported");
}
