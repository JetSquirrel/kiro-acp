//! ACP Protocol Definitions

mod messages;
mod session;

pub use messages::*;
pub use session::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// ACP 连接处理
pub struct AcpConnection {
    stdin: tokio::io::BufReader<tokio::io::Stdin>,
    stdout: tokio::io::Stdout,
}

impl AcpConnection {
    pub fn new_stdio() -> Self {
        Self {
            stdin: BufReader::new(tokio::io::stdin()),
            stdout: tokio::io::stdout(),
        }
    }

    /// 接收 JSON-RPC 消息
    pub async fn receive(&mut self) -> Result<Option<JsonRpcMessage>> {
        let mut line = String::new();

        match self.stdin.read_line(&mut line).await {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                let message = serde_json::from_str(&line)?;
                Ok(Some(message))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// 发送 JSON-RPC 消息
    pub async fn send(&mut self, message: JsonRpcMessage) -> Result<()> {
        let json = serde_json::to_string(&message)?;
        self.stdout.write_all(json.as_bytes()).await?;
        self.stdout.write_all(b"\n").await?;
        self.stdout.flush().await?;
        Ok(())
    }
}
