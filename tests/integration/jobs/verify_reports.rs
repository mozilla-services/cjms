use crate::{
    models::{refunds::make_fake_refund, subscriptions::make_fake_sub},
    utils::get_test_db_pool,
};
use lib::{
    cj::client::{convert_amount_to_decimal, CJClient},
    jobs::verify_reports::verify_reports_with_cj,
    models::{
        refunds::{Refund, RefundModel},
        status_history::{Status, StatusHistoryEntry, UpdateStatus},
        subscriptions::{Subscription, SubscriptionModel},
    },
    settings::{get_settings, Settings},
    telemetry::StatsD,
};
use serde_json::{json, Value};
use time::{Duration, OffsetDateTime};
use wiremock::{
    matchers::{body_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

struct VerifyReportsTestSetup {
    sub_1: Subscription,
    sub_2: Subscription,
    sub_3: Subscription,
    sub_4: Subscription,
    sub_5: Subscription,
    refund_1: Refund,
    refund_2: Refund,
    refund_3: Refund,
    refund_4: Refund,
    refund_5: Refund,
    required_query: String,
    response_body: Value,
}

fn make_refund_amount(amount: i32) -> f32 {
    (-1.00 * convert_amount_to_decimal(amount)) as f32
}

async fn setup_test(
    settings: &Settings,
    sub_model: &SubscriptionModel<'_>,
    refund_model: &RefundModel<'_>,
) -> VerifyReportsTestSetup {
    let now = OffsetDateTime::now_utc();
    let min_sub = now - Duration::hours(48);
    let min_refund = now - Duration::hours(72);

    // Sub 1 - Reported, expect to have been recieved by CJ
    let mut sub_1 = make_fake_sub();
    sub_1.update_status(Status::Reported);
    // Sub 2 - Reported 48 hours ago, CJ has the wrong amount
    let mut sub_2 = make_fake_sub();
    sub_2.update_status(Status::Reported);
    sub_2.set_status_t(Some(min_sub));
    // Sub 3 - Reported 48 hours ago, CJ has the wrong sku
    let mut sub_3 = make_fake_sub();
    sub_3.update_status(Status::Reported);
    sub_3.set_status_t(Some(min_sub));
    // Sub 4 - Reported 48 hours ago (> 36 hours ago), CJ has the wrong id - mark CJNotReceived
    let mut sub_4 = make_fake_sub();
    sub_4.update_status(Status::Reported);
    sub_4.set_status_t(Some(min_sub));
    // Sub 5 - Reported < 36 hours ago, CJ has the wrong id - leave as Reported for now
    let mut sub_5 = make_fake_sub();
    sub_5.update_status(Status::Reported);
    sub_5.set_status_t(Some(now - Duration::hours(35)));

    for (i, sub) in [&sub_1, &sub_2, &sub_3, &sub_4, &sub_5].iter().enumerate() {
        println!("Sub {} - {}", i + 1, sub.id);
        sub_model
            .create_from_sub(sub)
            .await
            .expect("Failed to create sub.");
    }

    // Refund 1 - Reported, expect to have been recieved by CJ
    let mut refund_1 = make_fake_refund();
    refund_1.update_status(Status::Reported);
    let mut refund_1_sub = make_fake_sub();
    refund_1_sub.subscription_id = refund_1.subscription_id.clone();
    refund_1_sub.plan_amount = refund_1.refund_amount;
    // Refund 2 - Reported 48 hours ago, CJ has the wrong amount
    let mut refund_2 = make_fake_refund();
    refund_2.update_status(Status::Reported);
    refund_2.set_status_t(Some(min_refund));
    let mut refund_2_sub = make_fake_sub();
    refund_2_sub.subscription_id = refund_2.subscription_id.clone();
    refund_2_sub.plan_amount = refund_2.refund_amount;
    // Refund 3 - Reported 48 hours ago, CJ has the wrong sku
    let mut refund_3 = make_fake_refund();
    refund_3.update_status(Status::Reported);
    refund_3.set_status_t(Some(min_refund));
    let mut refund_3_sub = make_fake_sub();
    refund_3_sub.subscription_id = refund_3.subscription_id.clone();
    refund_3_sub.plan_amount = refund_3.refund_amount;
    // Refund 4 - Reported 48 hours ago (> 36 hours ago), CJ has the wrong id - mark CJNotReceived
    let mut refund_4 = make_fake_refund();
    refund_4.update_status(Status::Reported);
    refund_4.set_status_t(Some(min_refund));
    let mut refund_4_sub = make_fake_sub();
    refund_4_sub.subscription_id = refund_4.subscription_id.clone();
    refund_4_sub.plan_amount = refund_4.refund_amount;
    // Refund 5 - Reported < 36 hours ago, CJ has the wrong id - leave as Reported for now
    let mut refund_5 = make_fake_refund();
    refund_5.update_status(Status::Reported);
    refund_5.set_status_t(Some(now - Duration::hours(35)));
    let mut refund_5_sub = make_fake_sub();
    refund_5_sub.subscription_id = refund_5.subscription_id.clone();
    refund_5_sub.plan_amount = refund_5.refund_amount;

    for refund in [&refund_1, &refund_2, &refund_3, &refund_4, &refund_5] {
        refund_model
            .create_from_refund(refund)
            .await
            .expect("Failed to create refund.");
    }
    for (i, sub) in [
        &refund_1_sub,
        &refund_2_sub,
        &refund_3_sub,
        &refund_4_sub,
        &refund_5_sub,
    ]
    .iter()
    .enumerate()
    {
        println!("Refund sub {} - {}", i + 1, sub.id);
        sub_model
            .create_from_sub(sub)
            .await
            .expect("Failed to create sub.");
    }

    let required_query = format!(
        r#"{{
        advertiserCommissions(
            forAdvertisers: ["{}"],
            sincePostingDate:"{}T00:00:00Z",
            beforePostingDate:"{}T00:00:00Z",
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
        settings.cj_sftp_user,
        min_refund.format("%F"), // because that's the furthest away
        (now + Duration::days(1)).format("%F")
    );
    println!("Expected query: {}", required_query);
    let response_body = json!(
        {"data":
            {"advertiserCommissions":
                {
                    "count": 6,
                    "records": [
                        {
                            "original": true,
                            "orderId": sub_1.id,
                            "correctionReason": null,
                            "saleAmountPubCurrency": convert_amount_to_decimal(sub_1.plan_amount),
                            "items": [
                                {
                                    "sku": sub_1.plan_id
                                }
                            ]
                        },
                        // This refund exists so that we can check that the subscription data correctly picks out original: true record above
                        {
                            "original": false,
                            "orderId": sub_1.id,
                            "correctionReason": "RETURNED_MERCHANDISE",
                            "saleAmountPubCurrency": make_refund_amount(sub_1.plan_amount),
                            "items": [
                                {
                                    "sku": sub_1.plan_id
                                }
                            ]
                        },
                        {
                            "original": true,
                            "orderId": sub_2.id,
                            "correctionReason": null,
                            "saleAmountPubCurrency": -999.99,
                            "items": [
                                {
                                    "sku": sub_2.plan_id
                                }
                            ]
                        },
                        {
                            "original": true,
                            "orderId": sub_3.id,
                            "correctionReason": null,
                            "saleAmountPubCurrency": convert_amount_to_decimal(sub_3.plan_amount),
                            "items": [
                                {
                                    "sku": "WRONG SKU"
                                }
                            ]
                        },
                        {
                            "original": true,
                            "orderId": "WRONGID",
                            "correctionReason": null,
                            "saleAmountPubCurrency": convert_amount_to_decimal(sub_4.plan_amount),
                            "items": [
                                {
                                    "sku": sub_4.plan_id
                                }
                            ]
                        },
                        {
                            "original": true,
                            "orderId": "WRONGID",
                            "correctionReason": null,
                            "saleAmountPubCurrency": convert_amount_to_decimal(sub_5.plan_amount),
                            "items": [
                                {
                                    "sku": sub_5.plan_id
                                }
                            ]
                        },
                        // ------------
                        // ------------
                        // ------------
                        // ------------ REFUND ENTRIES
                        // This subscription exists so that we can check that the refund check correctly picks out original: false record
                        {
                            "original": true,
                            "orderId": refund_1_sub.id,
                            "correctionReason": null,
                            "saleAmountPubCurrency": convert_amount_to_decimal(refund_1_sub.plan_amount),
                            "items": [
                                {
                                    "sku": refund_1_sub.plan_id
                                }
                            ]
                        },
                        {
                            "original": false,
                            "orderId": refund_1_sub.id,
                            "correctionReason": "RETURNED_MERCHANDISE",
                            "saleAmountPubCurrency": make_refund_amount(refund_1.refund_amount),
                            "items": [
                                {
                                    "sku": refund_1_sub.plan_id
                                }
                            ]
                        },
                        {
                            "original": false,
                            "orderId": refund_2_sub.id,
                            "correctionReason": "RETURNED_MERCHANDISE",
                            "saleAmountPubCurrency": 999.99,
                            "items": [
                                {
                                    "sku": refund_2_sub.plan_id
                                }
                            ]
                        },
                        {
                            "original": true,
                            "orderId": refund_3_sub.id,
                            "correctionReason": "RETURNED_MERCHANDISE",
                            "saleAmountPubCurrency": make_refund_amount(refund_3.refund_amount),
                            "items": [
                                {
                                    "sku": "WRONG SKU"
                                }
                            ]
                        },
                        {
                            "original": true,
                            "orderId": "WRONGID",
                            "correctionReason": "RETURNED_MERCHANDISE",
                            "saleAmountPubCurrency": make_refund_amount(refund_4.refund_amount),
                            "items": [
                                {
                                    "sku": refund_4_sub.plan_id
                                }
                            ]
                        },
                        {
                            "original": true,
                            "orderId": "WRONGID",
                            "correctionReason": "RETURNED_MERCHANDISE",
                            "saleAmountPubCurrency": make_refund_amount(refund_5.refund_amount),
                            "items": [
                                {
                                    "sku": refund_5_sub.plan_id
                                }
                            ]
                        },
                    ]
                }
            }
        }
    );
    VerifyReportsTestSetup {
        sub_1,
        sub_2,
        sub_3,
        sub_4,
        sub_5,
        refund_1,
        refund_2,
        refund_3,
        refund_4,
        refund_5,
        required_query,
        response_body,
    }
}

#[tokio::test]
#[should_panic(expected = "Got no data from CJ. [{\"locations\":[{\"colu")]
async fn test_when_cj_sends_errors() {
    let settings = get_settings();
    let mock_statsd = StatsD::new(&settings);
    let db_pool = get_test_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };
    let refund_model = RefundModel { db_pool: &db_pool };
    let _ = setup_test(&settings, &sub_model, &refund_model).await;
    let mock_cj = MockServer::start().await;
    let response_body = json!({
      "data": null,
      "errors": [
        {
          "message": "Read timed out",
          "path": [
            "advertiserCommissions"
          ],
          "locations": [
            {
              "line": 32,
              "column": 3
            }
          ]
        }
      ]
    });
    let response = ResponseTemplate::new(200).set_body_json(response_body);
    Mock::given(path("/"))
        .respond_with(response)
        .expect(1)
        .mount(&mock_cj)
        .await;
    let mock_cj_client = CJClient::new(&settings, None, Some(&mock_cj.uri()), None);

    // GO
    verify_reports_with_cj(&db_pool, &mock_cj_client, &mock_statsd).await;
}

#[tokio::test]
async fn test_correct_and_incorrectly_received_subscriptions_are_handled_correctly() {
    // SETUP
    let settings = get_settings();
    let mock_statsd = StatsD::new(&settings);
    let db_pool = get_test_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };
    let refund_model = RefundModel { db_pool: &db_pool };
    let test_setup = setup_test(&settings, &sub_model, &refund_model).await;
    let sub_1 = test_setup.sub_1;
    let sub_2 = test_setup.sub_2;
    let sub_3 = test_setup.sub_3;
    let sub_4 = test_setup.sub_4;
    let sub_5 = test_setup.sub_5;
    let mock_cj = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(test_setup.response_body);
    Mock::given(path("/"))
        .and(method("POST"))
        .and(header(
            "Authorization",
            format!("Bearer {}", settings.cj_api_access_token).as_str(),
        ))
        .and(body_json(&json!({ "query": test_setup.required_query })))
        .respond_with(response)
        .expect(1)
        .mount(&mock_cj)
        .await;
    let mock_cj_client = CJClient::new(&settings, None, Some(&mock_cj.uri()), None);

    // GO
    let now = OffsetDateTime::now_utc();
    verify_reports_with_cj(&db_pool, &mock_cj_client, &mock_statsd).await;

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
    let sub_5_updated = sub_model
        .fetch_one_by_id(&sub_5.id)
        .await
        .expect("Could not get sub");

    println!("Testing sub: {}", sub_1_updated.id);
    assert_eq!(sub_1_updated.get_status().unwrap(), Status::CJReceived);
    let updated_history = sub_1_updated.get_status_history().unwrap();
    assert_eq!(updated_history.entries.len(), 3);
    assert_eq!(
        updated_history.entries[2],
        StatusHistoryEntry {
            status: Status::CJReceived,
            t: now
        }
    );
    for not_found_sub in [&sub_2_updated, &sub_3_updated, &sub_4_updated] {
        println!("Testing sub: {}", not_found_sub.id);
        assert_eq!(not_found_sub.get_status().unwrap(), Status::CJNotReceived);
        let updated_history = not_found_sub.get_status_history().unwrap();
        assert_eq!(updated_history.entries.len(), 3);
        assert_eq!(
            updated_history.entries[2],
            StatusHistoryEntry {
                status: Status::CJNotReceived,
                t: now
            }
        );
    }
    // Leave unchanged as we'll try again to see if the report comes through
    println!("Testing sub: {}", sub_5_updated.id);
    assert_eq!(sub_5_updated.get_status().unwrap(), Status::Reported);
    let updated_history = sub_5_updated.get_status_history().unwrap();
    assert_eq!(updated_history.entries.len(), 2);
}

#[tokio::test]
async fn test_correct_and_incorrectly_received_refunds_are_handled_correctly() {
    // SETUP
    let settings = get_settings();
    let mock_statsd = StatsD::new(&settings);
    let db_pool = get_test_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };
    let refund_model = RefundModel { db_pool: &db_pool };
    let test_setup = setup_test(&settings, &sub_model, &refund_model).await;
    let refund_1 = test_setup.refund_1;
    let refund_2 = test_setup.refund_2;
    let refund_3 = test_setup.refund_3;
    let refund_4 = test_setup.refund_4;
    let refund_5 = test_setup.refund_5;
    let mock_cj = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(test_setup.response_body);
    Mock::given(path("/"))
        .and(method("POST"))
        .and(header(
            "Authorization",
            format!("Bearer {}", settings.cj_api_access_token).as_str(),
        ))
        .and(body_json(&json!({ "query": test_setup.required_query })))
        .respond_with(response)
        .expect(1)
        .mount(&mock_cj)
        .await;
    let mock_cj_client = CJClient::new(&settings, None, Some(&mock_cj.uri()), None);

    // GO
    let now = OffsetDateTime::now_utc();
    verify_reports_with_cj(&db_pool, &mock_cj_client, &mock_statsd).await;

    // ASSERT
    let refund_1_updated = refund_model
        .fetch_one_by_refund_id(&refund_1.refund_id)
        .await
        .expect("Could not get refund");
    let refund_2_updated = refund_model
        .fetch_one_by_refund_id(&refund_2.refund_id)
        .await
        .expect("Could not get refund");
    let refund_3_updated = refund_model
        .fetch_one_by_refund_id(&refund_3.refund_id)
        .await
        .expect("Could not get refund");
    let refund_4_updated = refund_model
        .fetch_one_by_refund_id(&refund_4.refund_id)
        .await
        .expect("Could not get refund");
    let refund_5_updated = refund_model
        .fetch_one_by_refund_id(&refund_5.refund_id)
        .await
        .expect("Could not get refund");

    println!("Testing refund: {}", refund_1_updated.id);
    assert_eq!(refund_1_updated.get_status().unwrap(), Status::CJReceived);
    let updated_history = refund_1_updated.get_status_history().unwrap();
    assert_eq!(updated_history.entries.len(), 3);
    assert_eq!(
        updated_history.entries[2],
        StatusHistoryEntry {
            status: Status::CJReceived,
            t: now
        }
    );
    for not_found_refund in [&refund_2_updated, &refund_3_updated, &refund_4_updated] {
        println!("Testing refund: {}", not_found_refund.id);
        assert_eq!(
            not_found_refund.get_status().unwrap(),
            Status::CJNotReceived
        );
        let updated_history = not_found_refund.get_status_history().unwrap();
        assert_eq!(updated_history.entries.len(), 3);
        assert_eq!(
            updated_history.entries[2],
            StatusHistoryEntry {
                status: Status::CJNotReceived,
                t: now
            }
        );
    }
    // Leave unchanged as we'll try again to see if the report comes through
    println!("Testing refund: {}", refund_5_updated.id);
    assert_eq!(refund_5_updated.get_status().unwrap(), Status::Reported);
    let updated_history = refund_5_updated.get_status_history().unwrap();
    assert_eq!(updated_history.entries.len(), 2);
}

