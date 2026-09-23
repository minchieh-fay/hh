use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

use crate::{agent, llm};

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<llm::ChatMessage>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamEvent {
    pub task_id: String,
    pub agent_id: String,
    pub kind: String,
    pub content: Option<String>,
    pub detail: Option<String>,
}

/// 启动聊天 Agent 异步任务，并向前端发送生命周期事件。
#[tauri::command]
pub async fn start_chat_task(
    app: AppHandle,
    request: ChatRequest,
) -> Result<String, String> {
    // step.1 校验聊天上下文并通知前端任务开始
    help_validate_request(&request)?;
    let task_id = help_create_task_id();
    help_emit_event(&app, help_build_event(
        &task_id,
        "started",
        None,
        Some("正在准备聊天 Agent"),
    ))?;

    // step.2 将聊天请求放入后台并发送处理中状态
    let task_app = app.clone();
    let task_id_for_worker = task_id.clone();
    tauri::async_runtime::spawn(async move {
        let result = async {
            crate::config::get_config(&task_app).await?;
            help_emit_event(&task_app, help_build_event(
                &task_id_for_worker,
                "thinking",
                None,
                Some("聊天 Agent 正在思考"),
            ))?;
            agent::chat::run(request.messages).await
        }
        .await;

        // step.3 将聊天结果转换为前端消息或错误事件
        let event = match result {
            Ok(response) => help_build_event(
                &task_id_for_worker,
                "completed",
                help_extract_response(&response),
                None,
            ),
            Err(error) => help_build_event(
                &task_id_for_worker,
                "error",
                Some(error),
                None,
            ),
        };
        let _ = help_emit_event(&task_app, event);
    });

    Ok(task_id)
}

/// 结束聊天 Agent 的前端等待状态。
#[tauri::command]
pub fn cancel_chat_task(task_id: String) -> Result<(), String> {
    if task_id.trim().is_empty() {
        return Err("任务标识不能为空".to_string());
    }
    Ok(())
}

/// 校验聊天请求必须包含用户消息。
fn help_validate_request(request: &ChatRequest) -> Result<(), String> {
    if request.messages.is_empty() {
        return Err("聊天消息不能为空".to_string());
    }
    if request.messages.iter().any(|message| message.role == "system") {
        return Err("前端不能提交 system 消息".to_string());
    }
    Ok(())
}

/// 生成聊天任务的唯一标识。
fn help_create_task_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("chat-task-{timestamp}")
}

/// 创建前端任务栏可以消费的结构化事件。
fn help_build_event(
    task_id: &str,
    kind: &str,
    content: Option<String>,
    detail: Option<&str>,
) -> AgentStreamEvent {
    AgentStreamEvent {
        task_id: task_id.to_string(),
        agent_id: "chat".to_string(),
        kind: kind.to_string(),
        content,
        detail: detail.map(str::to_string),
    }
}

/// 将 Agent 事件广播到当前应用窗口。
fn help_emit_event(app: &AppHandle, event: AgentStreamEvent) -> Result<(), String> {
    app.emit("agent://stream", event)
        .map_err(|error| format!("发送 Agent 事件失败: {error}"))
}

/// 从聊天响应中提取首条助手消息。
fn help_extract_response(response: &llm::ChatCompletionResponse) -> Option<String> {
    response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
}
