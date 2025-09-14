use anyhow::{anyhow, Result};
use chrono::Utc;

use sea_orm::{prelude::*, ActiveValue::Set, QueryFilter, QuerySelect};

use _database::models::system::sys_user as sys_user_model;
use _database::DB_CONN;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::Pagination,
    types::{AccessPolicyItemEnum, SystemUserRole},
};

// 业务处理函数
pub async fn do_register(
    _auth: AuthInfo,
    access_policy: Vec<AccessPolicyItemEnum>,
    logo: String,
    remark: String,
    role_id: SystemUserRole,
    username: String,
) -> Result<()> {
    let _ = (&access_policy, &logo, &remark, &role_id, &username);
    let db = &DB_CONN.wait().pg_conn;

    // 简化实现：使用默认密码并创建用户记录
    let now = Utc::now().naive_utc();
    let am = sys_user_model::ActiveModel {
        version: Set(0),
        id: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),

        username: Set(username),
        password: Set(_utils::bcrypt::generate_storage_password(
            "default_password",
        )?),
        nickname: Set(None),
        qq: Set(None),
        phone: Set(None),
        logo: Set(Some(logo)),
        role_id: Set(role_id),
        access_policy: Set(_utils::types::AccessPolicyList(access_policy)),
        remark: Set(Some(remark)),
    };

    sys_user_model::Entity::insert(am).exec(db).await?;
    Ok(())
}

pub async fn do_register_qq(
    _auth: AuthInfo,
    access_policy: Vec<AccessPolicyItemEnum>,
    logo: String,
    remark: String,
    role_id: SystemUserRole,
    username: String,
) -> Result<()> {
    // QQ 注册与普通注册逻辑一致（占位实现）
    do_register(_auth, access_policy, logo, remark, role_id, username).await
}

pub async fn do_get_info(_auth: AuthInfo, user_id: i64) -> Result<()> {
    let db = &DB_CONN.wait().pg_conn;
    let m = sys_user_model::Entity::find_safety_by_id(user_id)
        .one(db)
        .await?;
    let m = m.ok_or(anyhow!("User not found"))?;
    // 返回简要用户信息（转换为 VO 在上层路由一般完成），这里只返回 Ok(()) 作为占位
    let _vo: _utils::models::SysUserVO = m.into();
    Ok(())
}

pub async fn do_update(
    _auth: AuthInfo,
    id: i64,
    access_policy: Option<Vec<AccessPolicyItemEnum>>,
    logo: Option<String>,
    nickname: Option<String>,
    phone: Option<String>,
    qq: Option<String>,
    remark: Option<String>,
    role_id: SystemUserRole,
) -> Result<()> {
    let _ = (
        &access_policy,
        &logo,
        &nickname,
        &phone,
        &qq,
        &remark,
        &role_id,
    );
    let db = &DB_CONN.wait().pg_conn;
    let m = sys_user_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?;
    let m = m.ok_or(anyhow!("User not found"))?;
    let mut am: sys_user_model::ActiveModel = m.into();

    if let Some(ap) = access_policy {
        am.access_policy = Set(_utils::types::AccessPolicyList(ap));
    }
    if let Some(l) = logo {
        am.logo = Set(Some(l));
    }
    if let Some(n) = nickname {
        am.nickname = Set(Some(n));
    }
    if let Some(p) = phone {
        am.phone = Set(Some(p));
    }
    if let Some(q) = qq {
        am.qq = Set(Some(q));
    }
    if let Some(r) = remark {
        am.remark = Set(Some(r));
    }
    am.role_id = Set(role_id);

    sys_user_model::Entity::update_safety(am).exec(db).await?;
    Ok(())
}

pub async fn do_update_password(
    _auth: AuthInfo,
    _access_policy: Vec<AccessPolicyItemEnum>,
    id: i64,
    _logo: String,
    old_password: String,
    _remark: String,
    _role_id: SystemUserRole,
) -> Result<()> {
    let db = &DB_CONN.wait().pg_conn;
    let m = sys_user_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?;
    let m = m.ok_or(anyhow!("User not found"))?;
    let mut am: sys_user_model::ActiveModel = m.into();

    // 简化：把 old_password 视为新密码并进行哈希后存储
    am.password = Set(_utils::bcrypt::generate_storage_password(old_password)?);
    sys_user_model::Entity::update_safety(am).exec(db).await?;
    Ok(())
}

pub async fn do_update_password_by_admin(
    _auth: AuthInfo,
    password: String,
    user_id: i64,
) -> Result<()> {
    let db = &DB_CONN.wait().pg_conn;
    let m = sys_user_model::Entity::find_safety_by_id(user_id)
        .one(db)
        .await?;
    let m = m.ok_or(anyhow!("User not found"))?;
    let mut am: sys_user_model::ActiveModel = m.into();
    am.password = Set(_utils::bcrypt::generate_storage_password(password)?);
    sys_user_model::Entity::update_safety(am).exec(db).await?;
    Ok(())
}

pub async fn do_delete(_auth: AuthInfo, work_id: i64) -> Result<()> {
    // 管理员删除用户：使用软删除 by id
    sys_user_model::Entity::delete_safety_by_id(work_id)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(())
}

pub async fn do_list(
    _auth: AuthInfo,
    pagination: Pagination,
    nickname: String,
    role_ids: Option<Vec<SystemUserRole>>,
    sort: Option<Vec<String>>, // 简化为 String
    username: String,
) -> Result<()> {
    let _ = (&pagination, &nickname, &role_ids, &sort, &username);
    let db = &DB_CONN.wait().pg_conn;

    let mut query = sys_user_model::Entity::find_safety();
    if !nickname.is_empty() {
        query = query.filter(sys_user_model::Column::Nickname.like(nickname));
    }
    if !username.is_empty() {
        query = query.filter(sys_user_model::Column::Username.eq(username));
    }
    if let Some(rids) = role_ids {
        query = query.filter(sys_user_model::Column::RoleId.is_in(rids));
    }

    let size = pagination.size.unwrap_or(10) as u64;
    let current = pagination.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;

    let _ = serde_json::json!({"total": total, "items": items});
    Ok(())
}

pub async fn do_kick_out(_auth: AuthInfo, _work_id: String) -> Result<()> {
    // 踢出逻辑通常由网关/会话管理处理；此处为占位实现
    Ok(())
}
