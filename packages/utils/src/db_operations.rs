pub trait SafeEntityTrait: ::sea_orm::EntityTrait {
    type ActiveModel: ::sea_orm::ActiveModelTrait + Send + Sync;

    fn find_safety() -> ::sea_orm::Select<Self>
    where
        Self: Sized;

    fn find_safety_by_id(id: i64) -> ::sea_orm::Select<Self>
    where
        Self: Sized;

    /// sea-orm 2.0 returns a `ValidatedUpdateOne` (the primary-key check is
    /// eager), so the safety helpers return `Result<ValidatedUpdateOne, _>`.
    /// Callers must `?` the result before `.exec().await`.
    ///
    /// **`del_flag` is deliberately `NotSet` in the generated UPDATE** (see the
    /// macro impl): callers routinely build a full-row ActiveModel from a DB
    /// model, and writing that row back must never resurrect a soft-deleted
    /// record as a side effect. Soft delete and restore therefore have their
    /// own helpers — never encode `del_flag` into the model handed to
    /// `update_safety` and expect it to take effect.
    fn update_safety(
        model: <Self as SafeEntityTrait>::ActiveModel,
    ) -> std::result::Result<
        ::sea_orm::ValidatedUpdateOne<<Self as SafeEntityTrait>::ActiveModel>,
        ::sea_orm::DbErr,
    >
    where
        Self: Sized;

    /// Execute an optimistic-lock update immediately and fail when it hits
    /// zero rows.
    ///
    /// `update_safety` returns `UpdateResult` and most production call sites
    /// ignored `rows_affected`: losing the optimistic-lock race produced a
    /// silent no-op (HTTP 200, data unchanged, client believes it saved).
    /// This variant is the same statement, but a zero-row result becomes the
    /// business-level conflict error the Java clients already understand.
    ///
    /// Only for updates where the row is expected to exist (update handlers).
    /// Deletes-by-id stay tolerant: zero rows means "already gone", which is
    /// the idempotent contract.
    // 静态分发的内部 trait：调用点直接 .await，从不命名 Future 类型；Send
    // 由 axum handler 的调用上下文自身校验。rustc 的 async_fn_in_trait 提示
    // 针对的是「公开 trait 无法声明 Future 的 auto-trait 约束」，此处不适用。
    #[allow(async_fn_in_trait)]
    async fn update_safety_strict<C>(
        model: <Self as SafeEntityTrait>::ActiveModel,
        db: &C,
    ) -> ::anyhow::Result<::sea_orm::UpdateResult>
    where
        C: ::sea_orm::ConnectionTrait + Send + Sync,
        Self: Sized;

    fn delete_safety(
        model: <Self as SafeEntityTrait>::ActiveModel,
    ) -> std::result::Result<
        ::sea_orm::ValidatedUpdateOne<<Self as SafeEntityTrait>::ActiveModel>,
        ::sea_orm::DbErr,
    >
    where
        Self: Sized;

    /// Flip `del_flag` back to `false` (the inverse of `delete_safety`).
    ///
    /// No version filter: like `delete_safety`, this is a `del_flag`
    /// transition, not a business-field edit, so it must succeed on the
    /// current row state (a soft-deleted row may have been updated by the
    /// delete itself).
    fn restore_safety(
        model: <Self as SafeEntityTrait>::ActiveModel,
    ) -> std::result::Result<
        ::sea_orm::ValidatedUpdateOne<<Self as SafeEntityTrait>::ActiveModel>,
        ::sea_orm::DbErr,
    >
    where
        Self: Sized;

    fn delete_safety_by_id(
        id: i64,
    ) -> std::result::Result<
        ::sea_orm::ValidatedUpdateOne<<Self as SafeEntityTrait>::ActiveModel>,
        ::sea_orm::DbErr,
    >
    where
        Self: Sized;
}

