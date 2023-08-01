pub mod score_stat;

use anyhow::Result;
use sea_orm::{ConnectionTrait, Schema};

use crate::DB_CONN;

pub async fn register() -> Result<()> {
    let db = DB_CONN.lock().await.get_mut().clone();
    let builder = db.get_database_backend();

    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(score_stat::Entity)
                .if_not_exists(),
        ),
    )
    .await?;

    Ok(())
}
