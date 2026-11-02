use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub const DEMO_NOTICE: &str = "这是规则模板生成的概念演示，不是大模型设计结果。布局、网络与价格均为示例，未执行 ERC、DRC、仿真或可制造性验证，不可直接投产。";
pub const AI_NOTICE: &str = "需求分析来自配置的 Responses API 模型服务，可能有误，须人工核对；PCB 布局、BOM、网络和价格仍来自内置参考模板。未执行 ERC、DRC、仿真或可制造性验证，不可直接投产。";

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Constraints {
    pub width_mm: u16,
    pub height_mm: u16,
    pub layers: u8,
    pub budget_cny: u32,
}

impl Default for Constraints {
    fn default() -> Self {
        Self {
            width_mm: 80,
            height_mm: 50,
            layers: 2,
            budget_cny: 80,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Draft,
    PendingReview,
    ConceptApproved,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub schema_version: u32,
    pub id: Uuid,
    pub name: String,
    pub prompt: String,
    pub constraints: Constraints,
    pub revision: u32,
    pub status: ProjectStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub design: Option<Design>,
    pub approval: Option<Approval>,
    pub history: Vec<Revision>,
}

impl Project {
    pub fn new(name: String, prompt: String, constraints: Constraints) -> Self {
        let now = timestamp();
        Self {
            schema_version: 1,
            id: Uuid::new_v4(),
            name,
            prompt,
            constraints,
            revision: 0,
            status: ProjectStatus::Draft,
            created_at: now,
            updated_at: now,
            design: None,
            approval: None,
            history: Vec::new(),
        }
    }

    pub fn apply_design(&mut self, prompt: String, constraints: Constraints, design: Design) {
        self.revision += 1;
        self.updated_at = timestamp();
        self.prompt = prompt;
        self.constraints = constraints;
        self.approval = None;
        self.status = ProjectStatus::PendingReview;
        self.history.push(Revision {
            revision: self.revision,
            prompt: self.prompt.clone(),
            summary: design.summary.clone(),
            created_at: self.updated_at,
        });
        if self.history.len() > 30 {
            self.history.remove(0);
        }
        self.design = Some(design);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Revision {
    pub revision: u32,
    pub prompt: String,
    pub summary: String,
    pub created_at: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Approval {
    pub revision: u32,
    pub reviewed_at: u64,
    pub acknowledgment: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Design {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circuit: Option<crate::circuit::CircuitSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engineering: Option<crate::eda::EngineeringRun>,
    pub engine: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai: Option<AiAnalysis>,
    pub template_id: String,
    pub template_name: String,
    pub summary: String,
    pub notice: String,
    pub manufacturing_ready: bool,
    pub requirements: Vec<Requirement>,
    pub modules: Vec<DesignModule>,
    pub bom: Vec<BomItem>,
    pub footprints: Vec<Footprint>,
    pub nets: Vec<Net>,
    pub checks: Vec<DesignCheck>,
    pub estimated_cost_cents: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AiPlan {
    #[serde(default)]
    pub circuit: Option<crate::circuit::CircuitSpec>,
    pub template_id: String,
    pub summary: String,
    pub requirements: Vec<Requirement>,
    pub assumptions: Vec<String>,
    pub unimplemented_requirements: Vec<String>,
    pub recommendations: Vec<String>,
    pub safety_concerns: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AiAnalysis {
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_origin: Option<String>,
    pub model: String,
    pub response_id: String,
    pub plan: AiPlan,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DesignModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub references: Vec<String>,
    pub kind: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub package: String,
    pub unit_price_cents: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BomItem {
    pub part: Part,
    pub references: Vec<String>,
    pub quantity: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Footprint {
    pub reference: String,
    pub part_id: String,
    pub label: String,
    pub kind: String,
    pub x_mm: f64,
    pub y_mm: f64,
    pub width_mm: f64,
    pub height_mm: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Net {
    pub name: String,
    pub from_ref: String,
    pub to_ref: String,
    pub layer: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Warning,
    Pending,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DesignCheck {
    pub id: String,
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRequest {
    #[serde(default)]
    pub config_version: u64,
    pub name: String,
    pub prompt: String,
    #[serde(default)]
    pub constraints: Constraints,
    #[serde(default)]
    pub allow_remote: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateRequest {
    #[serde(default)]
    pub config_version: u64,
    pub prompt: String,
    pub constraints: Constraints,
    pub expected_revision: u32,
    #[serde(default)]
    pub allow_remote: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRequest {
    pub expected_revision: u32,
    pub acknowledge_demo: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteRequest {
    pub expected_revision: u32,
}
