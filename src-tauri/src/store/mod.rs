mod help;

use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use self::help::help_build_storage_paths;

/// 应用内部使用的存储路径集合。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePaths {
    pub config_dir: String,
    pub config_file: String,
    pub data_dir: String,
    pub database_dir: String,
    pub database_file: String,
    pub sessions_dir: String,
    pub skills_dir: String,
    pub media_dir: String,
    pub cache_dir: String,
    pub logs_dir: String,
}

/// 获取应用在当前平台上的存储路径。
pub(crate) fn get_storage_paths(app: &AppHandle) -> Result<StoragePaths, String> {
    // step.1 按桌面端和移动端的存储约束获取基础目录
    let resolver = app.path();
    #[cfg(not(mobile))]
    let (config_dir, data_dir, cache_dir, logs_dir) = {
        let root_dir = resolver
            .home_dir()
            .map_err(|error| format!("获取用户主目录失败: {error}"))?
            .join(".hh");
        (
            root_dir.clone(),
            root_dir.clone(),
            root_dir.join("cache"),
            root_dir.join("logs"),
        )
    };

    #[cfg(mobile)]
    let (config_dir, data_dir, cache_dir, logs_dir) = {
        let config_dir = resolver
            .app_config_dir()
            .map_err(|error| format!("获取应用配置目录失败: {error}"))?;
        let data_dir = resolver
            .app_data_dir()
            .map_err(|error| format!("获取应用数据目录失败: {error}"))?;
        let cache_dir = resolver
            .app_cache_dir()
            .map_err(|error| format!("获取应用缓存目录失败: {error}"))?;
        let logs_dir = resolver
            .app_log_dir()
            .map_err(|error| format!("获取应用日志目录失败: {error}"))?;
        (config_dir, data_dir, cache_dir, logs_dir)
    };

    // step.2 按业务边界组织应用数据目录
    Ok(help_build_storage_paths(
        config_dir, data_dir, cache_dir, logs_dir,
    ))
}

/// 获取配置文件的完整路径，供配置模块复用。
pub(crate) fn config_file_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    #[cfg(not(mobile))]
    {
        return app
            .path()
            .home_dir()
            .map(|path| path.join(".hh").join("config.json"))
            .map_err(|error| format!("获取用户主目录失败: {error}"));
    }

    #[cfg(mobile)]
    {
        return app
            .path()
            .app_config_dir()
            .map(|path| path.join("config.json"))
            .map_err(|error| format!("获取应用配置目录失败: {error}"));
    }
}

/// 读取应用配置文件内容。
pub(crate) fn read_config_file(app: &AppHandle) -> Result<Option<String>, String> {
    let path = config_file_path(app)?;
    if !path.exists() {
        return Ok(None);
    }

    std::fs::read_to_string(path)
        .map(Some)
        .map_err(|error| format!("读取配置文件失败: {error}"))
}

/// 写入应用配置文件内容。
pub(crate) fn write_config_file(app: &AppHandle, content: &str) -> Result<(), String> {
    // step.1 获取配置路径并准备配置目录
    let path = config_file_path(app)?;
    let parent = path
        .parent()
        .ok_or_else(|| "配置文件路径无效".to_string())?;
    std::fs::create_dir_all(parent).map_err(|error| format!("创建配置目录失败: {error}"))?;

    // step.2 写入临时文件并原子替换正式配置
    let temporary_path = PathBuf::from(format!("{}.tmp", path.display()));
    std::fs::write(&temporary_path, content)
        .map_err(|error| format!("写入临时配置失败: {error}"))?;
    if let Err(error) = std::fs::rename(&temporary_path, &path) {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(format!("替换配置文件失败: {error}"));
    }

    Ok(())
}
