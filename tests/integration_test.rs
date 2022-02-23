#[cfg(test)]
mod tests {
    use actix_web::{test, web::Bytes, App};
    use chrono::{DateTime, Duration, Utc};
    use cjms::appconfig::config_app;
    use cjms::handlers::AICResponse;
    use serde_json::json;

    macro_rules! setup_app {
        () => {
            test::init_service(App::new().configure(config_app)).await
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
        let req = test::TestRequest::get()
            .uri("/__lbheartbeat__")
            .to_request();
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
        let mut app = setup_app!();
        let cj_event_value = "123ABC";
        let flow_id = "4jasdrkl";
        let data = json!({
            "flow_id": flow_id,
            "cj_id": cj_event_value,
        });
        let req = test::TestRequest::post()
            .set_json(&data)
            .uri("/aic")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 201);
        let aic: AICResponse = test::read_body_json(resp).await;
        println!("aic {:?}", aic);

        // Any AIC returned
        assert!(aic.aic_id.len() > 0); // TODO UUID - 32 hex + 4 dashes
                                       // Date is 30 days from today
        let today = Utc::now();
        let time_delta =
            today.signed_duration_since(DateTime::parse_from_rfc2822(&aic.expires).unwrap());
        assert_eq!(time_delta, Duration::days(30));
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
