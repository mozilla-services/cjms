use crate::utils::spawn_app;

#[tokio::test]
async fn test_corrections_has_id_in_path() {
    let app = spawn_app().await;
    let path = "/corrections/wrong/today.csv";
    let path = app.build_url(path);
    let client = reqwest::Client::new();
    let r = client.get(&path).send().await.expect("Failed to GET");
    assert_eq!(r.status(), 404);

    let path = format!("/corrections/{}/today.csv", &app.settings.cj_signature);
    let path = app.build_url(&path);
    let r = client.get(&path).send().await.expect("Failed to GET");
    assert_eq!(r.status(), 200, "Could not access {}.", path);
}
