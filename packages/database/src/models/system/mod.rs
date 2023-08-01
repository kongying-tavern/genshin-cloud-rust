pub mod oauth_client_details;
pub mod sys_role;
pub mod sys_user;
pub mod sys_user_archive;
pub mod sys_user_role_link;

use anyhow::Result;
use sea_orm::{ConnectionTrait, Schema};

use crate::DB_CONN;

pub async fn register() -> Result<()> {
    let db = DB_CONN.lock().await.get_mut().clone();
    let builder = db.get_database_backend();

    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(oauth_client_details::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(sys_role::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(sys_user::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(sys_user_archive::Entity)
                .if_not_exists(),
        ),
    )
    .await?;
    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(sys_user_role_link::Entity)
                .if_not_exists(),
        ),
    )
    .await?;

    Ok(())
}
