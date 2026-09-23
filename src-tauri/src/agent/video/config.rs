use crate::{config, llm};

/// 获取视频 Agent 所需的视频模型配置。
pub(super) fn load() -> Result<llm::video::config::Config, String> {
    let global = config::get_runtime_config()?;
    Ok(llm::video::config::from_global(&global))
}
