pub mod archive;
pub mod circuit;
pub mod eda;
pub mod engine;
pub mod export;
pub mod model;
pub mod openai;
pub mod repository;
pub mod settings;
pub mod sourcing;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State, rejection::JsonRejection},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use engine::DesignEngine;
use model::*;
use repository::Repository;
use serde_json::json;
use std::{
    io,
    sync::{Arc, Mutex, RwLock},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    repository: Arc<Mutex<Repository>>,
    runtime: Arc<RwLock<settings::RuntimeSettings>>,
    generation_slots: Arc<Semaphore>,
}

impl AppState {
    pub fn new(repository: Repository, engine: Arc<dyn DesignEngine>) -> Self {
        Self {
            repository: Arc::new(Mutex::new(repository)),
            runtime: Arc::new(RwLock::new(settings::RuntimeSettings::new(engine))),
            generation_slots: Arc::new(Semaphore::new(1)),
        }
    }

    fn begin_generation(
        &self,
        allow_remote: bool,
        version: u64,
    ) -> Result<(OwnedSemaphorePermit, Arc<dyn DesignEngine>), ApiError> {
        let permit = self.exclusive_operation()?;
        let runtime = self.runtime.read().map_err(ApiError::internal)?;
        if runtime.version != version {
            return Err(ApiError::conflict());
        }
        let engine = runtime.engine.clone();
        if engine.info().requires_remote_consent && !allow_remote {
            return Err(ApiError::invalid(
                "远程模型模式会把需求与板卡约束发送到配置的 API 服务（可能是第三方网关）。请先确认数据发送授权（allow_remote: true）。",
            ));
        }
        Ok((permit, engine))
    }

    fn exclusive_operation(&self) -> Result<OwnedSemaphorePermit, ApiError> {
        self.generation_slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| ApiError {
                status: StatusCode::TOO_MANY_REQUESTS,
                message: "已有生成、测试或工程任务正在运行，请等待完成后再试。".into(),
            })
    }

    async fn access<ResultValue, Operation>(
        &self,
        operation: Operation,
    ) -> Result<ResultValue, ApiError>
    where
        ResultValue: Send + 'static,
        Operation: FnOnce(&Repository) -> Result<ResultValue, ApiError> + Send + 'static,
    {
        let repository = self.repository.clone();
        tokio::task::spawn_blocking(move || {
            let repository = repository
                .lock()
                .map_err(|_| ApiError::internal("项目存储锁异常"))?;
            operation(&repository)
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    }
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }
    fn conflict() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: "项目已在其他窗口更新，请重新选择项目或刷新页面再试。".into(),
        }
    }
    fn internal(detail: impl std::fmt::Display) -> Self {
        eprintln!("AI-PCB internal error: {detail}");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "服务暂时无法读取或保存项目，请查看服务日志。".into(),
        }
    }
}

impl From<io::Error> for ApiError {
    fn from(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::NotFound {
            Self {
                status: StatusCode::NOT_FOUND,
                message: "项目不存在。".into(),
            }
        } else {
            Self::internal(error)
        }
    }
}

impl From<engine::EngineError> for ApiError {
    fn from(error: engine::EngineError) -> Self {
        Self {
            status: StatusCode::from_u16(error.status).unwrap_or(StatusCode::BAD_GATEWAY),
            message: error.message,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({"error": self.message}))).into_response()
    }
}

fn payload<Value>(payload: Result<Json<Value>, JsonRejection>) -> Result<Value, ApiError> {
    payload.map(|Json(value)| value).map_err(|error| ApiError {
        status: error.status(),
        message: "请求 JSON 格式错误、类型不正确、字段缺失或正文过大。".into(),
    })
}

fn project_id(value: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::invalid("项目 ID 格式不正确。"))
}

fn validate_prompt(prompt: &str) -> Result<(), ApiError> {
    if !(8..=4000).contains(&prompt.trim().chars().count()) {
        return Err(ApiError::invalid("请用 8–4000 个字符描述设计需求。"));
    }
    Ok(())
}

