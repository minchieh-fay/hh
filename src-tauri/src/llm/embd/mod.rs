use serde::Serialize;

mod config;

#[derive(Debug, Serialize)]
pub struct EmbeddingInfo {
    pub model: &'static str,
    pub dimension: u32,
    pub max_tokens: u32,
    pub quantized: bool,
}

/// 返回内置 bge-small-zh-v1.5 embedding 模型的能力信息。
pub fn embedding_info() -> EmbeddingInfo {
    EmbeddingInfo {
        model: config::MODEL_NAME,
        dimension: config::MODEL_DIMENSION,
        max_tokens: config::MODEL_MAX_TOKENS,
        quantized: config::MODEL_QUANTIZED,
    }
}
