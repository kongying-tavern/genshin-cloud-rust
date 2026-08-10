//! User invitation business logic — mirrors Java `SysUserInvitationService`.

use std::collections::HashSet;

use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryFilter, QuerySelect, TransactionTrait,
    prelude::*,
};

use _database::{
    DB_CONN,
    models::system::{sys_user as sys_user_model, sys_user_invitation as inv_model},
};
use _utils::{
    bcrypt,
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{SysUserInvitationVo, wrapper::CommonResponse},
    types::{AccessPolicyList, InvitationSort, SystemUserRole},
};

/// List invitations with optional filtering by code / username.
pub async fn do_list(
    _auth: AuthInfo,
    code: Option<String>,
    username: Option<String>,
    sort: Option<Vec<InvitationSort>>,
    size: u64,
    current: u64,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = inv_model::Entity::find_safety();

    if let Some(c) = code {
        query = query.filter(inv_model::Column::Code.eq(c));
    }
    if let Some(u) = username {
        query = query.filter(inv_model::Column::Username.eq(u));
    }

    // 排序：显式枚举映射，变体重命名会变成编译错误而非静默忽略排序键。
    if let Some(sorts) = sort {
        use sea_orm::QueryOrder;
        for s in sorts {
            let (column, desc) = match s {
                InvitationSort::CreateTime => (inv_model::Column::CreateTime, false),
                InvitationSort::CreateTimeReverse => (inv_model::Column::CreateTime, true),
                InvitationSort::Id => (inv_model::Column::Id, false),
                InvitationSort::IdReverse => (inv_model::Column::Id, true),
                InvitationSort::UpdateTime => (inv_model::Column::UpdateTime, false),
                InvitationSort::UpdateTimeReverse => (inv_model::Column::UpdateTime, true),
                InvitationSort::Username => (inv_model::Column::Username, false),
                InvitationSort::UsernameReverse => (inv_model::Column::Username, true),
            };
            query = if desc {
                query.order_by(column, sea_orm::Order::Desc)
            } else {
                query.order_by(column, sea_orm::Order::Asc)
            };
        }
    }

    let total = query.clone().count(db).await?;
    let offset = current.saturating_sub(1).saturating_mul(size);
    let items = query.limit(size).offset(offset).all(db).await?;
    let creator_ids: HashSet<i64> = items.iter().filter_map(|inv| inv.creator_id).collect();
    let record: Vec<SysUserInvitationVo> = items
        .into_iter()
        .map(|inv| SysUserInvitationVo {
            id: inv.id,
            create_time: inv.create_time.and_utc().timestamp_millis() as f64,
            update_time: inv
                .update_time
                .map(|t| t.and_utc().timestamp_millis() as f64),
            creator_id: inv.creator_id,
            code: inv.code,
            username: inv.username,
            role_id: inv.role_id.map(|r| r as i64),
            remark: inv.remark,
            access_policy: inv.access_policy,
        })
        .collect();

    let users = crate::functions::api::sys_user_map(db, &creator_ids).await?;
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": total,
        "record": record,
    })))
    .with_users(users))
}

/// Update or create an invitation by code (upsert semantics).
/// `code` 为空时生成新邀请码并插入；`code` 有值时按 code 更新，查不到则插入。
pub async fn do_update(
    auth: AuthInfo,
    code: Option<String>,
    username: String,
    role_id: i64,
    remark: String,
    access_policy: Vec<_utils::types::AccessPolicyItemEnum>,
) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;

    let role = match role_id {
        0 => _utils::types::SystemUserRole::Admin,
        1 => _utils::types::SystemUserRole::MapManager,
        2 => _utils::types::SystemUserRole::MapNeigui,
        3 => _utils::types::SystemUserRole::MapPunctuate,
        4 => _utils::types::SystemUserRole::MapUser,
        5 => _utils::types::SystemUserRole::Visitor,
        _ => return Err(anyhow!("Invalid role id")),
    };
    // 安全边界：邀请码不可授予 Admin（管理员仅能由 Admin 直接注册/提拔）。
    // 显式拒绝而非静默降级，防止低权限操作者通过邀请码扩散管理员权限。
    if role == SystemUserRole::Admin {
        return Err(anyhow!("Admin role cannot be granted via invitation code"));
    }
    let access_policy = serde_json::to_value(AccessPolicyList(access_policy))?;

    if let Some(c) = code {
        if let Some(inv) = inv_model::Entity::find_safety()
            .filter(inv_model::Column::Code.eq(&c))
            .one(db)
            .await?
        {
            let mut am: inv_model::ActiveModel = inv.into();
            am.username = Set(username);
            am.role_id = Set(Some(role));
            am.remark = Set(Some(remark));
            am.access_policy = Set(Some(access_policy));
            am.updater_id = Set(Some(auth.info.id));
            inv_model::Entity::update_safety(am)?.exec(db).await?;
            return Ok(CommonResponse::new(Ok(())));
        }

        let now = Utc::now().naive_utc();
        let am = inv_model::ActiveModel {
            version: Set(0),
            id: NotSet,
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(Some(auth.info.id)),
            updater_id: Set(None),
            del_flag: Set(false),
            code: Set(c),
            username: Set(username),
            role_id: Set(Some(role)),
            remark: Set(Some(remark)),
            access_policy: Set(Some(access_policy)),
        };
        inv_model::Entity::insert(am).exec(db).await?;
        return Ok(CommonResponse::new(Ok(())));
    }

    // 邀请码：uuid 前 12 位 hex（48 bit 熵，显著高于原 8 位，降低撞码/爆破风险）
    let code = &uuid::Uuid::new_v4().simple().to_string()[..12];
    let now = Utc::now().naive_utc();
    let am = inv_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(None),
        del_flag: Set(false),
        code: Set(code.to_string()),
        username: Set(username),
        role_id: Set(Some(role)),
        remark: Set(Some(remark)),
        access_policy: Set(Some(access_policy)),
    };
    inv_model::Entity::insert(am).exec(db).await?;
    Ok(CommonResponse::new(Ok(())))
}

