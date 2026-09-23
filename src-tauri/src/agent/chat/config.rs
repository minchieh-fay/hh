use crate::{config, llm};

/// 获取聊天 Agent 所需的聊天模型配置。
pub(super) fn load() -> Result<llm::chat::config::Config, String> {
    let global = config::get_runtime_config()?;
    Ok(llm::chat::config::from_global(&global))
}
