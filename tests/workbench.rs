use ai_pcb::{AppState, engine::DemoEngine, repository::Repository, router, seed};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

fn setup() -> (tempfile::TempDir, Router) {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path()).unwrap();
    seed(&repository).unwrap();
    let app = router(AppState::new(repository, Arc::new(DemoEngine)));
    (dir, app)
}

async fn call(app: &Router, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("host", "localhost:3000")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn api(base_url: &str, key: &str) -> Value {
    json!({"provider":"openai", "base_url":base_url, "model":"test-model", "api_key":key, "timeout_seconds":10, "max_output_tokens":4000, "reasoning_effort":"", "stream":false, "output_format":"json_schema"})
}

#[tokio::test]
async fn settings_hot_swap_redaction_retention_and_versions() {
    let (_dir, app) = setup();
    let (_, initial) = call(&app, "GET", "/api/settings", json!(null)).await;
    assert_eq!(initial["config_version"], 0);
    assert_eq!(initial["key_configured"], false);
    let (status, saved) = call(
        &app,
        "PUT",
        "/api/settings",
        json!({"expected_version":0,"api":api("http://127.0.0.1:49123/v1", "TOP_SECRET_TEST_KEY")}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["generation"]["requires_remote_consent"], true);
    assert_eq!(saved["key_configured"], true);
    assert!(!saved.to_string().contains("TOP_SECRET_TEST_KEY"));
    assert!(saved["api"].get("api_key").is_none());
    let (_, health) = call(&app, "GET", "/api/health", json!(null)).await;
    assert_eq!(health["generation"]["model"], "test-model");
    assert!(!health.to_string().contains("TOP_SECRET"));
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/settings",
            json!({"expected_version":0,"api":api("http://127.0.0.1:49123/v1", "")})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/settings",
            json!({"expected_version":1,"api":api("http://localhost:49123/v1", "")})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, retained) = call(
        &app,
        "PUT",
        "/api/settings",
        json!({"expected_version":1,"api":api("http://127.0.0.1:49123/v1/", "")}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(retained["key_configured"], true);
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/settings/test",
            json!({"expected_version":2,"allow_remote":false})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, cleared) = call(
        &app,
        "DELETE",
        "/api/settings",
        json!({"expected_version":2,"allow_remote":false}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cleared["key_configured"], false);
    assert_eq!(cleared["generation"]["requires_remote_consent"], false);
}

#[tokio::test]
async fn rejects_cross_origin_and_dns_rebinding() {
    let (_dir, app) = setup();
    for (host, origin) in [
        ("localhost:3000", "https://attacker.test"),
        ("attacker.test:3000", "http://attacker.test:3000"),
        ("localhost:3000", "null"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/settings")
                    .header("host", host)
                    .header("origin", origin)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/settings")
                .header("host", "127.0.0.1:3000")
                .header("origin", "http://127.0.0.1:3000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
}

#[tokio::test]
async fn cross_site_links_only_allow_top_level_homepage_navigation() {
    let (_dir, app) = setup();
    for (method, path, mode, destination, expected) in [
        ("GET", "/", "navigate", "document", StatusCode::OK),
        ("GET", "/", "cors", "empty", StatusCode::FORBIDDEN),
        ("GET", "/", "navigate", "iframe", StatusCode::FORBIDDEN),
        ("POST", "/", "navigate", "document", StatusCode::FORBIDDEN),
        (
            "GET",
            "/api/settings",
            "navigate",
            "document",
            StatusCode::FORBIDDEN,
        ),
        (
            "POST",
            "/api/projects",
            "navigate",
            "document",
            StatusCode::FORBIDDEN,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("host", "127.0.0.1:3000")
                    .header("sec-fetch-site", "cross-site")
                    .header("sec-fetch-mode", mode)
                    .header("sec-fetch-dest", destination)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            expected,
            "{method} {path} {mode} {destination}"
        );
    }
    for (host, origin) in [
        ("attacker.test:3000", None),
        ("127.0.0.1:3000", Some("https://attacker.test")),
        ("127.0.0.1:3000", Some("null")),
    ] {
        let mut request = Request::builder()
            .uri("/")
            .header("host", host)
            .header("sec-fetch-site", "cross-site")
            .header("sec-fetch-mode", "navigate")
            .header("sec-fetch-dest", "document");
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn delete_requires_matching_revision_and_empty_stays_empty() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path()).unwrap();
    seed(&repository).unwrap();
    assert!(repository.list().unwrap().is_empty());
    let mut project = ai_pcb::model::Project::new(
        "owned-test".into(),
        "an environment sensing board".into(),
        Default::default(),
    );
    let design = DemoEngine.build(&project.prompt, &project.constraints, None);
    project.apply_design(project.prompt.clone(), project.constraints.clone(), design);
    repository.save(&project).unwrap();
    let app = router(AppState::new(repository, Arc::new(DemoEngine)));
    let path = format!("/api/projects/{}", project.id);
    assert_eq!(
        call(&app, "DELETE", &path, json!({"expected_revision":0}))
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(&app, "GET", &path, json!(null)).await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "DELETE", &path, json!({"expected_revision":1}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "GET", &path, json!(null)).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, "DELETE", &path, json!({"expected_revision":1}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    drop(app);
    let reopened = Repository::open(dir.path()).unwrap();
    seed(&reopened).unwrap();
    assert!(reopened.list().unwrap().is_empty());
}

#[tokio::test]
async fn configuration_rejects_insecure_or_confused_targets() {
    let (_dir, app) = setup();
    for endpoint in [
        "http://api.example.com/v1",
        "https://api.example.com/v1?api_key=secret",
        "https://name:secret@api.example.com/v1",
        "https://api.example.com/v1/responses",
    ] {
        assert_eq!(
            call(
                &app,
                "PUT",
                "/api/settings",
                json!({"expected_version":0,"api":api(endpoint, "secret")})
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    let (_, settings) = call(&app, "GET", "/api/settings", json!(null)).await;
    assert_eq!(settings["config_version"], 0);
}

#[tokio::test]
async fn compatible_api_supports_custom_targets_without_provider_presets() {
    let (_directory, app) = setup();
    let (status, saved) = call(
        &app,
        "PUT",
        "/api/settings",
        json!({"expected_version":0,"api":api("https://gateway.example.com/v1", "TEST_ONLY_FIRST_TARGET_KEY")}),
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["api"]["provider"], "openai");
    assert_eq!(saved["generation"]["provider_label"], "OpenAI 兼容 API");
    assert_eq!(
        saved["generation"]["remote_origin"],
        "https://gateway.example.com"
    );
    assert_eq!(saved["generation"]["api_protocol"], "responses");
    assert_eq!(saved["verified"], false);
    let (status, _) = call(
        &app,
        "PUT",
        "/api/settings",
        json!({"expected_version":1,"api":api("https://api.openai.com/v1", "")}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (_, unchanged) = call(&app, "GET", "/api/settings", json!(null)).await;
    assert_eq!(unchanged, saved);
    let (status, saved) = call(
        &app,
        "PUT",
        "/api/settings",
        json!({"expected_version":1,"api":api("https://api.openai.com/v1", "TEST_ONLY_SECOND_TARGET_KEY")}),
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["generation"]["provider_label"], "OpenAI 兼容 API");
    assert_eq!(
        saved["generation"]["remote_origin"],
        "https://api.openai.com"
    );
    assert_eq!(saved["config_version"], 2);
    assert!(!saved.to_string().contains("TEST_ONLY_SECOND_TARGET_KEY"));
    let mut unsupported = api("https://api.openai.com/v1", "TEST_ONLY_SECOND_TARGET_KEY");
    unsupported["provider"] = json!("unrecognized-provider");
    let (status, _) = call(
        &app,
        "PUT",
        "/api/settings",
        json!({"expected_version":2,"api":unsupported}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (_, unchanged) = call(&app, "GET", "/api/settings", json!(null)).await;
    assert_eq!(unchanged, saved);
}

#[tokio::test]
async fn supplier_credentials_do_not_leak_or_trigger_queries_implicitly() {
    let (_dir, app) = setup();
    let (status, settings) = call(
        &app,
        "PUT",
        "/api/settings/supplier",
        json!({"expected_version":0,"api_key":"supplier-secret"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(settings["supplier_configured"], true);
    assert!(!settings.to_string().contains("supplier-secret"));
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/components/search",
            json!({"keyword":"resistor", "allow_remote":false})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "PUT",
            "/api/settings/supplier",
            json!({"expected_version":1,"api_key":""})
        )
        .await
        .1["supplier_configured"],
        false
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/components/search",
            json!({"keyword":"resistor", "allow_remote":true})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn artifact_and_export_production_gate_cannot_be_bypassed() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path()).unwrap();
    let mut project = ai_pcb::model::Project::new(
        "production gate".into(),
        "passive rc low pass".into(),
        Default::default(),
    );
    let spec = ai_pcb::circuit::CircuitSpec {
        kind: "rc_lowpass".into(),
        resistance_ohm: 10000.0,
        capacitance_nf: 100.0,
        max_input_v: 5.0,
    };
    let mut design = ai_pcb::circuit::design(&spec, &project.constraints);
    design.engineering = Some(ai_pcb::eda::EngineeringRun {
        verified: true,
        finished_at: 1,
        released_at: None,
        steps: vec![],
        artifacts: vec![
            ai_pcb::eda::Artifact {
                name: "production/board.gbr".into(),
                production: true,
                content: String::new(),
            },
            ai_pcb::eda::Artifact {
                name: "board.kicad_sch".into(),
                production: false,
                content: "source".into(),
            },
        ],
    });
    project.apply_design(project.prompt.clone(), project.constraints.clone(), design);
    repository.save(&project).unwrap();
    let app = router(AppState::new(repository, Arc::new(DemoEngine)));
    assert_eq!(
        call(
            &app,
            "GET",
            &format!(
                "/api/projects/{}/artifacts/0?expected_revision=1",
                project.id
            ),
            json!(null)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "GET",
            &format!(
                "/api/projects/{}/artifacts/1?expected_revision=0",
                project.id
            ),
            json!(null)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("/api/projects/{}/export/gerber", project.id),
            json!(null)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (_, export) = call(
        &app,
        "GET",
        &format!("/api/projects/{}/export/project", project.id),
        json!(null),
    )
    .await;
    assert_eq!(export["design"]["manufacturing_ready"], false);
}

#[tokio::test]
async fn upstream_errors_are_actionable_private_and_never_retried() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let secret = "PRIVATE_TEST_KEY_MUST_NOT_LEAK";
    let private_prompt = "PRIVATE_REQUIREMENTS_MUST_NOT_LEAK";
    let cases = [
        (404, json!({"error":{"code":"model_not_found","param":"model","message":secret}}).to_string(), "模型或通道不可用"),
        (400, json!({"error":{"code":"unsupported_value","param":"reasoning.effort","message":secret}}).to_string(), "推理强度"),
        (400, json!({"error":{"code":"invalid_json_schema","param":"text.format.schema","message":secret}}).to_string(), "Text JSON"),
        (422, json!({"error":{"param":"max_output_tokens","message":secret}}).to_string(), "输出上限"),
        (400, json!({"error":{"param":"stream","message":secret}}).to_string(), "流式参数"),
        (400, json!({"error":{"message":format!("No available channel: {secret}")}}).to_string(), "模型或通道不可用"),
        (403, json!({"error":{"message":secret}}).to_string(), "账户和模型权限"),
        (429, json!({"error":{"code":"insufficient_quota","message":secret}}).to_string(), "额度和限流"),
        (404, format!("<html>{secret}</html>"), "/v1/responses"),
        (400, json!({"error":{"code":secret,"param":private_prompt,"message":secret},"request":private_prompt}).to_string(), "未提供可识别"),
        (400, json!({"error":{"code":"model_not_found","message":"x".repeat(33 * 1024)}}).to_string(), "未提供可识别"),
        (503, json!({"error":{"message":secret}}).to_string(), "上游未完成请求"),
    ];
    for (upstream_status, response_body, expected_hint) in cases {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let attempts = Arc::new(AtomicUsize::new(0));
        let captured_attempts = attempts.clone();
        let mock = Router::new().route(
            "/v1/responses",
            axum::routing::post(move |axum::Json(body): axum::Json<Value>| {
                captured_attempts.fetch_add(1, Ordering::SeqCst);
                assert_eq!(body["text"]["format"]["type"], "text");
                assert!(body.get("reasoning").is_none());
                assert!(body.get("max_output_tokens").is_none());
                let response_body = response_body.clone();
                async move {
                    (
                        StatusCode::from_u16(upstream_status).unwrap(),
                        response_body,
                    )
                }
            }),
        );
        let task = tokio::spawn(async move {
            axum::serve(listener, mock).await.unwrap();
        });
        let (_directory, app) = setup();
        let mut configuration = api(&format!("http://{address}/v1"), secret);
        configuration["output_format"] = json!("text");
        configuration["max_output_tokens"] = json!(0);
        let (status, _) = call(
            &app,
            "PUT",
            "/api/settings",
            json!({"expected_version":0,"api":configuration}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(attempts.load(Ordering::SeqCst), 0);
        let (status, _) = call(
            &app,
            "POST",
            "/api/settings/test",
            json!({"expected_version":1,"allow_remote":false}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(attempts.load(Ordering::SeqCst), 0);
        let (status, result) = call(
            &app,
            "POST",
            "/api/settings/test",
            json!({"expected_version":1,"allow_remote":true}),
        )
        .await;
        assert_eq!(
            status,
            if upstream_status == 429 {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::BAD_GATEWAY
            }
        );
        let message = result["error"].as_str().unwrap();
        assert!(
            message.contains(&format!("HTTP {upstream_status}")),
            "{message}"
        );
        assert!(message.contains(expected_hint), "{message}");
        assert!(!message.contains(secret));
        assert!(!message.contains(private_prompt));
        assert!(message.len() < 2000);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        let (_, settings) = call(&app, "GET", "/api/settings", json!(null)).await;
        assert_eq!(settings["verified"], false);
        assert_eq!(settings["api"]["output_format"], "text");
        assert_eq!(settings["config_version"], 1);
        assert!(!settings.to_string().contains(secret));
        let (_, projects) = call(&app, "GET", "/api/projects", json!(null)).await;
        assert_eq!(projects, json!([]));
        task.abort();
        let _ = task.await;
    }
}

#[tokio::test]
async fn mock_responses_rc_blockers_are_specific_atomic_and_never_retried() {
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    for (has_unimplemented, has_safety) in [(true, false), (false, true), (true, true)] {
        let unimplemented = if has_unimplemented {
            vec![
                "未实现隔离接口".to_string(),
                "缺".repeat(200),
                "未实现第三项需求".to_string(),
                "第四项未实现原因不应展示".to_string(),
            ]
        } else {
            vec![]
        };
        let safety = if has_safety {
            vec![
                "输入电压存在安全疑点".to_string(),
                "险".repeat(200),
                "第三项安全疑点".to_string(),
                "第四项安全原因不应展示".to_string(),
            ]
        } else {
            vec![]
        };
        let plan = json!({
            "template_id":"rc_lowpass",
            "circuit":{"kind":"rc_lowpass","resistance_ohm":10000,"capacitance_nf":100,"max_input_v":5},
            "summary":"RC 请求被模型报告的具体问题阻断",
            "requirements":[{"label":"filter","value":"lowpass"}],
            "assumptions":[], "recommendations":[],
            "unimplemented_requirements":unimplemented, "safety_concerns":safety
        });
        let attempts = Arc::new(AtomicUsize::new(0));
        let captured_attempts = attempts.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let mock = Router::new().route(
            "/v1/responses",
            axum::routing::post(move |axum::Json(body): axum::Json<Value>| {
                captured_attempts.fetch_add(1, Ordering::SeqCst);
                assert_eq!(body["store"], false);
                let plan = plan.clone();
                async move {
                    axum::Json(json!({
                        "id":"response-local-blockers", "model":"test-model", "status":"completed",
                        "output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":plan.to_string()}]}]
                    }))
                }
            }),
        );
        let task = tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
        let (directory, app) = setup();
        let (status, original) = call(
            &app, "POST", "/api/projects",
            json!({"name":"Existing local concept must survive","prompt":"An environment sensing board"}),
        ).await;
        assert_eq!(status, StatusCode::CREATED);
        let project_path = format!("/api/projects/{}", original["id"].as_str().unwrap());
        let (status, original_list) = call(&app, "GET", "/api/projects", json!(null)).await;
        assert_eq!(status, StatusCode::OK);
        let snapshot_files = || -> BTreeMap<_, _> {
            std::fs::read_dir(directory.path())
                .unwrap()
                .filter(|entry| entry.as_ref().unwrap().file_name() != ".lock")
                .map(|entry| {
                    let entry = entry.unwrap();
                    (entry.file_name(), std::fs::read(entry.path()).unwrap())
                })
                .collect()
        };
        let original_files = snapshot_files();
        let (status, _) = call(
            &app, "PUT", "/api/settings",
            json!({"expected_version":0,"api":api(&format!("http://{address}/v1"), "local-blocker-test-key")}),
        ).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(attempts.load(Ordering::SeqCst), 0);

        for (request_index, (path, body)) in [
            (
                "/api/projects".to_string(),
                json!({"name":"Blocked RC creation","prompt":"Design a passive RC lowpass at 5V","config_version":1,"allow_remote":true}),
            ),
            (
                format!("{project_path}/generate"),
                json!({"prompt":"Replace with a passive RC lowpass at 5V","constraints":original["constraints"],"expected_revision":original["revision"],"config_version":1,"allow_remote":true}),
            ),
        ].into_iter().enumerate() {
            let (status, response) = call(&app, "POST", &path, body).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            let message = response["error"].as_str().unwrap();
            assert!(message.starts_with("需求包含未实现功能或安全疑点，已阻止生成不完整工程。"), "{message}");
            for (present, category, reason, repeated, third, fourth) in [
                (has_unimplemented, "模型报告的未实现需求", "未实现隔离接口", "缺", "未实现第三项需求", "第四项未实现原因不应展示"),
                (has_safety, "模型报告的安全疑点", "输入电压存在安全疑点", "险", "第三项安全疑点", "第四项安全原因不应展示"),
            ] {
                assert_eq!(message.contains(category), present, "{message}");
                assert_eq!(message.contains(reason), present, "{message}");
                assert_eq!(message.contains(third), present, "{message}");
                if present {
                    assert!(message.contains(&repeated.repeat(150)), "{message}");
                }
                assert!(!message.contains(&repeated.repeat(161)), "{message}");
                assert!(!message.contains(fourth), "{message}");
            }
            assert!(!message.contains("local-blocker-test-key"));
            assert_eq!(attempts.load(Ordering::SeqCst), request_index + 1);
            let (status, current) = call(&app, "GET", &project_path, json!(null)).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(current, original);
            let (status, current_list) = call(&app, "GET", "/api/projects", json!(null)).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(current_list, original_list);
            assert_eq!(snapshot_files(), original_files);
        }
        task.abort();
        let _ = task.await;
    }
}

#[tokio::test]
async fn mock_responses_multiblock_json_and_sse_require_one_complete_plan() {
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let plan = json!({
        "template_id":"rc_lowpass",
        "circuit":{"kind":"rc_lowpass","resistance_ohm":22000,"capacitance_nf":47,"max_input_v":5},
        "summary":"PRIVATE_MOCK_PLAN_BODY",
        "requirements":[{"label":"filter","value":"lowpass"}],
        "assumptions":[], "recommendations":[], "unimplemented_requirements":[], "safety_concerns":[]
    });
    let text = plan.to_string();
    let split = text.find("PRIVATE_MOCK_PLAN_BODY").unwrap() + 7;
    let parts = vec![
        json!({"type":"output_text","text":&text[..split]}),
        json!({"type":"output_text","text":&text[split..]}),
    ];
    let message =
        |content: Vec<Value>| json!({"type":"message","role":"assistant","content":content});
    let duplicate = vec![
        json!({"type":"output_text","text":text}),
        json!({"type":"output_text","text":text}),
    ];
    let cases = [
        (
            "json_split_content",
            false,
            vec![message(parts.clone())],
            true,
            true,
        ),
        (
            "json_split_messages",
            false,
            vec![
                message(vec![parts[0].clone()]),
                message(vec![parts[1].clone()]),
            ],
            true,
            true,
        ),
        (
            "json_duplicate",
            false,
            vec![message(duplicate.clone())],
            true,
            false,
        ),
        (
            "json_empty_text",
            false,
            vec![message(vec![json!({"type":"output_text","text":""})])],
            true,
            false,
        ),
        ("json_no_output", false, vec![], true, false),
        ("sse_split_content", true, vec![message(parts)], true, true),
        ("sse_duplicate", true, vec![message(duplicate)], true, false),
        (
            "sse_empty_completed",
            true,
            vec![message(vec![])],
            true,
            false,
        ),
        ("sse_delta_only", true, vec![], false, false),
    ];
    for (case_name, streaming, output, completed, succeeds) in cases {
        let response = json!({"id":"response-local-multiblock","model":"test-model","status":"completed","output":output});
        let response_body = if streaming {
            let delta_plan = if succeeds {
                json!({"summary":"PRIVATE_DELTA_MUST_NOT_OVERRIDE_COMPLETED"}).to_string()
            } else {
                text.clone()
            };
            let delta = json!({"type":"response.output_text.delta","output_index":0,"content_index":0,"delta":delta_plan});
            let mut events = format!("event: response.output_text.delta\ndata: {delta}\n\n");
            if completed {
                let event = json!({"type":"response.completed","response":response});
                events.push_str(&format!("event: response.completed\ndata: {event}\n\n"));
            }
            events.push_str("data: [DONE]\n\n");
            events
        } else {
            response.to_string()
        };
        let attempts = Arc::new(AtomicUsize::new(0));
        let captured_attempts = attempts.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let mock = Router::new().route(
            "/v1/responses",
            axum::routing::post(move |axum::Json(body): axum::Json<Value>| {
                captured_attempts.fetch_add(1, Ordering::SeqCst);
                assert_eq!(body["store"], false);
                assert_eq!(body["stream"], streaming);
                let response_body = response_body.clone();
                async move {
                    (
                        [(
                            "content-type",
                            if streaming {
                                "text/event-stream"
                            } else {
                                "application/json"
                            },
                        )],
                        response_body,
                    )
                }
            }),
        );
        let task = tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
        let (directory, app) = setup();
        let (status, original) = call(
            &app,
            "POST",
            "/api/projects",
            json!({"name":"Existing parser test concept","prompt":"An environment sensing board"}),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let project_path = format!("/api/projects/{}", original["id"].as_str().unwrap());
        let (status, original_list) = call(&app, "GET", "/api/projects", json!(null)).await;
        assert_eq!(status, StatusCode::OK);
        let snapshot_files = || -> BTreeMap<_, _> {
            std::fs::read_dir(directory.path())
                .unwrap()
                .map(|entry| entry.unwrap())
                .filter(|entry| entry.file_name() != ".lock")
                .map(|entry| (entry.file_name(), std::fs::read(entry.path()).unwrap()))
                .collect()
        };
        let original_files = snapshot_files();
        let (status, _) = call(
            &app, "PUT", "/api/settings/tools",
            json!({"expected_version":0,"tools":{"kicad_cli":"","kicad_python":"","ngspice":"","java":"","freerouting_jar":""}}),
        ).await;
        assert_eq!(status, StatusCode::OK);
        let mut configuration = api(&format!("http://{address}/v1"), "local-parser-test-key");
        configuration["stream"] = json!(streaming);
        let (status, _) = call(
            &app,
            "PUT",
            "/api/settings",
            json!({"expected_version":1,"api":configuration}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(attempts.load(Ordering::SeqCst), 0);
        let (status, result) = call(
            &app, "POST", "/api/projects",
            json!({"name":case_name,"prompt":"Design a passive RC lowpass at 5V","config_version":2,"allow_remote":true}),
        ).await;
        assert_eq!(attempts.load(Ordering::SeqCst), 1, "{case_name}");
        if succeeds {
            assert_eq!(status, StatusCode::CREATED, "{case_name}: {result}");
            assert_eq!(result["design"]["summary"], "PRIVATE_MOCK_PLAN_BODY");
            assert_eq!(result["design"]["circuit"]["resistance_ohm"], 22000.0);
            assert_eq!(result["design"]["circuit"]["capacitance_nf"], 47.0);
            assert_eq!(result["design"]["manufacturing_ready"], false);
            assert_eq!(result["design"]["engineering"]["verified"], false);
        } else {
            for (request_index, (status, result)) in [
                (status, result),
                call(
                    &app, "POST", &format!("{project_path}/generate"),
                    json!({"prompt":"Replace with a passive RC lowpass at 5V","constraints":original["constraints"],"expected_revision":original["revision"],"config_version":2,"allow_remote":true}),
                ).await,
            ].into_iter().enumerate() {
                assert_eq!(status, StatusCode::BAD_GATEWAY, "{case_name}, request {request_index}: {result}");
                let error = result["error"].as_str().unwrap();
                assert!(!error.is_empty());
                if case_name.ends_with("duplicate") {
                    assert!(error.contains("模型方案 JSON 校验失败"), "{case_name}: {error}");
                    assert!(error.contains("output_text 块=2"), "{case_name}: {error}");
                } else if completed {
                    assert!(error.contains("没有可解析的方案正文"), "{case_name}: {error}");
                    assert!(error.contains("正文字符=0"), "{case_name}: {error}");
                } else {
                    assert!(error.contains("有效的完整 Responses 响应"), "{case_name}: {error}");
                }
                assert!(!error.contains("PRIVATE_MOCK_PLAN_BODY"));
                assert!(!error.contains("PRIVATE_DELTA_MUST_NOT_OVERRIDE_COMPLETED"));
                assert!(!error.contains("local-parser-test-key"));
            }
            assert_eq!(attempts.load(Ordering::SeqCst), 2, "{case_name}");
            let (status, current_list) = call(&app, "GET", "/api/projects", json!(null)).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(current_list, original_list, "{case_name}");
            assert_eq!(snapshot_files(), original_files, "{case_name}");
        }
        let (status, current) = call(&app, "GET", &project_path, json!(null)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(current, original, "{case_name}");
        task.abort();
        let _ = task.await;
    }
}

#[tokio::test]
async fn mock_responses_generates_engineering_sources_without_faking_checks() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let mock = Router::new().route("/v1/responses", axum::routing::post(|axum::Json(body): axum::Json<Value>| async move {
        assert_eq!(body["store"], false);
        assert!(body["text"]["format"]["schema"]["properties"].get("circuit").is_some());
        let plan = json!({"template_id":"rc_lowpass", "circuit":{"kind":"rc_lowpass","resistance_ohm":10000,"capacitance_nf":100,"max_input_v":5}, "summary":"RC 工程待工具验证", "requirements":[{"label":"filter","value":"lowpass"}], "assumptions":[], "unimplemented_requirements":[], "recommendations":[], "safety_concerns":[]});
        axum::Json(json!({"id":"response-local-test", "model":"test-model", "status":"completed", "output":[{"type":"message", "role":"assistant", "content":[{"type":"output_text", "text":plan.to_string()}]}]}))
    }));
    let task = tokio::spawn(async move {
        axum::serve(listener, mock).await.unwrap();
    });
    let (_dir, app) = setup();
    let (status, _) = call(
        &app,
        "PUT",
        "/api/settings",
        json!({"expected_version":0,"api":api(&format!("http://{address}/v1"), "test-key")}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, settings) = call(
        &app,
        "POST",
        "/api/settings/test",
        json!({"expected_version":1,"allow_remote":true}),
    )
    .await;
    assert_eq!(settings["verified"], true);
    let (status, project) = call(&app, "POST", "/api/projects", json!({"name":"RC acceptance", "prompt":"Design a passive RC lowpass at 5V", "allow_remote":true,"config_version":1})).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(project["design"]["circuit"]["kind"], "rc_lowpass");
    assert_eq!(
        project["design"]["engine"],
        "openai-responses+validated-circuit-compiler-v1"
    );
    assert_eq!(project["design"]["engineering"]["verified"], false);
    assert_eq!(project["design"]["manufacturing_ready"], false);
    assert!(
        project["design"]["engineering"]["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["name"] == "board.kicad_pcb")
    );
    task.abort();
    let _ = task.await;
}
