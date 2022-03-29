use crate::utils::{send_get_request, spawn_app};

#[tokio::test]
async fn test_corrections_get_requires_basic_auth() {
    let app = spawn_app().await;
    let test_cases = ["/corrections", "/corrections/20220328-account_id"];
    for path in test_cases {
        let r = send_get_request(&app, path).await;
        assert_eq!(
            r.status(),
            403,
            "Expected Basic Auth to be required for {}",
            path
        );
        let path = app.build_url(path);
        let client = reqwest::Client::new();
        let r = client
            .get(&path)
            .basic_auth("corrections", app.settings.cj_auth)
            .send()
            .await
            .expect("Failed to GET");
        assert_eq!(r.status(), 200, "Could not access {}.", path);
    }
}
