//! Kiro CLI Process Management

use anyhow::Result;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

pub struct KiroProcess {
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout_rx: mpsc::Receiver<String>,
}

impl KiroProcess {
    /// 启动 Kiro CLI 子进程
    pub async fn spawn(kiro_path: &Path, working_dir: &Path) -> Result<Self> {
        let mut child = Command::new(kiro_path)
            .current_dir(working_dir)
            // TODO: 根据 Kiro CLI 实际支持的参数调整
            // .arg("--non-interactive")
            // .arg("--json-output")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdin"))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

        // 创建 stdout 读取通道
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        if tx.send(line.clone()).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error reading stdout: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            stdout_rx: rx,
        })
    }

    /// 向 Kiro CLI 发送输入
    pub async fn send_input(&mut self, input: &str) -> Result<()> {
        self.stdin.write_all(input.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// 读取 Kiro CLI 响应
    pub async fn read_response(&mut self) -> Result<String> {
        let mut response = String::new();

        // TODO: 需要根据 Kiro CLI 的实际输出格式来解析
        // 这里简单地读取直到遇到特定结束标记或超时
        while let Some(line) = self.stdout_rx.recv().await {
            response.push_str(&line);

            // 检查是否是响应结束（需要根据实际情况调整）
            if self.is_response_complete(&response) {
                break;
            }
        }

        Ok(response)
    }

    /// 发送中断信号
    pub async fn send_interrupt(&mut self) -> Result<()> {
        // 发送 Ctrl+C
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            if let Some(id) = self.child.id() {
                let _ = kill(Pid::from_raw(id as i32), Signal::SIGINT);
            }
        }
        Ok(())
    }

    /// 终止子进程
    pub async fn terminate(&mut self) -> Result<()> {
        self.child.kill().await?;
        Ok(())
    }

    fn is_response_complete(&self, response: &str) -> bool {
        // TODO: 根据 Kiro CLI 的实际输出格式判断响应是否完成
        // 可能需要检查特定的 JSON 结构或结束标记
        response.ends_with("```\n") || response.contains("[END]")
    }
}
