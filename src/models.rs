use crate::schema::cj_events;

#[derive(Debug, Clone, Queryable)]
pub struct CjEvent {
    pub id: i32,
    pub flow_id: String,
    pub cj_id: String,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "cj_events"]
pub struct NewCjEvent {
    pub flow_id: String,
    pub cj_id: String,
}
