use crate::{ApiError, AppState, model::timestamp, payload};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupplierSettings {
    expected_version: u64,
    api_key: String,
}

pub async fn save(
    State(state): State<AppState>,
    request: Result<Json<SupplierSettings>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let request = payload(request)?;
    let key = request.api_key.trim();
    if key.len() > 256 || key.chars().any(|c| !c.is_ascii_graphic()) {
        return Err(ApiError::invalid("Mouser Search API 密钥格式不正确。"));
    }
    let _permit = state.exclusive_operation()?;
    let mut runtime = state.runtime.write().map_err(ApiError::internal)?;
    if runtime.version != request.expected_version {
        return Err(ApiError::conflict());
    }
    runtime.supplier_key = key.into();
    runtime.version += 1;
    Ok(Json(runtime.view()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    keyword: String,
    allow_remote: bool,
}

pub async fn search(
    State(state): State<AppState>,
    request: Result<Json<SearchRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let request = payload(request)?;
    if !request.allow_remote {
        return Err(ApiError::invalid("查询关键词将发送至 Mouser，请先授权。"));
    }
    let keyword = request.keyword.trim();
    if !(2..=100).contains(&keyword.chars().count()) {
        return Err(ApiError::invalid("查询关键词须为 2-100 个字符。"));
    }
    let _permit = state.exclusive_operation()?;
    let key = state
        .runtime
        .read()
        .map_err(ApiError::internal)?
        .supplier_key
        .clone();
    if key.is_empty() {
        return Err(ApiError::invalid(
            "请先配置独立的 Mouser Search API Key，OpenAI 密钥不能用于器件数据库。",
        ));
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(25))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| ApiError::invalid("无法创建供应商连接"))?;
    let mut response = client.post("https://api.mouser.com/api/v2/search/keywordandmanufacturer")
        .query(&[("apiKey", &key)])
        .json(&json!({ "SearchByKeywordMfrNameRequest": { "keyword": keyword, "records": 10, "pageNumber": 1, "searchOptions": "None" } }))
        .send().await.map_err(|_| ApiError::invalid("Mouser 查询失败或超时；未返回模拟库存。"))?;
    if !response.status().is_success() {
        return Err(ApiError::invalid(
            "Mouser 拒绝查询，请检查 Search API 权限、密钥和配额。",
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| ApiError::invalid("器件查询响应不完整。"))?
    {
        if bytes.len() + chunk.len() > 1024 * 1024 {
            return Err(ApiError::invalid("器件响应超过大小限制。"));
        }
        bytes.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| ApiError::invalid("器件 API 响应不是有效 JSON。"))?;
    if value
        .get("Errors")
        .and_then(Value::as_array)
        .is_some_and(|errors| !errors.is_empty())
    {
        return Err(ApiError::invalid(
            "Mouser 返回查询错误，请核对 Search API 授权和查询条件。",
        ));
    }
    let parts = value
        .pointer("/SearchResults/Parts")
        .and_then(Value::as_array)
        .ok_or_else(|| ApiError::invalid("器件 API 未返回有效 Parts 列表。"))?;
    let filtered: Vec<_> = parts.iter().take(10).map(|part| {
        let text = |name: &str| part.get(name).and_then(Value::as_str).unwrap_or("").chars().take(2000).collect::<String>().replace(&key, "[redacted]");
        let prices: Vec<_> = part.get("PriceBreaks").and_then(Value::as_array).into_iter().flatten().take(5).map(|p| json!({ "quantity": p.get("Quantity").and_then(Value::as_u64), "price": p.get("Price").and_then(Value::as_str).unwrap_or(""), "currency": p.get("Currency").and_then(Value::as_str).unwrap_or("") })).collect();
        json!({ "manufacturer": text("Manufacturer"), "mpn": text("ManufacturerPartNumber"), "supplier_part": text("MouserPartNumber"), "description": text("Description"), "availability": text("Availability"), "lifecycle": text("LifecycleStatus"), "prices": prices })
    }).collect();
    Ok(Json(
        json!({"provider": "Mouser Search API v2", "queried_at": timestamp(), "keyword": keyword, "parts": filtered}),
    ))
}
