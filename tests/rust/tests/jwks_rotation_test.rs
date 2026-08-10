//! JWKS key-rotation test (PLAN.md M4 gap): after rotating the RSA signing
//! key, tokens signed with the **old** key must still verify (the old public
//! key stays in the verification set), the new key signs, and the JWKS
//! publishes both keys with stable kids (current = v1, historical = v2+).
//!
//! No DB needed. This binary is its own process, and the test sets the env
//! vars before any JWT operation touches the lazy key material (same pattern
//! as `api_db_test`).

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::pkcs8::{DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::traits::PublicKeyParts;

fn rsa_pair() -> (String, String) {
    let key = rsa::RsaPrivateKey::new(&mut rand_core::OsRng, 2048).expect("generate RSA key");
    (
        key.to_pkcs8_pem(LineEnding::LF)
            .expect("encode private key")
            .to_string(),
        key.to_public_key()
            .to_public_key_pem(LineEnding::LF)
            .expect("encode public key")
            .to_string(),
    )
}

fn sign_with(now: chrono::DateTime<chrono::Utc>, private_pem: &str) -> String {
    let claims = _utils::jwt::Claims {
        sub: 42,
        jti: uuid::Uuid::new_v4(),
        iss: "genshin-cloud".to_string(),
        aud: "genshin-map".to_string(),
        iat: now,
        exp: now + chrono::Duration::days(1),
        token_type: None,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(private_pem.as_bytes()).expect("encoding key"),
    )
    .expect("sign token")
}

#[tokio::test]
async fn rotated_keys_stay_verifiable_and_published() {
    let now = chrono::Utc::now();
    let (old_priv, old_pub) = rsa_pair();
    let (new_priv, new_pub) = rsa_pair();

    // SAFETY: single test binary; env vars are set once before any JWT
    // operation reads the lazy key material (edition 2024 marks set_var
    // unsafe because concurrent readers could race).
    unsafe {
        std::env::set_var("JWT_SECRET", "rotation-test-secret");
        std::env::set_var("JWT_RSA_PRIVATE_KEY_PEM", &new_priv);
        std::env::set_var("JWT_RSA_VERIFY_KEYS", &old_pub);
    }

    // 1) A token signed with the OLD (rotated-out) key still verifies.
    let old_token = sign_with(now, &old_priv);
    let claims = _utils::jwt::verify_token(&old_token)
        .await
        .expect("old-key token must verify after rotation");
    assert_eq!(claims.sub, 42);

    // 2) A token signed with the CURRENT key verifies too.
    let new_token = sign_with(now, &new_priv);
    let claims = _utils::jwt::verify_token(&new_token)
        .await
        .expect("new-key token must verify");
    assert_eq!(claims.sub, 42);

    // 3) The JWKS publishes both keys, current (v1) first, historical
    //    (v2) second — kids are stable and the n's differ.
    let jwks = _utils::jwt::jwks().expect("jwks");
    let keys = jwks["keys"].as_array().expect("keys array");
    assert_eq!(keys.len(), 2, "JWKS must publish current + historical keys");
    assert_eq!(keys[0]["kid"], "genshin-cloud-rsa-v1");
    assert_eq!(keys[1]["kid"], "genshin-cloud-rsa-v2");

    use base64::Engine;
    let modulus = |jwk: &serde_json::Value| -> rsa::BigUint {
        rsa::BigUint::from_bytes_be(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(jwk["n"].as_str().expect("n"))
                .expect("valid base64url n"),
        )
    };
    assert_ne!(
        modulus(&keys[0]),
        modulus(&keys[1]),
        "rotated keys must differ"
    );

    // 4) The published v1 matches the new private key's public half.
    let expected_new_n = rsa::RsaPublicKey::from_public_key_pem(&new_pub)
        .expect("parse new public pem")
        .n()
        .clone();
    assert_eq!(
        modulus(&keys[0]),
        expected_new_n,
        "v1 = current signing key"
    );
    let expected_old_n = rsa::RsaPublicKey::from_public_key_pem(&old_pub)
        .expect("parse old public pem")
        .n()
        .clone();
    assert_eq!(modulus(&keys[1]), expected_old_n, "v2 = historical key");
}
