use tauri::AppHandle;

/// 视频模型适配器自己的运行配置。
#[derive(Debug, Clone)]
pub(super) struct Config {
    pub api_key: Option<String>,
    pub base_url: String,
    pub query_base_url: String,
    pub model: String,
    pub create_endpoint: String,
    pub query_endpoint: String,
    pub mode: String,
    pub size: String,
    pub default_seconds: String,
    pub default_aspect_ratio: String,
    pub max_reference_images: usize,
}

/// 从全局配置构造视频模块配置，并集中定义 reference 模式约束。
pub(super) async fn load(app: &AppHandle) -> Result<Config, String> {
    let global = crate::config::get_config(app).await?;
    Ok(Config {
        api_key: global.api_key,
        base_url: global.base_url,
        query_base_url: "https://api.agnes-ai.cn".to_string(),
        model: global.video_model,
        create_endpoint: "/videos".to_string(),
        query_endpoint: "/agnesapi".to_string(),
        mode: "reference".to_string(),
        size: "720P".to_string(),
        default_seconds: "5".to_string(),
        default_aspect_ratio: "16:9".to_string(),
        max_reference_images: 5,
    })
}
