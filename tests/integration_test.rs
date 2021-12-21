use cjms::handlers;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web::{self, Bytes}, App};

    #[actix_rt::test]
    async fn test_index_get() {
        let mut app = test::init_service(
            App::new()
                .route("/", web::get().to(handlers::index)),
        )
        .await;
        let req = test::TestRequest::get().uri("/").to_request();
        let result = test::read_response(&mut app, req).await;
        assert_eq!(result, Bytes::from_static(b"Hello world!"));
    }
}