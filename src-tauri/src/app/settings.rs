use serde::Serialize;
use tauri::AppHandle;

use crate::config::{self as config_module, ConfigFile, EffectiveConfig};

/// 前端可见的配置摘要，不回传已保存的 API key。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInfo {
    pub api_key_configured: bool,
    pub base_url: String,
    pub model: String,
    pub image_model: String,
    pub video_model: String,
}

/// 为前端读取当前生效的应用配置。
#[tauri::command]
pub async fn get_config(app: AppHandle) -> Result<SettingsInfo, String> {
    // step.1 通过配置业务模块读取并解析配置
    let config = config_module::get_config(&app).await?;
    Ok(help_to_settings_info(config))
}

/// 为前端保存应用配置并返回保存后的生效配置。
#[tauri::command]
pub async fn save_config(app: AppHandle, config: ConfigFile) -> Result<SettingsInfo, String> {
    // step.1 通过配置业务模块校验并持久化配置
    let config = config_module::save_config(&app, config).await?;
    Ok(help_to_settings_info(config))
}

/// 将内部生效配置转换为不包含凭据的前端配置摘要。
fn help_to_settings_info(config: EffectiveConfig) -> SettingsInfo {
    SettingsInfo {
        api_key_configured: config.api_key.is_some(),
        base_url: config.base_url,
        model: config.model,
        image_model: config.image_model,
        video_model: config.video_model,
    }
}

#[cfg(test)]
mod tests {
    use super::{help_to_settings_info, EffectiveConfig};

    /// 验证前端配置响应只暴露 API key 是否存在，不包含密钥内容。
    #[test]
    fn settings_response_does_not_expose_api_key() {
        let response = help_to_settings_info(EffectiveConfig {
            api_key: Some("secret-key".to_string()),
            base_url: "https://example.test/v1".to_string(),
            model: "chat-model".to_string(),
            image_model: "image-model".to_string(),
            video_model: "video-model".to_string(),
        });
        let serialized = serde_json::to_string(&response).expect("settings should serialize");

        assert!(response.api_key_configured);
        assert!(!serialized.contains("secret-key"));
    }
}
