use std::path::PathBuf;

use super::StoragePaths;

/// 根据平台目录生成应用业务存储路径。
pub(super) fn help_build_storage_paths(
    config_dir: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
    logs_dir: PathBuf,
) -> StoragePaths {
    // step.1 在应用数据目录下划分可独立管理的业务空间
    let database_dir = data_dir.join("database");
    let sessions_dir = data_dir.join("sessions");
    let skills_dir = data_dir.join("skills");
    let media_dir = data_dir.join("media");

    // step.2 返回前端和后端都可使用的标准化路径集合
    StoragePaths {
        config_file: config_dir.join("config.json").display().to_string(),
        config_dir: config_dir.display().to_string(),
        database_file: database_dir.join("hh.sqlite").display().to_string(),
        database_dir: database_dir.display().to_string(),
        sessions_dir: sessions_dir.display().to_string(),
        skills_dir: skills_dir.display().to_string(),
        media_dir: media_dir.display().to_string(),
        data_dir: data_dir.display().to_string(),
        cache_dir: cache_dir.display().to_string(),
        logs_dir: logs_dir.display().to_string(),
    }
}
