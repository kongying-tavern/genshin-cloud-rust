use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait};

use _database::models::area::Entity as Area;

pub async fn list_area(db: Box<DatabaseConnection>) -> Result<String> {
    let mut res = String::new();
    for cc in Area::find().all(&*db).await? {
        res.push_str(&format!("{:?}\n", cc));
    }

    Ok(res)
}
