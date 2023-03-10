use sea_orm::DatabaseConnection;

pub mod functions;
pub mod schemas;

pub struct SharedDatabaseConnection {
    pub conn: Box<DatabaseConnection>,
    pub cache: _database::cache::CacheObject,
}