fn validate_constraints(constraints: &Constraints) -> Result<(), ApiError> {
    if !(30..=160).contains(&constraints.width_mm) || !(30..=120).contains(&constraints.height_mm) {
        return Err(ApiError::invalid(
            "板宽须为 30–160 mm，板高须为 30–120 mm。",
        ));
    }
    if ![2, 4].contains(&constraints.layers) {
        return Err(ApiError::invalid("Demo 支持 2 层或 4 层板意向。"));
    }
    if !(1..=5000).contains(&constraints.budget_cny) {
        return Err(ApiError::invalid("预算须为 1–5000 元。"));
    }
    Ok(())
}

pub fn seed(repository: &Repository) -> io::Result<()> {
    // Retained for API compatibility; an empty workspace must remain empty.
    let _ = repository;
    Ok(())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("../web/index.html")) }),
        )
        .route(
            "/styles.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../web/styles.css"),
                )
            }),
        )
        .route(
            "/app.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../web/app.js"),
                )
            }),
        )
        .route(
            "/board.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../web/board.js"),
                )
            }),
        )
        .route(
            "/settings.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../web/settings.js"),
                )
            }),
        )
        .route(
            "/favicon.svg",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml")],
                    include_str!("../web/favicon.svg"),
                )
            }),
        )
        .route("/api/health", get(health))
        .route(
            "/api/settings",
            get(settings::get_settings)
                .put(settings::save_settings)
                .delete(settings::clear_settings),
        )
        .route("/api/settings/test", post(settings::test_settings))
        .route(
            "/api/settings/tools",
            axum::routing::put(settings::save_tools),
        )
        .route("/api/settings/supplier", axum::routing::put(sourcing::save))
        .route("/api/components/search", post(sourcing::search))
        .route(
            "/api/templates",
            get(|| async { Json(engine::templates()) }),
        )
        .route("/api/components", get(|| async { Json(engine::catalog()) }))
        .route("/api/projects", get(list_projects).post(create_project))
        .route(
            "/api/projects/{id}",
            get(get_project).delete(delete_project),
        )
        .route("/api/projects/{id}/generate", post(generate_project))
        .route("/api/projects/{id}/review", post(review_project))
        .route("/api/projects/{id}/engineering", post(run_engineering))
        .route(
            "/api/projects/{id}/artifacts/{index}",
            get(download_artifact),
        )
        .route("/api/projects/{id}/export/{format}", get(export_project))
        .fallback(|| async {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "页面或接口不存在。"})),
            )
        })
        .layer(DefaultBodyLimit::max(32 * 1024))
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let runtime = state.runtime.read().map_err(ApiError::internal)?;
    let generation = runtime.engine.info();
    Ok(Json(json!({
        "status": "ok", "version": env!("CARGO_PKG_VERSION"),
        "engine": match generation.provider { "openai" => "openai-responses", _ => "rule-based-demo" },
        "generation": generation,
        "manufacturing_ready": false,
        "upstream_verified": runtime.verified,
        "config_version": runtime.version,
    })))
}

async fn security_headers(request: axum::extract::Request, next: Next) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    let host_url = host.and_then(|value| reqwest::Url::parse(&format!("http://{value}")).ok());
    let invalid_host = host.is_some()
        && !host_url.as_ref().is_some_and(|url| {
            url.username().is_empty()
                && url.password().is_none()
                && url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none()
                && url.host_str().is_some_and(|value| {
                    value == "localhost"
                        || value
                            .trim_matches(['[', ']'])
                            .parse::<std::net::IpAddr>()
                            .is_ok_and(|ip| ip.is_loopback())
                })
        });
    let cross_origin = request.headers().get(header::ORIGIN).is_some_and(|origin| {
        origin.to_str().ok()
            != host_url
                .as_ref()
                .map(|url| url.origin().ascii_serialization())
                .as_deref()
    });
    let homepage_navigation = request.method() == axum::http::Method::GET
        && request.uri().path() == "/"
        && request
            .headers()
            .get("sec-fetch-mode")
            .is_some_and(|value| value == "navigate")
        && request
            .headers()
            .get("sec-fetch-dest")
            .is_some_and(|value| value == "document");
    let cross_site = request
        .headers()
        .get("sec-fetch-site")
        .is_some_and(|value| value == "cross-site")
        && !homepage_navigation;
    if invalid_host || cross_origin || cross_site {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "只允许本机同源访问。"})),
        )
            .into_response();
    }
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"));
    response
}