#[tokio::test]
async fn test_correct_when_only_one_sub() {
    // SETUP
    let settings = get_settings();
    let mock_statsd = StatsD::new(&settings);
    let db_pool = get_test_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };

    let mut sub_1 = make_fake_sub();
    sub_1.update_status(Status::Reported);
    sub_model
        .create_from_sub(&sub_1)
        .await
        .expect("Failed to create sub.");
    let response_body = json!(
        {"data":
            {"advertiserCommissions":
                {
                    "count": 1,
                    "records": [
                        {
                            "original": true,
                            "orderId": sub_1.id,
                            "correctionReason": null,
                            "saleAmountPubCurrency": convert_amount_to_decimal(sub_1.plan_amount),
                            "items": [
                                {
                                    "sku": sub_1.plan_id
                                }
                            ]
                        }
                    ]
                }
            }
        }
    );
    let mock_cj = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(response_body);
    Mock::given(path("/"))
        .respond_with(response)
        .expect(1)
        .mount(&mock_cj)
        .await;
    let mock_cj_client = CJClient::new(&settings, None, Some(&mock_cj.uri()), None);

    // GO
    verify_reports_with_cj(&db_pool, &mock_cj_client, &mock_statsd).await;

    // ASSERT
    let sub_1_updated = sub_model
        .fetch_one_by_id(&sub_1.id)
        .await
        .expect("Could not get sub");
    assert_eq!(sub_1_updated.get_status().unwrap(), Status::CJReceived);
}

