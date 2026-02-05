你是一个 Rust 工程师，你将会为 Kiro 开发 ACP 客户端。
由于 Kiro-cli 并未提供开源的接口，所以我们将会以子程序的形式开发。

你可以查阅kiro官方文档
- https://kiro.dev/docs/cli/

以下为参考方案
---
🏗️ 自建 Kiro ACP 适配器指南
架构概览
参考 claude-code-acp 的实现，ACP 适配器的核心架构如下：

Code
┌─────────────┐     JSON-RPC/stdin/stdout     ┌─────────────────┐
│  Zed Editor │ ◄──────────────────────────► │  ACP Adapter    │
│  (ACP客户端) │                               │  (kiro-acp)     │
└─────────────┘                               └────────┬────────┘
                                                       │
                                                       ▼
                                              ┌─────────────────┐
                                              │   Kiro CLI      │
                                              │   (子进程/API)   │
                                              └─────────────────┘
核心依赖
package.json
{
  "name": "@your-org/kiro-acp",
  "version": "0.1.0",
  "description": "ACP adapter for Kiro CLI",
  "type": "module",
  "bin": {
核心文件结构
Code
kiro-acp/
├── src/
│   ├── index.ts          # 入口点
│   ├── kiro-agent.ts     # ACP Agent 实现
│   ├── kiro-bridge.ts    # 与 Kiro CLI 通信的桥接层
│   └── utils.ts          # 工具函数
├── package.json
└── tsconfig.json
入口文件实现
src/index.ts
#!/usr/bin/env node

// stdout 用于 ACP 通信，重定向日志到 stderr
console.log = console.error;
console.info = console.error;
console.warn = console.error;
ACP Agent 核心实现
参考 claude-code-acp 的 acp-agent.ts，你需要实现 Agent 接口：

src/kiro-agent.ts
import {
  Agent,
  AgentSideConnection,
  InitializeRequest,
  InitializeResponse,
  NewSessionRequest,
Kiro CLI 桥接层
这是最关键的部分，需要根据 Kiro CLI 的实际接口来实现：

src/kiro-bridge.ts
import { spawn, ChildProcess } from "node:child_process";

interface BridgeOptions {
  onMessage: (content: string) => Promise<void>;
  onToolCall: (toolCall: { id: string; title: string }) => Promise<void>;
}
工具函数
src/utils.ts
import { Readable, Writable } from "node:stream";

export function nodeToWebReadable(stream: NodeJS.ReadableStream): ReadableStream<Uint8Array> {
  return Readable.toWeb(stream as Readable) as ReadableStream<Uint8Array>;
}

在 Zed 中配置
安装后，在 Zed 的 settings.json 中添加：

settings.json
{
  "agent": {
    "external_agents": [
      {
        "name": "Kiro",
        "command": "kiro-acp",
⚠️ 主要挑战
由于 Kiro CLI 是闭源的，你需要解决以下问题：

挑战	可能的解决方案
没有官方 SDK	使用子进程封装 CLI 命令
没有流式输出协议	解析 CLI 的 stdout，可能需要逆向工程
没有 API 文档	尝试 kiro --help，或使用网络抓包分析
认证机制不明	可能需要复用 Kiro 的登录状态 (~/.kiro/)
🔍 建议的探索步骤
探索 Kiro CLI 能力：

bash
kiro --help
kiro --version
# 查看是否有非交互模式、API 模式、JSON 输出等
查找配置文件：

bash
ls -la ~/.kiro/
ls -la ~/.config/kiro/
监控网络请求（如果 Kiro 使用 HTTP API）：

bash
# macOS
sudo tcpdump -i any port 443 -w kiro.pcap
关注官方动态：

GitHub Issue #4825 - ACP 支持请求
📚 参考资源
ACP 协议规范
Zed 外部代理文档
claude-code-acp 源码 - 最佳参考实现
@agentclientprotocol/sdk NPM

- https://github.com/zed-industries/codex/tree/acp