async fn list_projects(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let projects = state.access(|repository| Ok(repository.list()?)).await?;
    Ok(Json(json!(projects.iter().map(|project| json!({
        "id": project.id, "name": project.name, "revision": project.revision,
        "status": project.status, "updated_at": project.updated_at,
        "template_id": project.design.as_ref().map(|design| &design.template_id),
        "component_count": project.design.as_ref().map(|design| design.footprints.len()).unwrap_or(0),
    })).collect::<Vec<_>>())))
}

async fn create_project(
    State(state): State<AppState>,
    request: Result<Json<CreateRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<Project>), ApiError> {
    let request = payload(request)?;
    let name = request.name.trim();
    if !(1..=60).contains(&name.chars().count()) {
        return Err(ApiError::invalid("项目名称须为 1–60 个字符。"));
    }
    validate_prompt(&request.prompt)?;
    validate_constraints(&request.constraints)?;
    let (_permit, engine) = state.begin_generation(request.allow_remote, request.config_version)?;
    state
        .access(|repository| {
            if repository.list()?.len() >= 200 {
                return Err(ApiError::invalid(
                    "本地 Demo 最多保存 200 个项目，请使用新的数据目录。",
                ));
            }
            Ok(())
        })
        .await?;
    let prompt = request.prompt.trim().to_string();
    let mut design = engine.generate(&prompt, &request.constraints).await?;
    if let Some(spec) = &design.circuit {
        let tools = state
            .runtime
            .read()
            .map_err(ApiError::internal)?
            .tools
            .clone();
        design.engineering = Some(eda::build(spec, &request.constraints, &tools).await?);
    }
    let mut project = Project::new(name.into(), prompt.clone(), request.constraints.clone());
    project.apply_design(prompt, request.constraints, design);
    let project = state
        .access(move |repository| {
            if repository.list()?.len() >= 200 {
                return Err(ApiError::invalid(
                    "本地 Demo 最多保存 200 个项目，请使用新的数据目录。",
                ));
            }
            repository.save(&project)?;
            Ok(project)
        })
        .await?;
    Ok((StatusCode::CREATED, Json(project)))
}

async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, ApiError> {
    let id = project_id(&id)?;
    Ok(Json(
        state
            .access(move |repository| Ok(repository.get(id)?))
            .await?,
    ))
}

async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Result<Json<DeleteRequest>, JsonRejection>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = project_id(&id)?;
    let request = payload(request)?;
    let _permit = state.exclusive_operation()?;
    state
        .access(move |repository| {
            let project = repository.get(id)?;
            if project.revision != request.expected_revision {
                return Err(ApiError::conflict());
            }
            repository.delete(id)?;
            Ok(())
        })
        .await?;
    Ok(Json(json!({"deleted": id})))
}

