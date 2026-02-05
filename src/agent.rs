//! Kiro ACP Agent Implementation

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::bridge::KiroBridge;
use crate::protocol::{
    AcpConnection, InitializeRequest, InitializeResponse, NewSessionRequest,
    NewSessionResponse, ConversationTurnRequest, ConversationTurnResponse,
    JsonRpcMessage, JsonRpcRequest, JsonRpcResponse,
};

/// ACP Agent 状态
#[derive(Default)]
struct AgentState {
    initialized: bool,
    sessions: HashMap<String, SessionInfo>,
}

#[derive(Clone)]
struct SessionInfo {
    id: String,
    cwd: String,
}

/// Kiro ACP Agent
pub struct KiroAgent {
    bridge: Arc<Mutex<KiroBridge>>,
    state: Arc<Mutex<AgentState>>,
}

impl KiroAgent {
    pub fn new() -> Result<Self> {
        let bridge = KiroBridge::new()?;
        Ok(Self {
            bridge: Arc::new(Mutex::new(bridge)),
            state: Arc::new(Mutex::new(AgentState::default())),
        })
    }

    /// 运行 Agent 主循环
    pub async fn run(&self, mut connection: AcpConnection) -> Result<()> {
        loop {
            match connection.receive().await {
                Ok(Some(message)) => {
                    let response = self.handle_message(message).await;
                    if let Some(resp) = response {
                        connection.send(resp).await?;
                    }
                }
                Ok(None) => {
                    tracing::info!("Connection closed");
                    break;
                }
                Err(e) => {
                    tracing::error!("Error receiving message: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    async fn handle_message(&self, message: JsonRpcMessage) -> Option<JsonRpcMessage> {
        match message {
            JsonRpcMessage::Request(req) => {
                let response = self.handle_request(req).await;
                Some(JsonRpcMessage::Response(response))
            }
            JsonRpcMessage::Notification(notif) => {
                self.handle_notification(notif).await;
                None
            }
            _ => None,
        }
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "newSession" => self.handle_new_session(request.params).await,
            "conversationTurn" => self.handle_conversation_turn(request.params).await,
            "cancelTurn" => self.handle_cancel_turn(request.params).await,
            "destroy" => self.handle_destroy(request.params).await,
            _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse::success(id, value),
            Err(e) => JsonRpcResponse::error(id, -32603, e.to_string()),
        }
    }

    async fn handle_initialize(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let _request: InitializeRequest = params
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();

        let mut state = self.state.lock().await;
        state.initialized = true;

        let response = InitializeResponse {
            protocol_version: "2025-01-01".to_string(),
            name: "Kiro".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: AgentCapabilities {
                streaming: true,
                tools: true,
            },
        };

        Ok(serde_json::to_value(response)?)
    }

    async fn handle_new_session(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let request: NewSessionRequest = params
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();

        let session_id = uuid::Uuid::new_v4().to_string();
        let cwd = request.cwd.unwrap_or_else(|| ".".to_string());

        let mut state = self.state.lock().await;
        state.sessions.insert(session_id.clone(), SessionInfo {
            id: session_id.clone(),
            cwd: cwd.clone(),
        });

        // 初始化 Kiro Bridge
        let mut bridge = self.bridge.lock().await;
        bridge.start_session(&cwd).await?;

        let response = NewSessionResponse {
            session_id,
        };

        Ok(serde_json::to_value(response)?)
    }

    async fn handle_conversation_turn(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let request: ConversationTurnRequest = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing params"))?
        )?;

        let mut bridge = self.bridge.lock().await;
        let response = bridge.send_message(&request.session_id, &request.message).await?;

        Ok(serde_json::to_value(ConversationTurnResponse {
            content: response,
        })?)
    }

    async fn handle_cancel_turn(&self, _params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let mut bridge = self.bridge.lock().await;
        bridge.cancel().await?;
        Ok(serde_json::json!({}))
    }

    async fn handle_destroy(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        // 清理会话
        let mut bridge = self.bridge.lock().await;
        bridge.shutdown().await?;
        Ok(serde_json::json!({}))
    }

    async fn handle_notification(&self, _notif: crate::protocol::JsonRpcNotification) {
        // 处理通知（如 initialized）
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub streaming: bool,
    pub tools: bool,
}
