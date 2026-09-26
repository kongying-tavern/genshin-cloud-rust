//! 领域错误类型：业务/输入/权限错误的显式分类，替代 router 层对错误
//! 文案的关键字子串猜测（旧行为：拼错误链小写串后匹配 db/sql/... 关键字，
//! 既会漏判内部错误（原文外泄），也会误伤含关键字的业务文案）。

use thiserror::Error;

/// 领域错误：由业务层显式构造，router 层按变体映射响应（见
/// routes::internal_error）。
#[derive(Debug, Error)]
pub enum DomainError {
    /// 业务错误：Java `RestException` 契约 —— HTTP 200 + R{errorStatus:500,
    /// message}，文案对客户端可见（前端 axios 只读 2xx 响应体）。
    #[error("{0}")]
    Business(String),
    /// 客户端输入错误：真实 400 + R 包装（如旧密码错误）。
    #[error("{0}")]
    BadRequest(String),
    /// 权限不足：真实 403 + R 包装。
    #[error("{0}")]
    Forbidden(String),
}