async fn generate_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Result<Json<GenerateRequest>, JsonRejection>,
) -> Result<Json<Project>, ApiError> {
    let id = project_id(&id)?;
    let request = payload(request)?;
    validate_prompt(&request.prompt)?;
    validate_constraints(&request.constraints)?;
    let (_permit, engine) = state.begin_generation(request.allow_remote, request.config_version)?;
    let expected_revision = request.expected_revision;
    state
        .access(move |repository| {
            let project = repository.get(id)?;
            if project.revision != expected_revision {
                return Err(ApiError::conflict());
            }
            Ok(())
        })
        .await?;
    let prompt = request.prompt.trim().to_string();
    let mut design = engine.generate(&prompt, &request.constraints).await?;
    if let Some(spec) = &design.circuit {
        let tools = state
            .runtime
            .read()
            .map_err(ApiError::internal)?
            .tools
            .clone();
        design.engineering = Some(eda::build(spec, &request.constraints, &tools).await?);
    }
    let project = state
        .access(move |repository| {
            let mut project = repository.get(id)?;
            if project.revision != request.expected_revision {
                return Err(ApiError::conflict());
            }
            project.apply_design(prompt, request.constraints, design);
            repository.save(&project)?;
            Ok(project)
        })
        .await?;
    Ok(Json(project))
}

async fn run_engineering(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Result<Json<DeleteRequest>, JsonRejection>,
) -> Result<Json<Project>, ApiError> {
    let id = project_id(&id)?;
    let request = payload(request)?;
    let _permit = state.exclusive_operation()?;
    let project = state
        .access(move |repository| {
            let project = repository.get(id)?;
            if project.revision != request.expected_revision {
                return Err(ApiError::conflict());
            }
            Ok(project)
        })
        .await?;
    let spec = project.design.as_ref().and_then(|design| design.circuit.as_ref())
        .ok_or_else(|| ApiError::invalid("此项目是概念模板，不含管脚级电路，不能运行工程工具。请在 API 模式下新建低压 RC 低通工程。"))?;
    let tools = state
        .runtime
        .read()
        .map_err(ApiError::internal)?
        .tools
        .clone();
    let run = eda::build(spec, &project.constraints, &tools).await?;
    let updated = state
        .access(move |repository| {
            let mut current = repository.get(id)?;
            if current.revision != request.expected_revision {
                return Err(ApiError::conflict());
            }
            let mut design = current
                .design
                .take()
                .ok_or_else(|| ApiError::invalid("缺少设计"))?;
            design.engineering = Some(run);
            design.manufacturing_ready = false;
            current.apply_design(current.prompt.clone(), current.constraints.clone(), design);
            repository.save(&current)?;
            Ok(current)
        })
        .await?;
    Ok(Json(updated))
}

