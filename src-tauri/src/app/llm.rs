use tauri::AppHandle;

use crate::llm;

/// 调用文本模型完成一次非流式 Chat Completions 请求。
#[tauri::command]
pub async fn chat_completion(
    app: AppHandle,
    request: llm::ChatCompletionRequest,
) -> Result<llm::ChatCompletionResponse, String> {
    llm::chat_completion(&app, request).await
}

/// 调用图片模型生成图片或执行参考图生图。
#[tauri::command]
pub async fn generate_image(
    app: AppHandle,
    request: llm::ImageGenerationRequest,
) -> Result<llm::ImageGenerationResponse, String> {
    llm::generate_image(&app, request).await
}

/// 创建 Agnes Video reference 模式任务。
#[tauri::command]
pub async fn create_video(
    app: AppHandle,
    request: llm::VideoGenerationRequest,
) -> Result<llm::VideoResponse, String> {
    llm::create_video(&app, request).await
}

/// 查询 Agnes Video reference 模式任务。
#[tauri::command]
pub async fn get_video(app: AppHandle, video_id: String) -> Result<llm::VideoResponse, String> {
    llm::get_video(&app, video_id).await
}

/// 返回本地 embedding 模型的固定能力信息。
#[tauri::command]
pub fn get_embedding_info() -> llm::EmbeddingInfo {
    llm::embedding_info()
}
