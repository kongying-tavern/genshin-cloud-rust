use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub const JWT_SECRET: Lazy<(EncodingKey, DecodingKey)> = Lazy::new(|| {
    let key = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "下定决心，不怕牺牲，排除万难，去争取胜利".into());
    (
        EncodingKey::from_secret(key.as_bytes()),
        DecodingKey::from_secret(key.as_bytes()),
    )
});

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthInfo {
    pub token: String,
    pub user_id: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

mod jwt_numeric_date {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.timestamp();
        serializer.serialize_i64(timestamp)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Utc.timestamp_opt(i64::deserialize(deserializer)?, 0)
            .single()
            .ok_or_else(|| serde::de::Error::custom("invalid Unix timestamp value"))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u64,
    pub jti: Uuid,
    #[serde(with = "jwt_numeric_date")]
    pub iat: DateTime<Utc>,
    #[serde(with = "jwt_numeric_date")]
    pub exp: DateTime<Utc>,
}

pub static EXPIRED_APPEND_DURATION: chrono::Duration = chrono::Duration::days(15);

pub async fn generate_token(now: DateTime<Utc>, user_id: u64, jti: Uuid) -> Result<String> {
    let claims = Claims {
        sub: user_id,
        jti,
        iat: now,
        exp: now + EXPIRED_APPEND_DURATION,
    };

    Ok(encode(&Header::default(), &claims, &JWT_SECRET.0).context("Failed to encode token")?)
}

pub async fn verify_token(token: &str) -> Result<Claims> {
    let token_data =
        decode::<Claims>(token, &JWT_SECRET.1, &Validation::default()).context("Invalid token")?;
    Ok(token_data.claims)
}
