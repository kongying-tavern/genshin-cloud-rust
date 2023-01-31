#![allow(unused_variables)] //允许未使用的变量
//允许未使用的代码
#![allow(dead_code)]
#![allow(unused_must_use)]

use state::Container;

pub mod utils;

/// 全局上下文
/// 内含：
/// 1. 全局配置
/// 2. Rbatis连接池实例
pub static APPLICATION_CONTEXT: Container![Send + Sync] = <Container![Send + Sync]>::new();

pub async fn init_context() {
    
    
}


