//! Kiro CLI Bridge - 与 Kiro CLI 子进程通信

mod process;
mod parser;

pub use process::KiroProcess;
pub use parser::OutputParser;

use anyhow::Result;
use std::path::PathBuf;
use tokio::process::Child;

/// Kiro CLI 桥接器
pub struct KiroBridge {
    process: Option<KiroProcess>,
    working_dir: PathBuf,
}

impl KiroBridge {
    pub fn new() -> Result<Self> {
        Ok(Self {
            process: None,
            working_dir: PathBuf::from("."),
        })
    }

    /// 启动 Kiro CLI 会话
    pub async fn start_session(&mut self, cwd: &str) -> Result<()> {
        self.working_dir = PathBuf::from(cwd);

        // 尝试查找 Kiro CLI 可执行文件
        let kiro_path = self.find_kiro_executable()?;

        tracing::info!("Starting Kiro CLI from: {:?}", kiro_path);

        let process = KiroProcess::spawn(&kiro_path, &self.working_dir).await?;
        self.process = Some(process);

        Ok(())
    }

    /// 向 Kiro 发送消息并获取响应
    pub async fn send_message(&mut self, _session_id: &str, message: &str) -> Result<String> {
        let process = self.process.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Kiro process not started"))?;

        process.send_input(message).await?;
        let response = process.read_response().await?;

        Ok(response)
    }

    /// 取消当前操作
    pub async fn cancel(&mut self) -> Result<()> {
        if let Some(process) = &mut self.process {
            process.send_interrupt().await?;
        }
        Ok(())
    }

    /// 关闭会话
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.terminate().await?;
        }
        Ok(())
    }

    /// 查找 Kiro CLI 可执行文件
    fn find_kiro_executable(&self) -> Result<PathBuf> {
        // 1. 检查环境变量
        if let Ok(path) = std::env::var("KIRO_PATH") {
            return Ok(PathBuf::from(path));
        }

        // 2. 检查 PATH 中的 kiro 命令
        if let Ok(path) = which::which("kiro") {
            return Ok(path);
        }

        // 3. 检查常见安装位置
        let home = dirs::home_dir().unwrap_or_default();
        let candidates = [
            home.join(".kiro/bin/kiro"),
            home.join(".local/bin/kiro"),
            PathBuf::from("/usr/local/bin/kiro"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        Err(anyhow::anyhow!(
            "Kiro CLI not found. Please install Kiro or set KIRO_PATH environment variable."
        ))
    }
}
