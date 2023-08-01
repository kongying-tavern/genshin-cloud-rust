pub mod icon;
pub mod icon_type;
pub mod icon_type_link;
pub mod tag;
pub mod tag_type;
pub mod tag_type_link;

use anyhow::Result;
use sea_orm::{ConnectionTrait, Schema};

use crate::DB_CONN;

pub async fn register() -> Result<()> {
    let db = DB_CONN.lock().await.get_mut().clone();
    let builder = db.get_database_backend();

    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(icon::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(icon_type::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(icon_type_link::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(tag::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(tag_type::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(tag_type_link::Entity)
                .if_not_exists(),
        ),
    )
    .await?;

    Ok(())
}
