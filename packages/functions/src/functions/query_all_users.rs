use sea_orm::{DatabaseConnection, EntityTrait};

use _database::models::sys_user::Entity as User;

pub async fn query_all_users(
    db: Box<DatabaseConnection>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut res = String::new();
    for cc in User::find().all(&*db).await? {
        res.push_str(&format!("{:?}\n", cc));
    }
    Ok(res)
}
