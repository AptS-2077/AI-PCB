use crate::model::{CheckStatus, DEMO_NOTICE, Project};

fn csv_cell(value: &str) -> String {
    let cleaned = value.replace('"', "\"\"");
    let prefix = if value
        .trim_start()
        .starts_with(['=', '+', '-', '@', '\t', '\r'])
    {
        "'"
    } else {
        ""
    };
    format!("\"{prefix}{cleaned}\"")
}

pub fn bom_csv(project: &Project) -> String {
    let mut lines = vec!["\u{feff}位号,型号,封装,数量,示例单价_CNY,示例小计_CNY,说明".to_string()];
    if let Some(design) = &project.design {
        for item in &design.bom {
            lines.push(
                [
                    csv_cell(&item.references.join(" / ")),
                    csv_cell(&item.part.name),
                    csv_cell(&item.part.package),
                    item.quantity.to_string(),
                    format!("{:.2}", f64::from(item.part.unit_price_cents) / 100.0),
                    format!(
                        "{:.2}",
                        f64::from(item.part.unit_price_cents * item.quantity) / 100.0
                    ),
                    csv_cell("DEMO ONLY · 未工程验证，不可投产；价格为内置估值"),
                ]
                .join(","),
            );
        }
    }
    lines.join("\r\n") + "\r\n"
}

fn markdown_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('`', "\\`")
}

pub fn report(project: &Project) -> String {
    let notice = project
        .design
        .as_ref()
        .map(|design| design.notice.as_str())
        .unwrap_or(DEMO_NOTICE);
    let mut text = format!(
        "# AI-PCB 概念方案报告\n\n> {notice}\n\n- 项目：{}\n- ID：{}\n- 版本：v{}\n- 制造就绪：**否**\n- 板框意向：{} × {} mm / {} 层\n\n## 需求原文\n\n{}\n\n",
        markdown_text(&project.name),
        project.id,
        project.revision,
        project.constraints.width_mm,
        project.constraints.height_mm,
        project.constraints.layers,
        markdown_text(&project.prompt)
    );
    if let Some(design) = &project.design {
        text.push_str(&format!(
            "## 参考方案\n\n{}\n\n内置 BOM 估值：¥{:.2}，不含 PCB 制造、组装、运费或外部负载。\n\n",
            markdown_text(&design.summary),
            f64::from(design.estimated_cost_cents) / 100.0
        ));
        if let Some(analysis) = &design.ai {
            text.push_str(&format!("## AI 分析来源\n\n- 接入方：{}\n- API 目标：{}\n- 模型：{}\n- 响应标识：{}\n- 布局来源：内置 {} 参考模板，非模型自动布线\n\n### 模型解读（需人工核对）\n\n", markdown_text(&analysis.provider), markdown_text(analysis.remote_origin.as_deref().unwrap_or("旧版本未记录")), markdown_text(&analysis.model), markdown_text(&analysis.response_id), design.template_id));
            for requirement in &analysis.plan.requirements {
                text.push_str(&format!(
                    "- {}：{}\n",
                    markdown_text(&requirement.label),
                    markdown_text(&requirement.value)
                ));
            }
            for (title, items) in [
                ("设计假设", &analysis.plan.assumptions),
                ("未实现需求", &analysis.plan.unimplemented_requirements),
                ("待验证建议", &analysis.plan.recommendations),
                ("安全疑点", &analysis.plan.safety_concerns),
            ] {
                if !items.is_empty() {
                    text.push_str(&format!("\n### {title}\n\n"));
                    for item in items {
                        text.push_str(&format!("- {}\n", markdown_text(item)));
                    }
                }
            }
        }
        text.push_str("\n## 检查记录\n\n");
        for check in &design.checks {
            let status = match check.status {
                CheckStatus::Passed => "演示检查通过",
                CheckStatus::Warning => "注意",
                CheckStatus::Pending => "未执行",
                CheckStatus::Blocked => "阻断",
            };
            text.push_str(&format!(
                "- **{} · {}**：{}\n",
                markdown_text(&check.name),
                status,
                markdown_text(&check.detail)
            ));
        }
    }
    text.push_str(if project.approval.is_some() {
        "\n## 审核\n\n用户已确认本版本的演示概念方案。该确认不代表工程验证通过，不允许直接制造。\n"
    } else {
        "\n## 审核\n\n尚未确认概念方案。\n"
    });
    text
}
