use serde_json::Value as JsonValue;
use sqlx::{query_as, Error, PgPool};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use super::status_history::{Status, UpdateStatus};

pub struct PartialRefund {
    pub id: Uuid,
    pub refund_id: String,
    pub subscription_id: String,
    pub refund_created: OffsetDateTime,
    pub refund_amount: i32,
    pub refund_status: Option<String>,
    pub refund_reason: Option<String>,
    pub correction_file_date: Option<Date>,
}
#[derive(Debug)]
pub struct Refund {
    pub id: Uuid,
    pub refund_id: String,
    pub subscription_id: String,
    pub refund_created: OffsetDateTime,
    pub refund_amount: i32,
    pub refund_status: Option<String>,
    pub refund_reason: Option<String>,
    pub correction_file_date: Option<Date>,
    // Note we use string and json to save in database for simplicity
    status: Option<String>,
    status_t: Option<OffsetDateTime>,
    status_history: Option<JsonValue>,
}

impl PartialEq for Refund {
    fn eq(&self, other: &Self) -> bool {
        let simple_match = self.id == other.id &&
        self.refund_id == other.refund_id &&
        self.subscription_id == other.subscription_id &&
        self.refund_created.unix_timestamp() == other.refund_created.unix_timestamp() &&
        self.refund_amount == other.refund_amount &&
        self.refund_status == other.refund_status &&
        self.refund_reason == other.refund_reason &&
        self.correction_file_date == other.correction_file_date &&
        self.status == other.status
        // Compare manually if needed
        // self.status_history == other.status_history
        ;
        let status_t_match = match self.status_t {
            Some(self_v) => match other.status_t {
                Some(other_v) => self_v.unix_timestamp() == other_v.unix_timestamp(),
                None => false,
            },
            None => other.status_t.is_none(),
        };
        status_t_match && simple_match
    }
}
impl Eq for Refund {}

impl UpdateStatus for Refund {
    fn get_raw_status(&self) -> Option<String> {
        self.status.clone()
    }

    fn get_raw_status_history(&self) -> Option<JsonValue> {
        self.status_history.clone()
    }

    fn set_raw_status(&mut self, v: Option<String>) {
        self.status = v;
    }

    fn set_raw_status_history(&mut self, v: Option<JsonValue>) {
        self.status_history = v;
    }

    fn get_status_t(&self) -> Option<OffsetDateTime> {
        self.status_t
    }

    fn set_status_t(&mut self, v: Option<OffsetDateTime>) {
        self.status_t = v;
    }
}

impl Refund {
    pub fn new(partial: PartialRefund) -> Self {
        let mut r = Refund {
            id: partial.id,
            refund_id: partial.refund_id,
            subscription_id: partial.subscription_id,
            refund_created: partial.refund_created,
            refund_amount: partial.refund_amount,
            refund_status: partial.refund_status,
            refund_reason: partial.refund_reason,
            correction_file_date: partial.correction_file_date,
            status: None,
            status_t: None,
            status_history: None,
        };
        r.update_status(Status::NotReported);
        r
    }
}

pub struct RefundModel<'a> {
    pub db_pool: &'a PgPool,
}

impl RefundModel<'_> {
    pub async fn create_from_refund(&self, refund: &Refund) -> Result<Refund, Error> {
        query_as!(
            Refund,
            "INSERT INTO refunds (id, refund_id, subscription_id, refund_created, refund_amount, refund_status, refund_reason, correction_file_date, status, status_t, status_history)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
			RETURNING *",
            refund.id,
            refund.refund_id,
            refund.subscription_id,
            refund.refund_created,
            refund.refund_amount,
            refund.refund_status,
            refund.refund_reason,
            refund.correction_file_date,
            refund.status,
            refund.status_t,
            refund.status_history,
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn fetch_one_by_refund_id(&self, refund_id: &str) -> Result<Refund, Error> {
        query_as!(
            Refund,
            "SELECT * FROM refunds WHERE refund_id = $1",
            refund_id
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn update_refund(&self, r: &Refund) -> Result<Refund, Error> {
        query_as!(
            Refund,
            "UPDATE refunds
            SET
                subscription_id = $1,
                refund_created = $2,
                refund_amount = $3,
                refund_status = $4,
                refund_reason = $5,
                correction_file_date = $6,
                status = $7,
                status_t = $8,
                status_history = $9
            WHERE refund_id = $10
			RETURNING *",
            r.subscription_id,
            r.refund_created,
            r.refund_amount,
            r.refund_status,
            r.refund_reason,
            r.correction_file_date,
            r.status,
            r.status_t,
            r.status_history,
            r.refund_id
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn fetch_all(&self) -> Result<Vec<Refund>, Error> {
        query_as!(Refund, "SELECT * FROM refunds")
            .fetch_all(self.db_pool)
            .await
    }

    pub async fn fetch_not_reported(&self) -> Result<Vec<Refund>, Error> {
        query_as!(Refund, "SELECT * FROM refunds WHERE status = 'NotReported'")
            .fetch_all(self.db_pool)
            .await
    }

    pub async fn fetch_by_correction_file_day(&self, day: &Date) -> Result<Vec<Refund>, Error> {
        query_as!(
            Refund,
            "SELECT * FROM refunds WHERE correction_file_date = $1",
            day
        )
        .fetch_all(self.db_pool)
        .await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::random_simple_ascii_string;

    #[test]
    fn test_new_sets_not_reported_status_and_history() {
        let new = Refund::new(PartialRefund {
            id: Uuid::new_v4(),
            refund_id: random_simple_ascii_string(),
            subscription_id: random_simple_ascii_string(),
            refund_created: OffsetDateTime::now_utc(),
            refund_amount: 1,
            refund_status: None,
            refund_reason: None,
            correction_file_date: None,
        });
        let now = OffsetDateTime::now_utc();
        assert_eq!(new.get_status().unwrap(), Status::NotReported);
        assert_eq!(
            new.get_status_t().unwrap().unix_timestamp(),
            now.unix_timestamp()
        );
        let status_history = new.get_status_history().unwrap();
        assert_eq!(status_history.entries.len(), 1);
        assert_eq!(status_history.entries[0].status, Status::NotReported);
        assert_eq!(
            status_history.entries[0].t.unix_timestamp(),
            now.unix_timestamp()
        );
    }
}
