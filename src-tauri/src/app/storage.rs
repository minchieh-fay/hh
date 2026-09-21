use serde::Serialize;
use tauri::AppHandle;

use crate::store;

/// 前端可使用的应用存储信息。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub config_file: String,
    pub database_file: String,
    pub sessions_dir: String,
    pub skills_dir: String,
    pub media_dir: String,
    pub cache_dir: String,
    pub logs_dir: String,
}

/// 为前端提供经过业务层封装的应用存储信息。
#[tauri::command]
pub fn get_storage_info(app: AppHandle) -> Result<StorageInfo, String> {
    // step.1 通过基础设施层获取当前平台的应用路径
    let paths = store::get_storage_paths(&app)?;
    // step.2 返回前端业务需要的存储信息
    Ok(StorageInfo {
        config_file: paths.config_file,
        database_file: paths.database_file,
        sessions_dir: paths.sessions_dir,
        skills_dir: paths.skills_dir,
        media_dir: paths.media_dir,
        cache_dir: paths.cache_dir,
        logs_dir: paths.logs_dir,
    })
}