#[tokio::test]
async fn test_correct_when_only_one_refund() {
    // SETUP
    let settings = get_settings();
    let mock_statsd = StatsD::new(&settings);
    let db_pool = get_test_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };
    let refund_model = RefundModel { db_pool: &db_pool };

    let mut refund_1 = make_fake_refund();
    refund_1.update_status(Status::Reported);
    let mut related_sub = make_fake_sub();
    related_sub.subscription_id = refund_1.subscription_id.clone();
    refund_model
        .create_from_refund(&refund_1)
        .await
        .expect("Failed to create refund.");
    sub_model
        .create_from_sub(&related_sub)
        .await
        .expect("Failed to create sub.");
    let response_body = json!(
        {"data":
            {"advertiserCommissions":
                {
                    "count": 1,
                    "records": [
                        {
                            "original": false,
                            "orderId": related_sub.id,
                            "correctionReason": "RETURNED_MERCHANDISE",
                            "saleAmountPubCurrency": make_refund_amount(refund_1.refund_amount),
                            "items": [
                                {
                                    "sku": related_sub.plan_id
                                }
                            ]
                        }
                    ]
                }
            }
        }
    );
    let mock_cj = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(response_body);
    Mock::given(path("/"))
        .respond_with(response)
        .expect(1)
        .mount(&mock_cj)
        .await;
    let mock_cj_client = CJClient::new(&settings, None, Some(&mock_cj.uri()), None);

    // GO
    verify_reports_with_cj(&db_pool, &mock_cj_client, &mock_statsd).await;

    // ASSERT
    let refund_1_updated = refund_model
        .fetch_one_by_refund_id(&refund_1.refund_id)
        .await
        .expect("Could not get refund");
    assert_eq!(refund_1_updated.get_status().unwrap(), Status::CJReceived);
}

#[tokio::test]
async fn test_graceful_exit_when_nothing_to_check() {
    // SETUP
    let settings = get_settings();
    let mock_statsd = StatsD::new(&settings);
    let db_pool = get_test_db_pool().await;
    let response_body = json!(
        {"data":
            {"advertiserCommissions":
                {
                    "count": 0,
                    "records": []
                }
            }
        }
    );
    let mock_cj = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(response_body);
    Mock::given(path("/"))
        .respond_with(response)
        .expect(0)
        .mount(&mock_cj)
        .await;
    let mock_cj_client = CJClient::new(&settings, None, Some(&mock_cj.uri()), None);

    // GO
    verify_reports_with_cj(&db_pool, &mock_cj_client, &mock_statsd).await;
}
