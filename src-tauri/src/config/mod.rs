mod help;

use serde::{Deserialize, Serialize};
use std::sync::{OnceLock, RwLock};
use tauri::AppHandle;

use self::help::{help_parse_config, help_resolve_config, help_serialize_config};
use crate::store;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    pub image_model: String,
    pub video_model: String,
}

static RUNTIME_CONFIG: OnceLock<RwLock<Option<EffectiveConfig>>> = OnceLock::new();

/// 将应用层读取到的配置写入进程内缓存，供业务模块按需适配。
pub(crate) fn set_runtime_config(config: EffectiveConfig) -> Result<(), String> {
    let cache = RUNTIME_CONFIG.get_or_init(|| RwLock::new(None));
    let mut current = cache
        .write()
        .map_err(|_| "获取配置缓存写锁失败".to_string())?;
    *current = Some(config);
    Ok(())
}

/// 获取业务模块使用的进程内配置快照。
pub(crate) fn get_runtime_config() -> Result<EffectiveConfig, String> {
    let cache = RUNTIME_CONFIG
        .get()
        .ok_or_else(|| "应用配置尚未初始化".to_string())?;
    let current = cache
        .read()
        .map_err(|_| "获取配置缓存读锁失败".to_string())?;
    current
        .clone()
        .ok_or_else(|| "应用配置尚未初始化".to_string())
}

/// 读取配置文件并返回当前生效的 LLM 配置。
pub(crate) async fn get_config(app: &AppHandle) -> Result<EffectiveConfig, String> {
    // step.1 定位并读取用户配置文件
    let config = help_parse_config(store::read_config_file(app)?)?;
    // step.2 按用户配置生成基础配置并动态更新模型
    let config = help_fetch_latest_models(help_resolve_config(config)).await?;
    set_runtime_config(config.clone())?;
    Ok(config)
}

/// 写入 LLM 配置文件并返回当前生效的配置。
pub(crate) async fn save_config(
    app: &AppHandle,
    config: ConfigFile,
) -> Result<EffectiveConfig, String> {
    // step.1 定位配置文件并持久化用户配置
    let content = help_serialize_config(&config)?;
    store::write_config_file(app, &content)?;
    // step.2 返回保存后动态获取的最新模型
    let config = help_fetch_latest_models(help_resolve_config(config)).await?;
    set_runtime_config(config.clone())?;
    Ok(config)
}

/// 使用 API key 拉取并选择三类版本号最大的 flash 模型。
async fn help_fetch_latest_models(mut config: EffectiveConfig) -> Result<EffectiveConfig, String> {
    // step.1 没有用户 API key 时使用本地默认模型
    let Some(api_key) = config.api_key.as_deref() else {
        return Ok(config);
    };

    // step.2 请求模型列表并校验远端响应
    let response = reqwest::Client::new()
        .get(format!("{}/models", config.base_url))
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|error| format!("查询模型失败: {error}"))?
        .error_for_status()
        .map_err(|error| format!("查询模型接口返回错误: {error}"))?
        .json::<ModelListResponse>()
        .await
        .map_err(|error| format!("解析模型列表失败: {error}"))?;

    // step.3 按三类业务分别选择最高版本模型
    config.model = help_select_model(&response.data, ModelKind::Text).unwrap_or(config.model);
    config.image_model =
        help_select_model(&response.data, ModelKind::Image).unwrap_or(config.image_model);
    config.video_model =
        help_select_model(&response.data, ModelKind::Video).unwrap_or(config.video_model);
    Ok(config)
}

#[derive(Debug, Deserialize)]
struct ModelListResponse {
    data: Vec<ModelItem>,
}

#[derive(Debug, Deserialize)]
struct ModelItem {
    id: String,
}

#[derive(Clone, Copy)]
enum ModelKind {
    Text,
    Image,
    Video,
}

/// 从模型列表中选择指定类别的最高版本模型。
fn help_select_model(models: &[ModelItem], kind: ModelKind) -> Option<String> {
    models
        .iter()
        .filter_map(|model| {
            let version = help_model_version(&model.id, kind)?;
            Some((version, model.id.clone()))
        })
        .max_by_key(|(version, _)| *version)
        .map(|(_, model)| model)
}

/// 校验模型命名并解析其主次版本号。
fn help_model_version(model: &str, kind: ModelKind) -> Option<(u32, u32)> {
    let prefix = match kind {
        ModelKind::Text => "agnes-",
        ModelKind::Image => "agnes-image-",
        ModelKind::Video => "agnes-video-",
    };
    let version = model.strip_prefix(prefix)?.strip_suffix("-flash")?;
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    (parts.next().is_none()).then_some((major, minor))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证空配置能够使用固定的 Base URL 和默认模型。
    #[test]
    fn empty_file_config_uses_defaults() {
        let result = help_resolve_config(ConfigFile::default());

        assert_eq!(result.base_url, "https://api.agnes-ai.cn/v1");
        assert_eq!(result.model, "agnes-3.0-flash");
    }

    /// 验证 API key 只使用配置文件中的值。
    #[test]
    fn file_api_key_is_used_without_environment_fallback() {
        let result = help_resolve_config(ConfigFile {
            api_key: Some("file-key".to_string()),
        });

        assert_eq!(result.api_key, Some("file-key".to_string()));
    }
}
