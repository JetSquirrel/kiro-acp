//! Kiro ACP Adapter - Entry Point
//!
//! 此程序作为 Zed Editor 的外部 Agent，通过 stdin/stdout 进行 ACP 协议通信

use std::io::{self, BufRead, Write};
use tracing_subscriber::EnvFilter;

mod agent;
mod bridge;
mod protocol;
mod utils;

use agent::KiroAgent;
use protocol::AcpConnection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志到 stderr（stdout 用于 ACP 通信）
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(io::stderr)
        .init();

    tracing::info!("Starting Kiro ACP adapter");

    // 创建 ACP 连接（stdin/stdout）
    let connection = AcpConnection::new_stdio();

    // 创建 Kiro Agent
    let agent = KiroAgent::new()?;

    // 运行主事件循环
    agent.run(connection).await?;

    Ok(())
}
