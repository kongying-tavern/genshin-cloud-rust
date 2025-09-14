use anyhow::Result;

use bcrypt::{hash, verify as do_verify, DEFAULT_COST};

const BCRYPT_PREFIX: &str = "{bcrypt}";

/// 验证给定明文与存储值是否匹配。
/// 存储值可为原始 bcrypt hash，或以 `{bcrypt}` 前缀包装的形式（历史兼容）。
pub fn verify_password(input_raw: impl ToString, storage: impl ToString) -> Result<bool> {
    let storage = storage.to_string();
    let hash_str = if storage.starts_with(BCRYPT_PREFIX) {
        storage.trim_start_matches(BCRYPT_PREFIX)
    } else {
        storage.as_str()
    };
    Ok(do_verify(input_raw.to_string(), hash_str)?)
}

/// 生成不带前缀的 bcrypt hash（直接用于底层存储或进一步包装）
pub fn generate_hash(password_raw: impl ToString) -> Result<String> {
    Ok(hash(password_raw.to_string(), DEFAULT_COST)?.to_string())
}

/// 生成带 `{bcrypt}` 前缀的存储值，便于在数据库中区分加密方案
pub fn generate_storage_password(password_raw: impl ToString) -> Result<String> {
    let h = generate_hash(password_raw)?;
    Ok(format!("{}{}", BCRYPT_PREFIX, h))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_password_hash() {
        let map = [("user", "password_raw")];

        for (user, password_raw) in map.iter() {
            log::info!("{}: {}", user, generate_hash(password_raw).unwrap())
        }
    }
}