#[macro_export]
macro_rules! impl_safe_operation {
    {
        active_model_ty: $active_model_ty:ty,
        updated_at_column_name: $updated_at_column_name:ident,
        updated_at_column_init_expr: $updated_at_column_init_expr:expr,
        del_flag_column: $del_flag_column:expr
    } => {
        #[::async_trait::async_trait]
        impl ::sea_orm::ActiveModelBehavior for ActiveModel {
            async fn before_save<C>(
                self,
                _conn: &C,
                is_insert: bool,
            ) -> Result<Self, ::sea_orm::DbErr>
            where
                C: ::sea_orm::ConnectionTrait,
            {
                if !is_insert {
                    return Ok(Self {
                        $updated_at_column_name: ::sea_orm::ActiveValue::Set(Some(
                            $updated_at_column_init_expr,
                        )),
                        ..self
                    });
                }

                Ok(self)
            }

            async fn before_delete<C>(self, _conn: &C) -> Result<Self, ::sea_orm::DbErr>
            where
                C: ::sea_orm::ConnectionTrait,
            {
                Err(::sea_orm::DbErr::Query(::sea_orm::RuntimeErr::Internal(
                    "Hard delete is not supported".into(),
                )))
            }
        }

        impl ::_utils::db_operations::SafeEntityTrait for Entity {
            type ActiveModel = $active_model_ty;

            fn find_safety() -> ::sea_orm::Select<Self> {
                Self::find().filter($del_flag_column.eq(false))
            }

            fn find_safety_by_id(id: i64) -> ::sea_orm::Select<Self> {
                Self::find_safety().filter(Column::Id.eq(id))
            }

            fn update_safety(
                model: ActiveModel,
            ) -> std::result::Result<::sea_orm::ValidatedUpdateOne<ActiveModel>, ::sea_orm::DbErr> {
                let last_version = model
                    .version
                    .into_value()
                    .and_then(|v| match v {
                        ::sea_orm::Value::TinyInt(Some(val)) => Some(val as i64),
                        ::sea_orm::Value::SmallInt(Some(val)) => Some(val as i64),
                        ::sea_orm::Value::Int(Some(val)) => Some(val as i64),
                        ::sea_orm::Value::BigInt(Some(val)) => Some(val as i64),
                        ::sea_orm::Value::TinyUnsigned(Some(val)) => Some(val as i64),
                        ::sea_orm::Value::SmallUnsigned(Some(val)) => Some(val as i64),
                        ::sea_orm::Value::Unsigned(Some(val)) => Some(val as i64),
                        ::sea_orm::Value::BigUnsigned(Some(val)) => Some(val as i64),
                        _ => None,
                    })
                    .unwrap_or(1);
                // sea-orm 2.0: UpdateOne wraps Result<ValidatedUpdateOne>; call
                // .validate() then .map(.filter()) to stay in Result without ?.
                Self::update(ActiveModel {
                    version: ::sea_orm::ActiveValue::Set(last_version + 1),
                    // 调用方多由 `m.into()` 构造全字段 ActiveModel（含 del_flag=false），
                    // 若整行写回会覆盖 del_flag，把已软删的行“复活”；置 NotSet 让
                    // del_flag 不参与 UPDATE，更新只作用于业务字段。
                    del_flag: ::sea_orm::ActiveValue::NotSet,
                    ..model
                })
                .validate()
                .map(|v| v.filter(Column::Version.eq(last_version)))
            }

            async fn update_safety_strict<C>(
                model: ActiveModel,
                db: &C,
            ) -> ::anyhow::Result<::sea_orm::UpdateResult>
            where
                C: ::sea_orm::ConnectionTrait + Send + Sync,
            {
                // sea-orm 2.0 的 ValidatedUpdateOne::exec 返回更新后的 Model，
                // 拿不到行数；exec_without_returning 才暴露 rows_affected，且在
                // 0 行时给出 RecordNotUpdated。
                match Self::update_safety(model)?
                    .exec_without_returning(db)
                    .await
                {
                    Ok(res) => Ok(res),
                    // 乐观锁冲突（version 已变）或行已不存在：调用方原本
                    // `.exec()?; Ok(true)` 静默成功，这里按 Java 业务错误契约
                    // 返回（HTTP 200 + errorStatus 500 + 可读文案）。
                    Err(::sea_orm::DbErr::RecordNotUpdated) => {
                        Err(::_utils::errors::DomainError::Business(
                            "数据已更新，请刷新后重试".into(),
                        )
                        .into())
                    },
                    Err(err) => Err(err.into()),
                }
            }

            fn delete_safety(
                model: ActiveModel,
            ) -> std::result::Result<::sea_orm::ValidatedUpdateOne<ActiveModel>, ::sea_orm::DbErr> {
                Self::update(ActiveModel {
                    del_flag: ::sea_orm::ActiveValue::Set(true),
                    ..model
                })
                .validate()
            }

            fn restore_safety(
                model: ActiveModel,
            ) -> std::result::Result<::sea_orm::ValidatedUpdateOne<ActiveModel>, ::sea_orm::DbErr> {
                // 与 delete_safety 对称：显式 Set 覆盖调用方 ActiveModel 里的
                // del_flag 值；无 version filter（del_flag 迁移不是业务字段编辑）。
                Self::update(ActiveModel {
                    del_flag: ::sea_orm::ActiveValue::Set(false),
                    ..model
                })
                .validate()
            }

            fn delete_safety_by_id(
                id: i64,
            ) -> std::result::Result<::sea_orm::ValidatedUpdateOne<ActiveModel>, ::sea_orm::DbErr> {
                Self::delete_safety(ActiveModel {
                    id: ::sea_orm::ActiveValue::Set(id),
                    ..Default::default()
                })
            }
        }
    };
}
