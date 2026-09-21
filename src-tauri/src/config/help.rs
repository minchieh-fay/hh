use super::{ConfigFile, EffectiveConfig};

/// 从配置文件读取配置；文件不存在时返回默认的空配置。
pub(super) fn help_parse_config(content: Option<String>) -> Result<ConfigFile, String> {
    let Some(content) = content else {
        return Ok(ConfigFile::default());
    };

    serde_json::from_str(&content).map_err(|error| format!("配置文件格式错误: {error}"))
}

/// 将用户配置转换为固定站点下的基础配置。
pub(super) fn help_resolve_config(config: ConfigFile) -> EffectiveConfig {
    EffectiveConfig {
        api_key: config.api_key.filter(|value| !value.trim().is_empty()),
        base_url: "https://api.agnes-ai.cn/v1".to_string(),
        model: "agnes-3.0-flash".to_string(),
        image_model: "agnes-image-2.5-flash".to_string(),
        video_model: "agnes-video-2.5-flash".to_string(),
    }
}

/// 序列化需要保存的用户配置。
pub(super) fn help_serialize_config(config: &ConfigFile) -> Result<String, String> {
    serde_json::to_string_pretty(config).map_err(|error| format!("序列化配置失败: {error}"))
}
