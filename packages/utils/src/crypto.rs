//! Crypto provider initialization.
//!
//! jsonwebtoken v10 requires an explicit process-level CryptoProvider.
//! The actual installation (`rustls::crypto::ring::default_provider().
//! install_default()`) happens in `_router::main` because `_utils`
//! does not depend on `rustls` directly. This module exists to document
//! the requirement and provide a placeholder.

/// Placeholder — actual crypto provider installation is in `_router::main`.
pub fn install_default_provider() {
    // No-op. See _router/src/main.rs for the actual installation.
}
