use sqlx::{query, query_as, Error, PgPool};
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
impl PartialEq for AIC {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
        self.cj_event_value == other.cj_event_value &&
        self.flow_id == other.flow_id &&
        // When timestamps go in and out of database they lose precision to milliseconds
        self.created.millisecond() == other.created.millisecond() &&
        self.expires.millisecond() == other.expires.millisecond()
    }
}
impl Eq for AIC {}

pub struct AICModel<'a> {
    pub db_pool: &'a PgPool,
}

impl AICModel<'_> {
    pub async fn create_from_aic(&self, aic: &AIC) -> Result<AIC, Error> {
        query_as!(
            AIC,
            r#"INSERT INTO aic (id, cj_event_value, flow_id, created, expires)
			VALUES ($1, $2, $3, $4, $5)
			RETURNING *"#,
            aic.id,
            aic.cj_event_value,
            aic.flow_id,
            aic.created,
            aic.expires
        )
        .fetch_one(self.db_pool)
        .await
    }

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

    pub async fn fetch_one_by_id(&self, id: &Uuid) -> Result<AIC, Error> {
        query_as!(AIC, "SELECT * FROM aic WHERE id = $1", id)
            .fetch_one(self.db_pool)
            .await
    }

    pub async fn fetch_one_by_flow_id(&self, flow_id: &str) -> Result<AIC, Error> {
        query_as!(AIC, "SELECT * FROM aic WHERE flow_id = $1", flow_id)
            .fetch_one(self.db_pool)
            .await
    }

    pub async fn fetch_one_by_id_from_archive(&self, id: &Uuid) -> Result<AIC, Error> {
        query_as!(AIC, "SELECT * FROM aic_archive WHERE id = $1", id)
            .fetch_one(self.db_pool)
            .await
    }

    pub async fn fetch_one_by_flow_id_from_archive(&self, flow_id: &str) -> Result<AIC, Error> {
        query_as!(AIC, "SELECT * FROM aic_archive WHERE flow_id = $1", flow_id)
            .fetch_one(self.db_pool)
            .await
    }

    pub async fn create_archive_from_aic(&self, aic: &AIC) -> Result<AIC, Error> {
        query_as!(
            AIC,
            r#"INSERT INTO aic_archive (id, cj_event_value, flow_id, created, expires)
			VALUES ($1, $2, $3, $4, $5)
			RETURNING *"#,
            aic.id,
            aic.cj_event_value,
            aic.flow_id,
            aic.created,
            aic.expires
        )
        .fetch_one(self.db_pool)
        .await
    }

    async fn create_archive_delete_aic(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        aic: &AIC,
    ) -> Result<(), Box<dyn std::error::Error>> {
        query!("DELETE FROM aic WHERE id = $1", aic.id)
            .execute(&mut *transaction)
            .await?;
        query!(
            r#"INSERT INTO aic_archive (id, cj_event_value, flow_id, created, expires)
			VALUES ($1, $2, $3, $4, $5)
			RETURNING *"#,
            aic.id,
            aic.cj_event_value,
            aic.flow_id,
            aic.created,
            aic.expires
        )
        .fetch_one(&mut *transaction)
        .await?;
        Ok(())
    }

    pub async fn archive_aic(&self, aic: &AIC) -> Result<(), Box<dyn std::error::Error>> {
        // Wrap creating archive row and deleting aic row into one transaction
        let mut transaction = self.db_pool.begin().await?;
        self.create_archive_delete_aic(&mut transaction, aic)
            .await?;
        transaction.commit().await?;
        Ok(())
    }
}
