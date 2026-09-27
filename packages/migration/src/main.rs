//! `_migration` CLI —— 手写的极简迁移入口。
//!
//! **为何不用 sea-orm-migration 自带的 `cli` feature**：它捆绑 clap +
//! dotenvy + sea-orm-cli（整套实体代码生成工具链），换来的 `init`/
//! `generate` 子命令对本仓库毫无用处——迁移文件手工添加即可（见 lib.rs
//! 顶部「演进纪律」）。这里只映射运维真正需要的子命令，其余参数一律报错
//! 拒绝而不是静默忽略。
//!
//! 连接复用 `_database::connect_standalone_pg`：与 init_db / 主服务同一套
//! `DB_*` env 组装、同一套 `search_path` 绑定与 `CREATE SCHEMA IF NOT
//! EXISTS` 前置。
//!
//! 日志走 env_logger（本仓库标准初始化器）：sea-orm-migration 内部日志用
//! tracing 宏输出，但其 tracing 依赖开启了 `log` 特性桥接到 log 门面，
//! 因此这里的 env_logger 能原样显示迁移进度与 status 结果。默认 info 级，
//! `RUST_LOG` 可覆盖。
//!
//! 用法（workspace 根）：
//!   cargo run -p _migration                     # status：列出各迁移状态
//!   cargo run -p _migration -- up               # 应用全部待处理迁移
//!   cargo run -p _migration -- down [N]         # 回滚最近 N 个迁移（默认 1）
//!   cargo run -p _migration -- fresh            # 清空当前 schema 后全部重放
//!   cargo run -p _migration -- refresh          # 全部回滚再全部重放
//!
//! `fresh` 会删除当前 schema（`DB_SCHEMA`，默认 genshin_map）内的全部表
//! 与类型——仅面向 dev 环境。

use anyhow::{Context, Result, bail};

// up/down/status 等都是 MigratorTrait 的关联函数，trait 必须在作用域内。
use _migration::{Migrator, MigratorTrait};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let db = _database::connect_standalone_pg()
        .await
        .context("connect for migration CLI")?;

    let cmd = args.first().map(String::as_str);
    match cmd {
        // 无参数默认 status：最常用的只读检查，作为缺省行为不会破坏任何
        // 东西；其余破坏性操作（down/fresh/refresh）必须显式敲出来。
        None | Some("status") => Migrator::status(&db).await?,
        Some("up") => Migrator::up(&db, None).await?,
        Some("down") => {
            let steps = args
                .get(1)
                .map(|s| s.parse::<u32>())
                .transpose()
                .map_err(|e| anyhow::anyhow!("回滚步数必须是正整数：{e}"))?;
            Migrator::down(&db, steps).await?
        },
        Some("fresh") => Migrator::fresh(&db).await?,
        Some("refresh") => Migrator::refresh(&db).await?,
        other => bail!(
            "未知子命令 {other:?}：用法 _migration [status|up|down [N]|fresh|refresh]\
             （详见 src/main.rs 顶部注释）"
        ),
    }
    Ok(())
}
