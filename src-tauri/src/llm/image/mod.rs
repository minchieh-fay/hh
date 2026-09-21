use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use super::help::{help_check_response, help_validate_image_input};

mod config;

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageGenerationRequest {
    pub prompt: String,
    pub size: String,
    pub ratio: Option<String>,
    pub images: Option<Vec<String>>,
    pub return_base64: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageGenerationResponse {
    pub created: Option<u64>,
    pub data: Vec<ImageData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageData {
    pub url: Option<String>,
    pub b64_json: Option<String>,
}

/// 调用 Agnes 图片生成接口，兼容文生图和 base64 参考图生图。
pub async fn generate_image(
    app: &AppHandle,
    request: ImageGenerationRequest,
) -> Result<ImageGenerationResponse, String> {
    // step.1 加载图片模块配置并校验提示词、尺寸和参考图片
    let current_config = config::load(app).await?;
    if request.prompt.trim().is_empty() {
        return Err("图片提示词不能为空".to_string());
    }
    let size = if request.size.trim().is_empty() {
        current_config.default_size.clone()
    } else {
        request.size.clone()
    };
    let ratio = request
        .ratio
        .clone()
        .unwrap_or_else(|| current_config.default_ratio.clone());
    if let Some(images) = &request.images {
        if images.len() > current_config.max_reference_images {
            return Err(format!(
                "图片参考图最多支持 {} 张",
                current_config.max_reference_images
            ));
        }
        for image in images {
            help_validate_image_input(image)?;
        }
    }
    // step.2 使用图片模块配置并按 Agnes 协议组装请求
    let api_key = current_config
        .api_key
        .ok_or_else(|| "尚未配置 Agnes API key".to_string())?;
    let body = serde_json::json!({
        "model": current_config.model,
        "prompt": request.prompt,
        "size": size,
        "ratio": ratio,
        "return_base64": request.return_base64.unwrap_or(false),
        "extra_body": {
            "image": request.images,
            "response_format": if request.return_base64.unwrap_or(false) {
                "b64_json"
            } else {
                "url"
            }
        }
    });
    // step.3 请求服务并返回 URL 或 base64 结果
    let response = reqwest::Client::new()
        .post(format!(
            "{}{}",
            current_config.base_url, current_config.endpoint
        ))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("调用图片模型失败：{error}"))?;
    help_check_response(response)
        .await?
        .json::<ImageGenerationResponse>()
        .await
        .map_err(|error| format!("解析图片模型响应失败：{error}"))
}