async fn download_artifact(
    State(state): State<AppState>,
    Path((id, index)): Path<(String, usize)>,
    axum::extract::Query(query): axum::extract::Query<ArtifactQuery>,
) -> Result<Response, ApiError> {
    let id = project_id(&id)?;
    let project = state
        .access(move |repository| Ok(repository.get(id)?))
        .await?;
    if query.expected_revision != project.revision {
        return Err(ApiError::conflict());
    }
    let run = project
        .design
        .as_ref()
        .and_then(|design| design.engineering.as_ref())
        .ok_or_else(|| ApiError::invalid("没有工程产物。"))?;
    let artifact = run
        .artifacts
        .get(index)
        .ok_or_else(|| ApiError::invalid("工程文件不存在。"))?;
    // Manufacturing release is deliberately closed until component qualification is implemented.
    if artifact.production {
        return Err(ApiError::invalid(
            "生产文件已锁定：具体料号、封装和电气额定值尚未完成工程签核。",
        ));
    }
    let name = artifact.name.rsplit('/').next().unwrap_or("artifact.txt");
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {
        return Err(ApiError::invalid("无效的产物名称。"));
    }
    if query.preview && !(artifact.name.starts_with("previews/") && name.ends_with(".svg")) {
        return Err(ApiError::invalid("只有 KiCad SVG 预览支持内嵌显示。"));
    }
    Ok((
        [
            (
                header::CONTENT_TYPE,
                if query.preview {
                    "image/svg+xml"
                } else {
                    "application/octet-stream"
                }
                .to_string(),
            ),
            (
                header::CONTENT_DISPOSITION,
                format!(
                    "{}; filename=\"{name}\"",
                    if query.preview {
                        "inline"
                    } else {
                        "attachment"
                    }
                ),
            ),
        ],
        artifact.content.clone(),
    )
        .into_response())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactQuery {
    expected_revision: u32,
    #[serde(default)]
    preview: bool,
}

async fn review_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Result<Json<ReviewRequest>, JsonRejection>,
) -> Result<Json<Project>, ApiError> {
    let id = project_id(&id)?;
    let request = payload(request)?;
    if !request.acknowledge_demo {
        return Err(ApiError::invalid(
            "请先明确确认：这只是未工程验证、不可投产的概念演示。",
        ));
    }
    let project = state
        .access(move |repository| {
            let mut project = repository.get(id)?;
            if project.revision != request.expected_revision {
                return Err(ApiError::conflict());
            }
            let design = project
                .design
                .as_ref()
                .ok_or_else(|| ApiError::invalid("请先生成概念方案。"))?;
            if design
                .checks
                .iter()
                .any(|check| check.status == CheckStatus::Blocked)
            {
                return Err(ApiError::invalid(
                    "存在阻断检查项，请修改需求或板尺寸后重新生成。",
                ));
            }
            project.approval = Some(Approval {
                revision: project.revision,
                reviewed_at: timestamp(),
                acknowledgment: true,
            });
            project.status = ProjectStatus::ConceptApproved;
            project.updated_at = timestamp();
            repository.save(&project)?;
            Ok(project)
        })
        .await?;
    Ok(Json(project))
}

async fn export_project(
    State(state): State<AppState>,
    Path((id, format)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let id = project_id(&id)?;
    let project = state
        .access(move |repository| Ok(repository.get(id)?))
        .await?;
    if format == "engineering" {
        let run = project
            .design
            .as_ref()
            .and_then(|design| design.engineering.as_ref())
            .ok_or_else(|| ApiError::invalid("此项目没有工程文件。"))?;
        let mut files = run
            .artifacts
            .iter()
            .filter(|artifact| !artifact.production && !artifact.name.starts_with("production/"))
            .map(|artifact| (artifact.name.clone(), artifact.content.clone()))
            .collect::<Vec<_>>();
        files.push(("README.md".into(), "# AI-PCB engineering source package\n\nOpen board.kicad_pro in KiCad 9. Keep AI_PCB.pretty, AI_PCB.kicad_sym and library tables beside it.\n\nThis is NOT a production release. Review ERC, DRC, simulation, actual component dimensions, ratings, source/load impedance and assembly before fabrication. Tool failures remain in logs; generated geometry is not proof of verification.\n".into()));
        files.push(("report.md".into(), export::report(&project)));
        let content = archive::zip(&files).map_err(ApiError::internal)?;
        return Ok((
            [
                (header::CONTENT_TYPE, "application/zip".to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    format!(
                        "attachment; filename=\"ai-pcb-engineering-{id}-v{}.zip\"",
                        project.revision
                    ),
                ),
            ],
            content,
        )
            .into_response());
    }
    let (content_type, extension, content) = match format.as_str() {
        "bom" => ("text/csv; charset=utf-8", "csv", export::bom_csv(&project)),
        "report" => (
            "text/markdown; charset=utf-8",
            "md",
            export::report(&project),
        ),
        "project" => (
            "application/json; charset=utf-8",
            "json",
            serde_json::to_string_pretty(&project).map_err(ApiError::internal)?,
        ),
        _ => {
            return Err(ApiError::invalid(
                "仅支持 bom、project、report 演示导出，不提供生产文件。",
            ));
        }
    };
    Ok((
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!(
                    "attachment; filename=\"ai-pcb-demo-{id}-v{}.{extension}\"",
                    project.revision
                ),
            ),
        ],
        content,
    )
        .into_response())
}
