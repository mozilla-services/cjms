#[cfg(test)]
mod tests {
    use actix_web::{test, App, web::Bytes};
    use cjms::appconfig::config_app;

    macro_rules! setup_app {
    () => {
        test::init_service(
            App::new()
            .configure(config_app)
        )
        .await
    };
}

    #[actix_rt::test]
    async fn test_index_get() {
        let mut app = setup_app!();
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::read_response(&mut app, req).await;
        assert_eq!(resp, Bytes::from_static(b"Hello world!"));
    }

    #[actix_rt::test]
    async fn test_heartbeat_get() {
        let mut app = setup_app!();
        let req = test::TestRequest::get().uri("/__heartbeat__").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 200);
    }
}