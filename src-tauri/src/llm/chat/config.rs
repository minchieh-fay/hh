use tauri::AppHandle;

/// 聊天模型适配器自己的运行配置。
#[derive(Debug, Clone)]
pub(super) struct Config {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    pub endpoint: String,
    pub supports_image: bool,
}

/// 从全局配置构造聊天模块配置，并保留聊天协议自己的默认能力。
pub(super) async fn load(app: &AppHandle) -> Result<Config, String> {
    let global = crate::config::get_config(app).await?;
    Ok(Config {
        api_key: global.api_key,
        base_url: global.base_url,
        model: global.model,
        endpoint: "/chat/completions".to_string(),
        supports_image: true,
    })
}
