use super::help::{help_check_response, help_validate_image_input};
use serde::{Deserialize, Serialize};

pub(crate) mod config;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub tools: Option<serde_json::Value>,
    pub tool_choice: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionResponse {
    pub id: Option<String>,
    pub model: Option<String>,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatResponseMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<serde_json::Value>,
}

/// 调用 Agnes Chat Completions，并支持文本和 base64 图片消息。
pub async fn chat_completion(
    current_config: &config::Config,
    request: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, String> {
    // step.1 使用聊天模块配置并校验消息和图片输入
    if request.messages.is_empty() {
        return Err("至少需要一条消息".to_string());
    }
    if request.stream.unwrap_or(false) {
        return Err("当前聊天适配器暂不支持流式响应".to_string());
    }
    for message in &request.messages {
        if let MessageContent::Parts(parts) = &message.content {
            for part in parts {
                if let ContentPart::ImageUrl { image_url } = part {
                    if !current_config.supports_image {
                        return Err("当前聊天模型不支持图片输入".to_string());
                    }
                    help_validate_image_input(&image_url.url)?;
                }
            }
        }
    }
    // step.2 使用聊天模块配置组装兼容请求
    let api_key = current_config
        .api_key
        .clone()
        .ok_or_else(|| "尚未配置 Agnes API key".to_string())?;
    let body = serde_json::json!({
        "model": current_config.model,
        "messages": request.messages,
        "temperature": request.temperature,
        "top_p": request.top_p,
        "max_tokens": request.max_tokens,
        "stream": request.stream.unwrap_or(false),
        "tools": request.tools,
        "tool_choice": request.tool_choice,
    });
    // step.3 请求远程模型并解析结构化回答
    let response = reqwest::Client::new()
        .post(format!(
            "{}{}",
            current_config.base_url, current_config.endpoint
        ))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("调用文本模型失败：{error}"))?;
    help_check_response(response)
        .await?
        .json::<ChatCompletionResponse>()
        .await
        .map_err(|error| format!("解析文本模型响应失败：{error}"))
}
