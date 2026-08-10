use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, crypto::rust_crypto::DEFAULT_PROVIDER,
    decode, encode,
};
use rsa::{
    pkcs1::DecodeRsaPrivateKey,
    pkcs8::{DecodePrivateKey, EncodePublicKey},
    traits::PublicKeyParts,
};

use crate::models::SysUserVO;

pub static JWT_SECRET: Lazy<(EncodingKey, DecodingKey)> = Lazy::new(|| {
    // jsonwebtoken v10 requires an explicit process-level CryptoProvider.
    // Install the rust_crypto (ring-based) provider before any JWT operation.
    let _ = DEFAULT_PROVIDER.install_default();

    let key = jwt_secret_raw();
    (
        EncodingKey::from_secret(key.as_bytes()),
        DecodingKey::from_secret(key.as_bytes()),
    )
});

/// The raw JWT secret (env `JWT_SECRET`, **required**). There is deliberately
/// no dev default: a predictable secret would let anyone forge tokens, and the
/// JWKS endpoint must never publish an HMAC signing key.
///
/// Generates a strong secret with, e.g.:
///   openssl rand -base64 48
pub fn jwt_secret_raw() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        panic!(
            "JWT_SECRET must be set (see .env.example); generate one with: openssl rand -base64 48"
        )
    })
}

/// The RSA private key PEM (env `JWT_RSA_PRIVATE_KEY_PEM`, optional).
/// When set, tokens are signed with RS256 and the JWKS endpoint publishes the
/// RSA public key. When unset, the workspace stays on HS256.
pub fn jwt_rsa_private_key_pem() -> Option<String> {
    std::env::var("JWT_RSA_PRIVATE_KEY_PEM").ok()
}

/// Historical RSA **public** key PEMs (env `JWT_RSA_VERIFY_KEYS`, optional).
/// **Comma**-separated list (each entry may itself be a multi-line PEM — the
/// newlines inside a PEM must NOT be treated as separators). Keys listed
/// here are accepted for verification (and published in the JWKS) but never
/// used for signing — that's what makes key rotation possible without
/// invalidating live tokens.
pub fn jwt_rsa_verify_keys() -> Vec<String> {
    let Some(v) = std::env::var("JWT_RSA_VERIFY_KEYS").ok() else {
        return Vec::new();
    };
    v.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Whether RS256 signing is active (an RSA private key is configured).
pub fn is_rsa_signing() -> bool {
    jwt_rsa_private_key_pem().is_some()
}

/// The active signing algorithm.
pub fn jwt_alg() -> Algorithm {
    if is_rsa_signing() {
        Algorithm::RS256
    } else {
        Algorithm::HS256
    }
}

/// The JWKS key id of the i-th RSA key: v1 is always the *current* signing
/// key (the private key's public half); the historical verify keys follow in
/// configuration order (v2, v3, ...). Stable as long as the config order is.
fn rsa_kid(index: usize) -> String {
    format!("genshin-cloud-rsa-v{}", index + 1)
}

/// Lazily parsed key material for both algorithms.
pub struct JwtKeys {
    /// Key used to sign new tokens (HS256 secret or RS256 private key).
    pub encoding: EncodingKey,
    /// RSA verification keys (current + historical, in kid order), present
    /// when RSA is configured.
    pub decoding_rsa: Vec<DecodingKey>,
    /// HS256 verification key (always available — legacy tokens stay valid).
    pub decoding_hmac: DecodingKey,
    /// RSA public keys for JWKS publication: `(kid, pem)`, current first.
    pub rsa_public_keys: Vec<(String, String)>,
}

static JWT_KEYS: Lazy<JwtKeys> = Lazy::new(|| {
    let _ = DEFAULT_PROVIDER.install_default();

    let hmac = jwt_secret_raw();
    let encoding_hmac = EncodingKey::from_secret(hmac.as_bytes());
    let decoding_hmac = DecodingKey::from_secret(hmac.as_bytes());

    let Some(pem) = jwt_rsa_private_key_pem() else {
        return JwtKeys {
            encoding: encoding_hmac,
            decoding_rsa: Vec::new(),
            decoding_hmac,
            rsa_public_keys: Vec::new(),
        };
    };

    match rsa::RsaPrivateKey::from_pkcs8_pem(&pem)
        .or_else(|_| rsa::RsaPrivateKey::from_pkcs1_pem(&pem))
    {
        Ok(private) => {
            let public = private.to_public_key();
            let public_pem = public.to_public_key_pem(rsa::pkcs8::LineEnding::LF).ok();
            let mut rsa_keys: Vec<(String, String)> = Vec::new(); // (kid, pem)
            if let Some(p) = public_pem {
                rsa_keys.push((rsa_kid(0), p));
            }
            // Historical verify keys follow the current one, in order.
            for (i, verify_pem) in jwt_rsa_verify_keys().iter().enumerate() {
                // Dedup: skip a verify key that equals the current public key.
                if rsa_keys.iter().any(|(_, p)| p == verify_pem) {
                    continue;
                }
                rsa_keys.push((rsa_kid(i + 1), verify_pem.clone()));
            }
            let decoding_rsa = rsa_keys
                .iter()
                .filter_map(|(_, p)| DecodingKey::from_rsa_pem(p.as_bytes()).ok())
                .collect();
            let encoding = match EncodingKey::from_rsa_pem(pem.as_bytes()) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("failed to parse RSA private key PEM: {e}; falling back to HS256");
                    encoding_hmac
                },
            };
            JwtKeys {
                encoding,
                decoding_rsa,
                decoding_hmac,
                rsa_public_keys: rsa_keys,
            }
        },
        Err(e) => {
            eprintln!("failed to parse RSA private key PEM: {e}; falling back to HS256");
            JwtKeys {
                encoding: encoding_hmac,
                decoding_rsa: Vec::new(),
                decoding_hmac,
                rsa_public_keys: Vec::new(),
            }
        },
    }
});

