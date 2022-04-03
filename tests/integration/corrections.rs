use lib::models::{refunds::RefundModel, subscriptions::SubscriptionModel};
use reqwest::Response;
use time::{date, Date, OffsetDateTime};

use crate::{
    models::{
        refunds::{make_fake_refund, save_refund},
        subscriptions::{make_fake_sub, save_sub},
    },
    utils::spawn_app,
};

const ANOTHER_DAY: Date = date!(2021 - 11 - 07);

async fn make_test_refunds(refund_model: &RefundModel<'_>, sub_model: &SubscriptionModel<'_>) {
    let today = OffsetDateTime::now_utc().date();
    let mut refund_1 = make_fake_refund();
    refund_1.refund_id = "refund_1".to_string();
    refund_1.correction_file_date = Some(today);
    let mut sub_1 = make_fake_sub();
    sub_1.subscription_id = refund_1.subscription_id.clone();
    let mut refund_2 = make_fake_refund();
    refund_2.refund_id = "refund_2".to_string();
    refund_2.correction_file_date = Some(ANOTHER_DAY);
    let mut sub_2 = make_fake_sub();
    sub_2.subscription_id = refund_2.subscription_id.clone();
    let mut refund_3 = make_fake_refund();
    refund_3.correction_file_date = Some(today);
    refund_3.refund_id = "refund_3".to_string();
    let mut sub_3 = make_fake_sub();
    sub_3.subscription_id = refund_3.subscription_id.clone();
    let mut refund_4 = make_fake_refund();
    refund_4.correction_file_date = None;
    for r in [&refund_1, &refund_2, &refund_3, &refund_4] {
        save_refund(refund_model, r).await;
    }
    for s in [&sub_1, &sub_2, &sub_3] {
        save_sub(sub_model, s).await;
    }
}

#[tokio::test]
async fn test_corrections_by_day_auth() {
    let app = spawn_app().await;
    let path = app.build_url("/corrections/2022-03-28.csv");
    let client = reqwest::Client::new();
    // Bad auth - no auth
    let r = client.get(&path).send().await.expect("Failed to GET");
    assert_eq!(r.status(), 401,);
    // Bad auth - empty password
    let r = client
        .get(&path)
        .basic_auth("", Some(""))
        .send()
        .await
        .expect("Failed to GET");
    assert_eq!(r.status(), 401,);
    assert_eq!(r.text().await.unwrap(), "Password missing.");
    // Bad auth
    let r = client
        .get(&path)
        .basic_auth("", Some("not the password"))
        .send()
        .await
        .expect("Failed to GET");
    assert_eq!(r.status(), 401,);
    assert_eq!(r.text().await.unwrap(), "Incorrect password.");
    // Correct auth
    let r = client
        .get(&path)
        .basic_auth(
            "any user (we don't only check user)",
            Some(&app.settings.authentication),
        )
        .send()
        .await
        .expect("Failed to GET");
    assert_eq!(r.status(), 200);
}

async fn get_authed_path(path: &str, password: &str) -> Response {
    let client = reqwest::Client::new();
    let r = client
        .get(path)
        .basic_auth("user", Some(password))
        .send()
        .await
        .expect("Failed to GET");
    r
}

#[tokio::test]
async fn test_corrections_by_day_result() {
    let app = spawn_app().await;
    let refunds = RefundModel {
        db_pool: &app.db_connection(),
    };
    let subs = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    make_test_refunds(&refunds, &subs).await;
    let expected_refund = refunds.fetch_one_by_refund_id("refund_2").await.unwrap();
    let expected_sub = subs
        .fetch_one_by_subscription_id(&expected_refund.subscription_id)
        .await
        .unwrap();

    // Path with no expected refund
    let path = app.build_url("/corrections/2020-01-01.csv");
    let r = get_authed_path(&path, &app.settings.authentication).await;
    assert_eq!(r.status(), 200);
    let actual_body = r.text().await.unwrap();
    let expected_body = format!(
        r#"
&CID={}
&SUBID={}"#,
        app.settings.cj_cid, app.settings.cj_subid
    );
    assert_eq!(actual_body.trim(), expected_body.trim());

    // Path with expected refund
    let path = app.build_url("/corrections/2021-11-07.csv");
    let r = get_authed_path(&path, &app.settings.authentication).await;
    assert_eq!(r.status(), 200);
    let actual_body = r.text().await.unwrap();
    let expected_body = format!(
        r#"
&CID={}
&SUBID={}
RETRN,,{}"#,
        app.settings.cj_cid, app.settings.cj_subid, expected_sub.id
    );
    assert_eq!(actual_body.trim(), expected_body.trim());
}

#[tokio::test]
async fn test_corrections_today() {
    let app = spawn_app().await;
    let refunds = RefundModel {
        db_pool: &app.db_connection(),
    };
    let subs = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    make_test_refunds(&refunds, &subs).await;
    let expected_refund_1 = refunds.fetch_one_by_refund_id("refund_1").await.unwrap();
    let expected_refund_2 = refunds.fetch_one_by_refund_id("refund_3").await.unwrap();
    let expected_sub_1 = subs
        .fetch_one_by_subscription_id(&expected_refund_1.subscription_id)
        .await
        .unwrap();
    let expected_sub_2 = subs
        .fetch_one_by_subscription_id(&expected_refund_2.subscription_id)
        .await
        .unwrap();

    let path = app.build_url("/corrections/today.csv");
    let r = get_authed_path(&path, &app.settings.authentication).await;
    assert_eq!(r.status(), 200);
    let actual_body = r.text().await.unwrap();
    let expected_body = format!(
        r#"
&CID={}
&SUBID={}
RETRN,,{}
RETRN,,{}"#,
        app.settings.cj_cid, app.settings.cj_subid, expected_sub_1.id, expected_sub_2.id
    );
    assert_eq!(actual_body.trim(), expected_body.trim());
}
