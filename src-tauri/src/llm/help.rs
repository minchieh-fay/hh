use reqwest::Response;

/// 校验远程响应并将服务端错误转换为不泄露密钥的业务错误。
pub(super) async fn help_check_response(response: Response) -> Result<Response, String> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let detail = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(|error| error.get("message"))
                .or_else(|| value.get("detail"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "远程模型返回了未知错误".to_string());
    Err(format!("模型请求失败（{}）：{}", status.as_u16(), detail))
}

/// 校验用户提供的图片输入，允许公开 URL 和图片 Data URI。
pub(super) fn help_validate_image_input(value: &str) -> Result<(), String> {
    let is_url = value.starts_with("https://") || value.starts_with("http://");
    let is_data_uri = value.starts_with("data:image/") && value.contains(";base64,");
    if is_url || is_data_uri {
        return Ok(());
    }
    Err("图片必须是 http(s) URL 或图片 base64 Data URI".to_string())
}
