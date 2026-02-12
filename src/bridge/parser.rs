//! Kiro CLI Output Parser
//!
//! 解析 Kiro CLI 的输出，将其转换为 ACP 可以理解的格式

use anyhow::Result;

/// Kiro CLI 输出解析器
pub struct OutputParser {
    buffer: String,
}

impl OutputParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// 添加新的输出行
    pub fn add_line(&mut self, line: &str) {
        self.buffer.push_str(line);
    }

    /// 解析缓冲区中的内容
    pub fn parse(&self) -> Result<ParsedOutput> {
        // TODO: 根据 Kiro CLI 的实际输出格式实现解析逻辑
        // 这里先返回一个简单的实现
        Ok(ParsedOutput {
            content: self.buffer.clone(),
            is_complete: self.is_complete(),
        })
    }

    /// 检查响应是否完整
    pub fn is_complete(&self) -> bool {
        // TODO: 根据 Kiro CLI 的实际输出格式判断是否完整
        // 可能需要检查特定的结束标记
        self.buffer.ends_with("```\n")
            || self.buffer.contains("[END]")
            || self.buffer.ends_with("\n\n")
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for OutputParser {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析后的输出
#[derive(Debug, Clone)]
pub struct ParsedOutput {
    pub content: String,
    pub is_complete: bool,
}
