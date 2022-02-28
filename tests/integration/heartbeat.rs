use crate::utils::{send_get_request, spawn_app};

#[tokio::test]
async fn index_get() {
    let app = spawn_app().await;
    let r = send_get_request(&app, "/").await;
    assert_eq!(r.status(), 200);
    let body = r.text().await.expect("Response body missing.");
    assert_eq!(body, "Hello world!");
}

#[tokio::test]
async fn heartbeats_get() {
    let app = spawn_app().await;
    let test_cases = ["/__heartbeat__", "/__lbheartbeat__"];
    for path in test_cases {
        let r = send_get_request(&app, path).await;
        assert_eq!(r.status(), 200, "Failed on path: {}", path);
    }
}
