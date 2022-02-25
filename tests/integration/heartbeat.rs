use crate::utils::spawn_app;

#[tokio::test]
async fn index_get() {
    let app = spawn_app().await;
    let path = app.build_url("/");
    let r = reqwest::get(path).await.expect("Failed to execute request");
    assert_eq!(r.status(), 200);
    let body = r.text().await.expect("Response body missing.");
    assert_eq!(body, "Hello world!");
}

#[tokio::test]
async fn heartbeats_get() {
    let app = spawn_app().await;
    let test_cases = vec!["/__heartbeat__", "/__lbheartbeat__"];
    for path in test_cases {
        let path = app.build_url(path);
        let r = reqwest::get(&path)
            .await
            .expect("Failed to execute request");
        assert_eq!(r.status(), 200, "Failed on path: {}", path);
    }
}
