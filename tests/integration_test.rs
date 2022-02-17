#[cfg(test)]
mod tests {
    use actix_web::{test, App, web::Bytes};
    use chrono::{DateTime, Utc};
    use cjms::appconfig::config_app;
    use cjms::handlers::{AICResponse};
    use serde_json::json;
    use uuid::{Uuid, Version};


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
        let body = test::read_response(&mut app, req).await;
        assert_eq!(body, Bytes::from_static(b"Hello world!"));
    }

    #[actix_rt::test]
    async fn test_heartbeat_get() {
        let mut app = setup_app!();
        let req = test::TestRequest::get().uri("/__heartbeat__").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_rt::test]
    async fn test_lbheartbeat_get() {
        let mut app = setup_app!();
        let req = test::TestRequest::get().uri("/__lbheartbeat__").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 200);
    }

    /*
     * START /aic endpoint (Affiliate Identifier Cookie)
     *
     */

    #[actix_rt::test]
    async fn test_aic_get_is_not_allowed() {
        let mut app = setup_app!();
        let req = test::TestRequest::get().uri("/aic").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 405);
        let req = test::TestRequest::get().uri("/aic/123").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 405);
    }

    #[actix_rt::test]
    async fn test_aic_endpoint_when_no_aic_sent() {
        /* Bedrock sends flowId and CJEvent value and not an AIC value
           - create a new AIC id
           - save creation time, expiration time, AIC id, flow ID, CJ event value
           - return expiration time, and AIC id
        */

        /* SETUP */
        let mut app = setup_app!();
        let cj_event_value = "123ABC";
        let flow_id = "4jasdrkl";
        let data = json!({
            "flow_id": flow_id,
            "cj_id": cj_event_value,
        });
        /* CALL */
        let req = test::TestRequest::post().set_json(&data).uri("/aic").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 201);
        let resp:AICResponse = test::read_body_json(resp).await;

        /* CHECK RESPONSE */
        // Should be UUID v4 aka Version::Random
        let returned_uuid = Uuid::parse_str(&resp.aic_id).unwrap();
        assert_eq!(Some(Version::Random), returned_uuid.get_version());
        // Expires date is 30 days from today
        // (because we created the expires a few nano seconds a go, this is a minute under 30 days)
        let expires = DateTime::parse_from_rfc2822(&resp.expires).unwrap();
        assert_eq!(expires.signed_duration_since(Utc::now()).num_minutes(), 30 * 24 * 60 - 1);

        /* CHECK DATABASE */
        assert!(false);
    }

    #[actix_rt::test]
    async fn test_something_happens_when_wrong_data_is_sent() {
        assert_eq!(true, false);
    }

    #[actix_rt::test]
    async fn test_aic_endpoint_when_no_aic_exists() {
        /* Bedrock sends flowId, CJEvent value, and AIC value but AIC doesn't exist in our DB
           - create a new AIC id
           - save creation time, expiration time, AIC id, flow ID, CJ event value
           - return expiration time, and AIC id
        */
        let mut app = setup_app!();
        let req = test::TestRequest::put().uri("/aic/123").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 201);
        assert_eq!(true, false);
    }

    #[actix_rt::test]
    async fn test_aic_endpoint_when_aic_exists() {
        /* Bedrock sends AIC id, flowId, new CJEvent value
           - keep existing AIC id
           - save new creation time, new expiration time, new flow ID, new CJ event value
           - return new expiration time, existing AIC id
        */
        let mut app = setup_app!();
        let req = test::TestRequest::put().uri("/aic/123").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 200);
        assert_eq!(true, false);
    }

    #[actix_rt::test]
    async fn test_aic_endpoint_when_aic_and_cjevent_exists() {
        /* Bedrock sends AIC id, flowId, existing CJEvent value
           - keep existing AIC id, creation time, expiration time, cjevent value
           - save new flow ID
           - return existing expiration time, existing AIC id
        */
        let mut app = setup_app!();
        let req = test::TestRequest::put().uri("/aic/123").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 200);
        assert_eq!(true, false);
    }

    // END /aic endpoint
}