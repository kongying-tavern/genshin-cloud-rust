//! Library surface of the router crate.
//!
//! The route table (`routes::router()`), the middlewares, and the `functions`
//! re-export live here so the integration test crate (`tests/rust`) can drive
//! the assembled `Router` directly via `tower::ServiceExt::oneshot` — no port,
//! no DB. Process-level wiring (logging, shutdown signals, the listener)
//! stays in `main.rs`; that binary consumes this crate like any other
//! downstream crate.

pub mod functions;
pub mod middlewares;
pub mod routes;
