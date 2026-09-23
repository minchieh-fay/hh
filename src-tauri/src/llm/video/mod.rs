use super::help::{help_check_response, help_validate_image_input};
use serde::{Deserialize, Serialize};

pub(crate) mod config;

#[derive(Debug, Deserialize, Serialize)]
pub struct VideoGenerationRequest {
    pub prompt: String,
    pub seconds: Option<String>,
    pub aspect_ratio: Option<String>,
    pub images: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VideoResponse {
    pub id: Option<String>,
    pub video_id: Option<String>,
    pub task_id: Option<String>,
    pub status: Option<String>,
    pub url: Option<String>,
    pub error: Option<serde_json::Value>,
}

/// 创建 Agnes Video reference 模式任务，仅接受图片 base64 或公开 URL。
pub async fn create_video(
    current_config: &config::Config,
    request: VideoGenerationRequest,
) -> Result<VideoResponse, String> {
    // step.1 使用视频模块配置并校验 reference 模式的提示词、时长和图片数量
    if request.prompt.trim().is_empty() {
        return Err("视频提示词不能为空".to_string());
    }
    if request.images.is_empty() {
        return Err("reference 模式至少需要一张参考图片".to_string());
    }
    if request.images.len() > current_config.max_reference_images {
        return Err(format!(
            "reference 模式最多支持 {} 张参考图片",
            current_config.max_reference_images
        ));
    }
    for image in &request.images {
        help_validate_image_input(image)?;
    }
    // step.2 使用视频模块配置并固定使用 reference 协议
    let api_key = current_config
        .api_key
        .clone()
        .ok_or_else(|| "尚未配置 Agnes API key".to_string())?;
    let body = serde_json::json!({
        "model": current_config.model,
        "prompt": request.prompt,
        "seconds": request
            .seconds
            .unwrap_or_else(|| current_config.default_seconds.clone()),
        "mode": current_config.mode,
        "size": current_config.size,
        "aspect_ratio": request
            .aspect_ratio
            .unwrap_or_else(|| current_config.default_aspect_ratio.clone()),
        "n": 1,
        "images": request.images,
    });
    // step.3 创建异步任务并返回任务标识
    let response = reqwest::Client::new()
        .post(format!(
            "{}{}",
            current_config.base_url, current_config.create_endpoint
        ))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("创建视频任务失败：{error}"))?;
    help_check_response(response)
        .await?
        .json::<VideoResponse>()
        .await
        .map_err(|error| format!("解析视频任务响应失败：{error}"))
}

/// 查询 Agnes Video reference 模式任务状态和最终视频地址。
pub async fn get_video(
    current_config: &config::Config,
    video_id: String,
) -> Result<VideoResponse, String> {
    // step.1 校验任务标识并使用视频模块配置
    if video_id.trim().is_empty() {
        return Err("视频任务标识不能为空".to_string());
    }
    let api_key = current_config
        .api_key
        .clone()
        .ok_or_else(|| "尚未配置 Agnes API key".to_string())?;
    // step.2 按 reference 模式要求查询异步任务
    let url = format!(
        "{}{}?video_id={}&model_name={}",
        current_config.query_base_url,
        current_config.query_endpoint,
        video_id,
        current_config.model
    );
    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|error| format!("查询视频任务失败：{error}"))?;
    // step.3 校验并解析任务状态
    help_check_response(response)
        .await?
        .json::<VideoResponse>()
        .await
        .map_err(|error| format!("解析视频任务响应失败：{error}"))
}
