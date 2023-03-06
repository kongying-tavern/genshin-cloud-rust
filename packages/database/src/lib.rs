pub mod models;

use log::info;
use std::time::Duration;

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, Schema, Statement};

pub struct DatabaseNetworkConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub async fn init(
    config: DatabaseNetworkConfig,
) -> Result<Box<DatabaseConnection>, Box<dyn std::error::Error>> {
    let mut opt = ConnectOptions::new(format!(
        "postgres://{}:{}@{}:{}/kongyin",
        config.username, config.password, config.host, config.port
    ));
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Trace);
    let db = Database::connect(opt).await?;

    let builder = db.get_database_backend();

    info!(
        "{:?}",
        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'genshin_map';"
                .to_owned(),
        ))
        .await?
    );

    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(models::sys_user::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(models::marker::Entity)
                .if_not_exists(),
        ),
    )
    .await?;

    info!("Database is ready");
    Ok(Box::new(db))
}
