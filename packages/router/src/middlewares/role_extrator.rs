use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

use _utils::{jwt::AuthInfo, types::SystemUserRole};

use super::auth_extrator::ExtractAuthInfo;

/// Authorize the caller at (or above) `threshold`, mirroring the Java
/// gateway's cumulative `authorities-filter` + `SecurityFilter` sort
/// comparison: a role passes when its ordinal is **<=** the threshold's
/// (smaller ordinal = more privileged, `Admin = 0`).
async fn authorize<S: Send + Sync>(
    parts: &mut Parts,
    state: &S,
    threshold: SystemUserRole,
) -> Result<AuthInfo, Box<Response>> {
    let ExtractAuthInfo(auth) = ExtractAuthInfo::from_request_parts(parts, state).await?;
    if (auth.info.role_id as i32) > (threshold as i32) {
        return Err((StatusCode::FORBIDDEN, "Forbidden".to_string())
            .into_response()
            .into());
    }
    Ok(auth)
}

/// Map-punctuate-and-above extractor (`/api/marker/**`, `/api/marker_link/**`,
/// `/api/res/**` writes in the Java matrix).
pub struct ExtractPunctuate(pub AuthInfo);

impl<S> FromRequestParts<S> for ExtractPunctuate
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        authorize(parts, state, SystemUserRole::MapPunctuate)
            .await
            .map(Self)
            .map_err(|boxed| *boxed)
    }
}

/// Map-manager-and-above extractor (base-data writes: `area` / `icon` /
/// `icon_type` / `item` / `item_type` / `item_common` / `history` / `tag*`
/// in the Java matrix).
pub struct ExtractManager(pub AuthInfo);

impl<S> FromRequestParts<S> for ExtractManager
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        authorize(parts, state, SystemUserRole::MapManager)
            .await
            .map(Self)
            .map_err(|boxed| *boxed)
    }
}

#[cfg(test)]
mod tests {
    use _utils::types::SystemUserRole;

    /// The ordinal comparison is the entire authorization contract: smaller
    /// ordinal = more privileged, and equality passes (same-role access).
    #[test]
    fn role_ordinals_order_privileges_correctly() {
        use SystemUserRole::*;
        assert!((Admin as i32) < (MapManager as i32));
        assert!((MapManager as i32) < (MapBeta as i32));
        assert!((MapBeta as i32) < (MapPunctuate as i32));
        assert!((MapPunctuate as i32) < (MapUser as i32));
        assert!((MapUser as i32) < (Visitor as i32));

        // threshold check mirrors `(role as i32) > (threshold as i32)` rejection
        let threshold = SystemUserRole::MapManager;
        for role in [Admin, MapManager] {
            assert!(
                (role as i32) <= (threshold as i32),
                "{role:?} must pass Manager gate"
            );
        }
        for role in [MapBeta, MapPunctuate, MapUser, Visitor] {
            assert!(
                (role as i32) > (threshold as i32),
                "{role:?} must fail Manager gate"
            );
        }
    }
}
