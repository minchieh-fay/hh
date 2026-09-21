use tauri::AppHandle;

use crate::config::{self as config_module, ConfigFile, EffectiveConfig};

/// 为前端读取当前生效的应用配置。
#[tauri::command]
pub async fn get_config(app: AppHandle) -> Result<EffectiveConfig, String> {
    // step.1 通过配置业务模块读取并解析配置
    config_module::get_config(&app).await
}

/// 为前端保存应用配置并返回保存后的生效配置。
#[tauri::command]
pub async fn save_config(app: AppHandle, config: ConfigFile) -> Result<EffectiveConfig, String> {
    // step.1 通过配置业务模块校验并持久化配置
    config_module::save_config(&app, config).await
}
