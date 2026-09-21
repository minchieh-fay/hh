use tauri::AppHandle;

/// 图片模型适配器自己的运行配置。
#[derive(Debug, Clone)]
pub(super) struct Config {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    pub endpoint: String,
    pub default_size: String,
    pub default_ratio: String,
    pub max_reference_images: usize,
}

/// 从全局配置构造图片模块配置，并集中定义图片接口默认参数。
pub(super) async fn load(app: &AppHandle) -> Result<Config, String> {
    let global = crate::config::get_config(app).await?;
    Ok(Config {
        api_key: global.api_key,
        base_url: global.base_url,
        model: global.image_model,
        endpoint: "/images/generations".to_string(),
        default_size: "1K".to_string(),
        default_ratio: "1:1".to_string(),
        max_reference_images: 5,
    })
}
