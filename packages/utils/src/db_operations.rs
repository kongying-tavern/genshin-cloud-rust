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
    fn update_safety(
        model: <Self as SafeEntityTrait>::ActiveModel,
    ) -> std::result::Result<
        ::sea_orm::ValidatedUpdateOne<<Self as SafeEntityTrait>::ActiveModel>,
        ::sea_orm::DbErr,
    >
    where
        Self: Sized;

    fn delete_safety(
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
                    ..model
                })
                .validate()
                .map(|v| v.filter(Column::Version.eq(last_version)))
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
