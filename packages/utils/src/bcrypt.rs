use anyhow::Result;

use bcrypt::{hash, verify as do_verify, DEFAULT_COST};

pub fn verify_hash(input_raw: impl ToString, storage_hash: impl ToString) -> Result<bool> {
    Ok(do_verify(
        input_raw.to_string(),
        storage_hash.to_string().as_str(),
    )?)
}

pub fn generate_hash(password_raw: impl ToString) -> Result<String> {
    Ok(hash(password_raw.to_string(), DEFAULT_COST)?.to_string())
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