/// Sign a token with the active algorithm.
pub fn encoding_key() -> &'static EncodingKey {
    &JWT_KEYS.encoding
}

/// Verification keys with their matching algorithms. RSA keys come in kid
/// order (current first, then historical), the HMAC fallback last — tokens
/// signed before an RSA/HS256 migration or before a key rotation stay
/// verifiable.
pub fn decoding_key_pairs() -> Vec<(&'static DecodingKey, Algorithm)> {
    if is_rsa_signing() {
        let mut v = Vec::new();
        for k in &JWT_KEYS.decoding_rsa {
            v.push((k, Algorithm::RS256));
        }
        v.push((&JWT_KEYS.decoding_hmac, Algorithm::HS256));
        v
    } else {
        vec![(&JWT_KEYS.decoding_hmac, Algorithm::HS256)]
    }
}

/// Build the JWKS (JSON Web Key Set) for the active signing scheme.
///
/// RS256: publishes **all** RSA public keys (current `v1` first, then the
/// historical `JWT_RSA_VERIFY_KEYS` in order), each with its `kid` — so
/// verifiers can follow a rotation by kid. The key used for signing is always
/// `v1`.
/// HS256: publishes an **empty** key set — the HMAC signing key is a shared
/// secret and must never be disclosed (RFC 7517 §6.4 `oct` keys are for
/// verification-only consumers in trusted environments). Deployments that
/// need JWKS-based verification should configure `JWT_RSA_PRIVATE_KEY_PEM`
/// to switch to RS256.
pub fn jwks() -> anyhow::Result<serde_json::Value> {
    use base64::Engine;
    if JWT_KEYS.rsa_public_keys.is_empty() {
        return Ok(serde_json::json!({ "keys": [] }));
    }
    let mut keys = Vec::new();
    for (kid, pem) in &JWT_KEYS.rsa_public_keys {
        let public = <rsa::RsaPublicKey as rsa::pkcs8::DecodePublicKey>::from_public_key_pem(pem)?;
        let n = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public.n().to_bytes_be());
        let e = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public.e().to_bytes_be());
        keys.push(serde_json::json!({
            "kty": "RSA",
            "kid": kid,
            "alg": "RS256",
            "use": "sig",
            "n": n,
            "e": e,
        }));
    }
    Ok(serde_json::json!({ "keys": keys }))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthInfo {
    pub info: SysUserVO,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl AuthInfo {
    /// Whether this token belongs to the anonymous client-credentials identity
    /// (user id 0). Anonymous tokens are for read-only browsing: write
    /// operations must call `require_non_anonymous`.
    pub fn is_anonymous(&self) -> bool {
        self.info.id == 0
    }

    /// Reject anonymous (client-credentials) tokens on write operations.
    pub fn require_non_anonymous(&self) -> anyhow::Result<()> {
        if self.is_anonymous() {
            anyhow::bail!("Anonymous token is not allowed for this operation")
        }
        Ok(())
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub jti: Uuid,
    #[serde(with = "jwt_numeric_date")]
    pub iat: DateTime<Utc>,
    #[serde(with = "jwt_numeric_date")]
    pub exp: DateTime<Utc>,
    /// 令牌用途：`access` | `refresh`。旧版本签发的令牌无此声明
    /// （`None`），按原有 Redis key 校验路径兼容处理。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
}

pub static EXPIRED_APPEND_DURATION: chrono::Duration = chrono::Duration::days(15);

pub async fn generate_token(
    now: DateTime<Utc>,
    user_id: i64,
    jti: Uuid,
    token_type: &str,
) -> Result<String> {
    let claims = Claims {
        sub: user_id,
        jti,
        iat: now,
        exp: now + EXPIRED_APPEND_DURATION,
        token_type: Some(token_type.to_string()),
    };

    encode(&Header::new(jwt_alg()), &claims, encoding_key()).context("Failed to encode token")
}

pub async fn verify_token(token: &str) -> Result<Claims> {
    // Try the active algorithm first, then the fallback — so tokens signed
    // before an RSA/HS256 migration stay verifiable.
    let mut last_err = None;
    for (key, alg) in decoding_key_pairs() {
        match decode::<Claims>(token, key, &Validation::new(alg)) {
            Ok(token_data) => return Ok(token_data.claims),
            Err(e) => last_err = Some(e),
        }
    }
    Err(anyhow::anyhow!(
        "Invalid token: {}",
        last_err.map(|e| e.to_string()).unwrap_or_default()
    ))
}
