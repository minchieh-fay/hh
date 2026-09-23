use crate::llm::{self, VideoGenerationRequest, VideoResponse};

mod config;

/// 将网关提取的提示词和图片交给视频模型创建任务。
pub async fn run(prompt: String, images: Vec<String>) -> Result<VideoResponse, String> {
    let current_config = config::load()?;
    llm::create_video(
        &current_config,
        VideoGenerationRequest {
            prompt,
            seconds: None,
            aspect_ratio: None,
            images,
        },
    )
    .await
}

/// 查询视频 Agent 创建的异步生成任务。
pub async fn get_task(video_id: String) -> Result<VideoResponse, String> {
    let current_config = config::load()?;
    llm::get_video(&current_config, video_id).await
}
