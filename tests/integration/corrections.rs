use crate::utils::spawn_app;

#[tokio::test]
async fn test_corrections_today_path() {
    let app = spawn_app().await;
    let path = app.build_url("/corrections/today.csv");
    let client = reqwest::Client::new();
    let r = client.get(&path).send().await.expect("Failed to GET");
    assert_eq!(r.status(), 200);
}

#[tokio::test]
async fn test_corrections_by_day_auth() {
    let app = spawn_app().await;
    let path = app.build_url("/corrections/20220328.csv");
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
    assert_eq!(r.text().await.unwrap(), "20220328");
}
