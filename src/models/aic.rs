use sqlx::{query_as, FromRow, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct AIC<'a> {
    pub id: Uuid,
    pub cj_event_value: &'a str,
    pub flow_id: &'a str,
    pub created: OffsetDateTime,
    pub expires: OffsetDateTime,
}

pub struct AICModel<'a> {
    pub db_pool: &'a PgPool,
}

impl AICModel<'_> {
    pub async fn create(&self) -> AIC<'_> {
        let id = Uuid::new_v4();
        let created = OffsetDateTime::now_utc();
        let expires = created.checked_add(Duration::days(30)).unwrap();
        let aic = AIC {
            id,
            cj_event_value: "cj_event_value",
            flow_id: "flow_id",
            created,
            expires,
        };
        query_as!(
            AIC,
            "INSERT INTO aic (id, cj_event_value, flow_id) VALUES ($1, $2, $3)",
            sqlx::types::Uuid::parse_str(&aic.id.to_string()).unwrap(),
            aic.cj_event_value,
            aic.flow_id
        )
        .execute(self.db_pool)
        .await
        .expect("errorrring ");
        aic
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    // AIC endpoing unit tests
    // sanitize input to post endpoint before put in db
    // expiration time based on environment variable
    #[test]
    fn go_here() {
        //assert!(1 == 0);
    }
}