/// Check invitation info by code.
pub async fn do_info(auth: AuthInfo, code: String) -> Result<CommonResponse<serde_json::Value>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let inv = inv_model::Entity::find_safety()
        .filter(inv_model::Column::Code.eq(code))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Invitation code not found"))?;
    Ok(CommonResponse::new(Ok(serde_json::to_value(inv)?)))
}

/// Consume (use) an invitation code — creates the invited user with the
/// invitation's role, then deletes the invitation code.
/// 返回 `{userId, result}`，对齐前端 `SysUserInvitationConsumeResultVo`。
#[allow(clippy::too_many_arguments)]
pub async fn do_consume(
    code: String,
    username: Option<String>,
    password: Option<String>,
    nickname: Option<String>,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;

    let now = Utc::now().naive_utc();
    let username = username
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| code.clone());
    // 前端注册流程必带密码；缺省时生成随机密码（邀请人可再通过管理员改密）
    let password = password
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| format!("{}_{}", code, now.and_utc().timestamp()));

    // 事务内完成「查邀请 → 抢占消费 → 建用户」，防并发重复消费
    let txn = db.begin().await?;

    let inv = inv_model::Entity::find_safety()
        .filter(inv_model::Column::Code.eq(&code))
        .one(&txn)
        .await?
        .ok_or_else(|| anyhow!("Invitation code not found"))?;

    // 条件软删抢占邀请码：并发消费时，后到的事务会等锁并在拿到行锁后重新
    // 评估 WHERE（del_flag=false 已不再满足）→ 影响行数为 0，说明已被消费，
    // 回滚并报错，杜绝同一邀请码被重复使用。
    let claimed = inv_model::Entity::update_many()
        .col_expr(
            inv_model::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .col_expr(
            inv_model::Column::UpdateTime,
            sea_orm::sea_query::Expr::value(now),
        )
        .filter(inv_model::Column::Id.eq(inv.id))
        .filter(inv_model::Column::DelFlag.eq(false))
        .exec(&txn)
        .await?
        .rows_affected;
    if claimed == 0 {
        txn.rollback().await?;
        return Err(anyhow!("Invitation code already consumed"));
    }

    // 用户已存在：不重复创建，返回 EXISTING 供前端直接走登录（回滚抢占，
    // 保留邀请码不被消耗）
    if let Some(user) = sys_user_model::Entity::find_safety()
        .filter(sys_user_model::Column::Username.eq(&username))
        .one(&txn)
        .await?
    {
        txn.rollback().await?;
        return Ok(CommonResponse::new(Ok(serde_json::json!({
            "userId": user.id,
            "result": "EXISTING",
        }))));
    }

    let user_am = sys_user_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),

        username: Set(username),
        password: Set(bcrypt::generate_storage_password(&password)?),
        nickname: Set(nickname),
        qq: Set(None),
        phone: Set(None),
        logo: Set(None),
        // 防御：历史遗留的 Admin 邀请码（新逻辑已禁止创建）消费时降级为 MapUser
        role_id: Set(inv
            .role_id
            .filter(|r| *r != SystemUserRole::Admin)
            .unwrap_or(SystemUserRole::MapUser)),
        access_policy: Set(inv
            .access_policy
            .as_ref()
            .and_then(|v| serde_json::from_value::<AccessPolicyList>(v.clone()).ok())),
        remark: Set(Some(String::new())),
    };
    let res = sys_user_model::Entity::insert(user_am).exec(&txn).await?;

    // 提交事务：用户落库 + 邀请码软删一起生效；任一步失败则整体回滚
    txn.commit().await?;
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "userId": res.last_insert_id,
        "result": "SUCCESS",
    }))))
}

/// Delete an invitation by id (soft delete).
pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;
    let inv = inv_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Invitation not found"))?;
    inv_model::Entity::delete_safety(inv.into())?
        .exec(db)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}
