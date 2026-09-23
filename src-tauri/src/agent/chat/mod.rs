use crate::llm::{self, ChatCompletionRequest, ChatCompletionResponse};

mod config;

/// 将完整会话历史交给聊天模型并返回其结构化回答。
pub async fn run(messages: Vec<llm::ChatMessage>) -> Result<ChatCompletionResponse, String> {
    let current_config = config::load()?;
    llm::chat_completion(&current_config, help_build_request(messages)).await
}

/// 将聊天 Agent 的消息转换为固定的非流式模型请求。
fn help_build_request(messages: Vec<llm::ChatMessage>) -> ChatCompletionRequest {
    ChatCompletionRequest {
        messages,
        temperature: None,
        top_p: None,
        max_tokens: None,
        stream: Some(false),
        tools: None,
        tool_choice: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{help_build_request, run};
    use crate::llm::{ChatMessage, MessageContent};

    /// 验证用户提问“你好”会被封装成非流式聊天请求。
    #[test]
    fn run_builds_greeting_request() {
        let request = help_build_request(vec![ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("你好".to_string()),
        }]);

        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
        assert!(matches!(
            &request.messages[0].content,
            MessageContent::Text(text) if text == "你好"
        ));
        assert_eq!(request.stream, Some(false));
        assert!(request.tools.is_none());
        assert!(request.tool_choice.is_none());
    }

    /// 使用真实 Agnes API 直接运行聊天 Agent，验证“你好”的完整调用链。
    #[tokio::test]
    #[ignore = "需要 AGNES_API_KEY、网络和真实 Agnes API"]
    async fn run_greeting_with_real_model() {
        let api_key = std::env::var("AGNES_API_KEY").expect("请设置 AGNES_API_KEY");
        crate::config::set_runtime_config(crate::config::EffectiveConfig {
            api_key: Some(api_key),
            base_url: "https://api.agnes-ai.cn/v1".to_string(),
            model: "agnes-3.0-flash".to_string(),
            image_model: "agnes-image-2.5-flash".to_string(),
            video_model: "agnes-video-2.5-flash".to_string(),
        })
        .expect("初始化测试配置失败");

        let response = run(vec![ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("你好".to_string()),
        }])
        .await
        .expect("聊天 Agent 调用失败");
        print!("response: {:?}", response);

        assert!(!response.choices.is_empty());
        assert!(response.choices[0].message.content.is_some());
    }
}
