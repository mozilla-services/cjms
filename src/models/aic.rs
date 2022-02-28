use sqlx::{query_as, Error, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug)]
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
    pub async fn create(&self, cj_event_value: &str, flow_id: &str) -> Result<AIC, Error> {
        let id = Uuid::new_v4();
        let created = OffsetDateTime::now_utc();
        let expires = created + Duration::days(30);
        query_as!(
            AIC,
            r#"INSERT INTO aic (id, cj_event_value, flow_id, created, expires)
			VALUES ($1, $2, $3, $4, $5)
			RETURNING *"#,
            id,
            cj_event_value,
            flow_id,
            created,
            expires
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn update_flow_id(&self, id: Uuid, flow_id: &str) -> Result<AIC, Error> {
        // A new flow_id alone, does not reset the clock on the cookie
        query_as!(
            AIC,
            r#"UPDATE aic
            SET flow_id = $1
            WHERE id = $2
			RETURNING *"#,
            flow_id,
            id,
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn update_flow_id_and_cj_event_value(
        &self,
        id: Uuid,
        cj_event_value: &str,
        flow_id: &str,
    ) -> Result<AIC, Error> {
        // A new cj_event_value resets the clock on the cookie
        let created = OffsetDateTime::now_utc();
        let expires = created + Duration::days(30);
        query_as!(
            AIC,
            r#"UPDATE aic
            SET
                cj_event_value = $1,
                flow_id = $2,
                created = $3,
                expires = $4
            WHERE id = $5
			RETURNING *"#,
            cj_event_value,
            flow_id,
            created,
            expires,
            id,
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn fetch_one(&self) -> Result<AIC, Error> {
        query_as!(AIC, "SELECT * FROM aic")
            .fetch_one(self.db_pool)
            .await
    }

    pub async fn fetch_one_by_id(&self, id: Uuid) -> Result<AIC, Error> {
        query_as!(AIC, "SELECT * FROM aic WHERE id = $1", id)
            .fetch_one(self.db_pool)
            .await
    }
}
