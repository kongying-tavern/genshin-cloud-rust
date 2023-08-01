pub mod area;
pub mod common;
pub mod icon;
pub mod item;
pub mod marker;
pub mod route;
pub mod score;
pub mod system;

use anyhow::Result;

pub async fn register() -> Result<()> {
    area::register().await?;
    common::register().await?;
    icon::register().await?;
    item::register().await?;
    marker::register().await?;
    route::register().await?;
    score::register().await?;
    system::register().await?;

    Ok(())
}
