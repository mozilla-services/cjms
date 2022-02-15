use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbError = Box<dyn std::error::Error + Send + Sync>;


pub fn create_database_pool(database_url: &String) -> DbPool {
    let connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(connection_manager)
        .expect(&format!("Failed to establish database connection to {}", database_url))
}
