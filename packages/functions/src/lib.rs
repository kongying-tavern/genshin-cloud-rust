use sea_orm::DatabaseConnection;

pub mod functions;

pub struct SharedDatabaseConnection {
    pub conn: Box<DatabaseConnection>,
    pub cache: _database::cache::CacheObject,
}
