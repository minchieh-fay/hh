use crate::config::EffectiveConfig;

/// 聊天模型适配器自己的运行配置。
#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    pub endpoint: String,
    pub supports_image: bool,
}

/// 从全局配置构造聊天模块配置，并保留聊天协议自己的默认能力。
pub(crate) fn from_global(global: &EffectiveConfig) -> Config {
    Config {
        api_key: global.api_key.clone(),
        base_url: global.base_url.clone(),
        model: global.model.clone(),
        endpoint: "/chat/completions".to_string(),
        supports_image: true,
    }
}
