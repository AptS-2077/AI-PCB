use crate::{
    ApiError, AppState,
    engine::DesignEngine,
    openai::{ApiConfig, engine_from_config},
    payload,
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

pub(crate) struct RuntimeSettings {
    pub engine: Arc<dyn DesignEngine>,
    pub api: ApiConfig,
    pub version: u64,
    pub verified: bool,
    pub tools: crate::eda::ToolConfig,
    pub supplier_key: String,
}

impl RuntimeSettings {
    pub fn new(engine: Arc<dyn DesignEngine>) -> Self {
        let api = engine.configuration().unwrap_or_default();
        Self {
            engine,
            api,
            version: 0,
            verified: false,
            tools: Default::default(),
            supplier_key: std::env::var("MOUSER_API_KEY").unwrap_or_default(),
        }
    }

    pub fn view(&self) -> Value {
        json!({
            "api": self.api,
            "key_configured": !self.api.api_key.is_empty(),
            "key_storage": "process_memory",
            "config_version": self.version,
            "verified": self.verified,
            "generation": self.engine.info(),
            "tools": self.tools,
            "tool_availability": self.tools.availability(),
            "supplier_configured": !self.supplier_key.is_empty(),
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolsRequest {
    pub expected_version: u64,
    pub tools: crate::eda::ToolConfig,
}

pub async fn save_tools(
    State(state): State<AppState>,
    request: Result<Json<ToolsRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let request = payload(request)?;
    request
        .tools
        .validate()
        .map_err(|error| ApiError::invalid(error.to_string()))?;
    let _permit = state.exclusive_operation()?;
    let mut runtime = state.runtime.write().map_err(ApiError::internal)?;
    if runtime.version != request.expected_version {
        return Err(ApiError::conflict());
    }
    runtime.tools = request.tools;
    runtime.version += 1;
    Ok(Json(runtime.view()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsRequest {
    pub expected_version: u64,
    pub api: ApiConfig,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestRequest {
    pub expected_version: u64,
    pub allow_remote: bool,
}

pub async fn get_settings(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        state.runtime.read().map_err(ApiError::internal)?.view(),
    ))
}

pub async fn save_settings(
    State(state): State<AppState>,
    request: Result<Json<SettingsRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let mut request = payload(request)?;
    let _permit = state.exclusive_operation()?;
    let mut runtime = state.runtime.write().map_err(ApiError::internal)?;
    if request.expected_version != runtime.version {
        return Err(ApiError::conflict());
    }
    // A retained credential must never silently move to a different endpoint.
    if request.api.api_key.trim().is_empty() {
        if request.api.provider != runtime.api.provider
            || request.api.base_url.trim().trim_end_matches('/')
                != runtime.api.base_url.trim().trim_end_matches('/')
        {
            return Err(ApiError::invalid(
                "切换服务商或 API 地址时必须重新输入对应密钥。",
            ));
        }
        request.api.api_key = runtime.api.api_key.clone();
    }
    let engine =
        engine_from_config(request.api).map_err(|error| ApiError::invalid(error.to_string()))?;
    runtime.api = engine
        .configuration()
        .ok_or_else(|| ApiError::internal("Missing API configuration"))?;
    runtime.engine = engine;
    runtime.version += 1;
    runtime.verified = false;
    Ok(Json(runtime.view()))
}

pub async fn clear_settings(
    State(state): State<AppState>,
    request: Result<Json<TestRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let request = payload(request)?;
    let _permit = state.exclusive_operation()?;
    let mut runtime = state.runtime.write().map_err(ApiError::internal)?;
    if runtime.version != request.expected_version {
        return Err(ApiError::conflict());
    }
    runtime.api.api_key.clear();
    runtime.engine = Arc::new(crate::engine::DemoEngine);
    runtime.version += 1;
    runtime.verified = false;
    Ok(Json(runtime.view()))
}

pub async fn test_settings(
    State(state): State<AppState>,
    request: Result<Json<TestRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let request = payload(request)?;
    let (_permit, engine) =
        state.begin_generation(request.allow_remote, request.expected_version)?;
    if !engine.info().requires_remote_consent {
        return Err(ApiError::invalid("请先保存 API 配置。"));
    }
    state.runtime.write().map_err(ApiError::internal)?.verified = false;
    let design = engine
        .generate(
            "连接测试：设计一个低压无源 RC 低通滤波电路，输入不超过 5V。",
            &Default::default(),
        )
        .await?;
    if design.ai.is_none() {
        return Err(ApiError {
            status: StatusCode::BAD_GATEWAY,
            message: "测试没有返回远程模型结果。".into(),
        });
    }
    let mut runtime = state.runtime.write().map_err(ApiError::internal)?;
    runtime.verified = true;
    Ok(Json(runtime.view()))
}
