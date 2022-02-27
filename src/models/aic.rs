use sqlx::{query_as, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub struct AIC {
    pub id: Uuid,
    pub cj_event_value: String,
    pub flow_id: String,
    pub created: OffsetDateTime,
    pub expires: OffsetDateTime,
}

pub struct AICModel<'a> {
    pub db_pool: &'a PgPool,
}

impl AICModel<'_> {
    pub async fn create(&self) -> AIC {
        let id = Uuid::new_v4();
        let created = OffsetDateTime::now_utc();
        let expires = created + Duration::days(30);
        let created = query_as!(
            AIC,
            r#"INSERT INTO aic (id, cj_event_value, flow_id, created, expires)
			VALUES ($1, $2, $3, $4, $5)
			RETURNING *"#,
            id,
            "cj_event_value",
            "flow_id",
            created,
            expires
        )
        .fetch_one(self.db_pool)
        .await
        .expect("errorrring "); // TODO - Need to properly handle errors
        created
    }

    pub async fn fetch_one(&self) -> AIC {
        query_as!(AIC, "SELECT * FROM aic")
            .fetch_one(self.db_pool)
            .await
            .expect("errorrrriing ") // TODO - Need to handle properly
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
