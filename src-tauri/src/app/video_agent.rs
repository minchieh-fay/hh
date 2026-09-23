use tauri::AppHandle;

use crate::{agent, llm};

/// 查询视频 Agent 创建的生成任务状态。
#[tauri::command]
pub async fn get_video_task(
    app: AppHandle,
    video_id: String,
) -> Result<llm::VideoResponse, String> {
    // step.1 刷新配置并交给视频 Agent 查询任务
    crate::config::get_config(&app).await?;
    agent::video::get_task(video_id).await
}
