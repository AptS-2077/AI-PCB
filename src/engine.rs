use crate::model::*;
use std::collections::HashSet;
use std::{future::Future, pin::Pin};

#[derive(Clone, serde::Serialize)]
pub struct EngineInfo {
    pub provider: &'static str,
    pub provider_label: &'static str,
    pub model: Option<String>,
    pub remote_origin: Option<String>,
    pub api_protocol: &'static str,
    pub reasoning_effort: Option<String>,
    pub requires_remote_consent: bool,
    pub timeout_seconds: u64,
}

#[derive(Debug)]
pub struct EngineError {
    pub status: u16,
    pub message: String,
}

pub type EngineFuture<'a> = Pin<Box<dyn Future<Output = Result<Design, EngineError>> + Send + 'a>>;

pub trait DesignEngine: Send + Sync {
    fn configuration(&self) -> Option<crate::openai::ApiConfig> {
        None
    }
    fn info(&self) -> EngineInfo;
    fn generate<'a>(&'a self, prompt: &'a str, constraints: &'a Constraints) -> EngineFuture<'a>;
}

pub struct DemoEngine;

pub fn templates() -> Vec<Template> {
    [
        ("environment", "环境感知终端", "感知温湿度与光照，让环境数据触手可及。", "设计一块桌面环境监测电路板，使用 ESP32 采集温湿度和光照，通过 Wi-Fi 上传数据，USB 5V 供电，并预留扩展接口。", "sun"),
        ("irrigation", "智能浇灌控制器", "从土壤感知到水泵控制，照顾每一株绿植。", "我想做一块能检测土壤湿度、自动控制水泵，并能通过手机远程查看数据的电路板。逻辑部分使用 USB 5V 供电。", "leaf"),
        ("motion", "双路电机控制板", "连接驱动与控制，为你的下一个创意赋能。", "设计一块 ESP32 双路直流电机控制板，支持 PWM 调速、无线控制和状态指示，逻辑部分采用 USB 5V 供电。", "motor"),
    ]
    .into_iter()
    .map(|(id, name, description, prompt, icon)| Template {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        prompt: prompt.into(),
        icon: icon.into(),
    })
    .collect()
}

pub fn catalog() -> Vec<Part> {
    [
        (
            "esp32",
            "ESP32-C3-MINI-1",
            "无线主控模块参考项",
            "主控",
            "Module",
            1280,
        ),
        (
            "usb",
            "USB Type-C 16P",
            "供电连接器；CC 等外围待工程设计",
            "连接器",
            "USB-C",
            75,
        ),
        (
            "ldo",
            "AP2112K-3.3",
            "3.3 V 稳压参考项；热设计未验证",
            "电源",
            "SOT-25",
            48,
        ),
        ("r10k", "10 kΩ", "通用电阻示例", "无源器件", "0603", 3),
        ("r4k7", "4.7 kΩ", "总线上拉电阻示例", "无源器件", "0603", 3),
        ("r330", "330 Ω", "指示灯限流电阻示例", "无源器件", "0603", 3),
        ("c100n", "100 nF", "去耦电容示例", "无源器件", "0603", 2),
        ("c10u", "10 μF", "滤波电容示例", "无源器件", "0805", 12),
        ("led", "Green LED", "工作状态指示灯示例", "指示", "0603", 8),
        (
            "header",
            "GPIO Header 1×08",
            "扩展排针，管脚映射未定义",
            "连接器",
            "2.54 mm",
            35,
        ),
        (
            "button",
            "Tactile Switch",
            "功能按键参考项",
            "开关",
            "SMD",
            15,
        ),
        (
            "sht30",
            "SHT30-DIS",
            "温湿度传感器参考项",
            "传感器",
            "DFN-8",
            650,
        ),
        (
            "bh1750",
            "BH1750FVI",
            "光照传感器参考项",
            "传感器",
            "WSOF-6",
            230,
        ),
        (
            "soil",
            "Sensor Header 1×03",
            "外接土壤湿度探头接口，不含探头",
            "连接器",
            "2.54 mm",
            25,
        ),
        (
            "mosfet",
            "AO3400A",
            "低侧开关参考项；电流能力未验证",
            "驱动",
            "SOT-23",
            18,
        ),
        ("diode", "SS14", "续流二极管参考项", "保护", "SMA", 10),
        (
            "terminal",
            "Terminal 2P",
            "外部负载端子，不含电机或水泵",
            "连接器",
            "3.5 mm",
            65,
        ),
        (
            "driver",
            "DRV8833",
            "双路电机驱动参考项；供电与散热待验证",
            "驱动",
            "HTSSOP-16",
            260,
        ),
    ]
    .into_iter()
    .map(|(id, name, description, category, package, price)| Part {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        category: category.into(),
        package: package.into(),
        unit_price_cents: price,
    })
    .collect()
}

struct Placement<'a> {
    reference: &'a str,
    part: &'a str,
    kind: &'a str,
    position: (f64, f64),
    size: (f64, f64),
}

impl<'a> Placement<'a> {
    fn new(
        reference: &'a str,
        part: &'a str,
        kind: &'a str,
        position: (f64, f64),
        size: (f64, f64),
    ) -> Self {
        Self {
            reference,
            part,
            kind,
            position,
            size,
        }
    }
}

fn check(id: &str, name: &str, status: CheckStatus, detail: impl Into<String>) -> DesignCheck {
    DesignCheck {
        id: id.into(),
        name: name.into(),
        status,
        detail: detail.into(),
    }
}

impl DesignEngine for DemoEngine {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            provider: "demo",
            provider_label: "本地模板",
            model: None,
            remote_origin: None,
            api_protocol: "local",
            reasoning_effort: None,
            requires_remote_consent: false,
            timeout_seconds: 0,
        }
    }

    fn generate<'a>(&'a self, prompt: &'a str, constraints: &'a Constraints) -> EngineFuture<'a> {
        Box::pin(async move { Ok(self.build(prompt, constraints, None)) })
    }
}

impl DemoEngine {
    pub fn build(
        &self,
        prompt: &str,
        constraints: &Constraints,
        template_override: Option<&str>,
    ) -> Design {
        let normalized = prompt.to_lowercase();
        let matches =
            |keywords: &[&str]| keywords.iter().any(|keyword| normalized.contains(keyword));
        let irrigation = matches(&["土壤", "浇灌", "浇水", "水泵", "irrigat", "soil", "pump"]);
        let motion = matches(&["电机", "马达", "motor", "pwm", "motion"]);
        let environment = matches(&[
            "温度",
            "湿度",
            "温湿度",
            "环境",
            "光照",
            "sensor",
            "temperature",
            "humidity",
        ]);
        let template_index = if let Some(template_id) = template_override {
            match template_id {
                "irrigation" => 1,
                "motion" => 2,
                _ => 0,
            }
        } else if irrigation {
            1
        } else if motion {
            2
        } else {
            0
        };
        let template = templates().remove(template_index);
        let parts = catalog();
        let mut placements = vec![
            Placement::new("M1", "esp32", "module", (0.50, 0.42), (16.6, 20.0)),
            Placement::new("J1", "usb", "usb", (0.065, 0.40), (8.0, 9.0)),
            Placement::new("U1", "ldo", "ic", (0.25, 0.28), (4.5, 4.5)),
            Placement::new("R1", "r10k", "resistor", (0.32, 0.44), (3.0, 1.6)),
            Placement::new("R2", "r10k", "resistor", (0.32, 0.55), (3.0, 1.6)),
            Placement::new("R3", "r330", "resistor", (0.76, 0.16), (3.0, 1.6)),
            Placement::new("C1", "c100n", "capacitor", (0.17, 0.27), (3.0, 1.6)),
            Placement::new("C2", "c100n", "capacitor", (0.32, 0.30), (3.0, 1.6)),
            Placement::new("C3", "c100n", "capacitor", (0.64, 0.29), (3.0, 1.6)),
            Placement::new("C4", "c10u", "capacitor", (0.16, 0.57), (3.5, 2.0)),
            Placement::new("D1", "led", "led", (0.86, 0.16), (3.0, 2.0)),
            Placement::new("J2", "header", "header", (0.48, 0.86), (21.0, 4.0)),
            Placement::new("SW1", "button", "button", (0.15, 0.79), (6.0, 6.0)),
        ];
        let (peripheral_name, peripheral_description, peripheral_refs) = match template.id.as_str()
        {
            "irrigation" => {
                placements.extend([
                    Placement::new("J3", "soil", "header", (0.84, 0.36), (8.0, 4.0)),
                    Placement::new("Q1", "mosfet", "ic", (0.74, 0.59), (4.5, 4.5)),
                    Placement::new("D2", "diode", "diode", (0.87, 0.59), (4.0, 2.5)),
                    Placement::new("J4", "terminal", "terminal", (0.81, 0.82), (10.0, 7.0)),
                ]);
                (
                    "感知与执行",
                    "土壤探头接口与水泵开关；独立负载电源待设计",
                    vec!["J3", "Q1", "D2", "J4"],
                )
            }
            "motion" => {
                placements.extend([
                    Placement::new("U2", "driver", "ic", (0.79, 0.47), (7.0, 7.0)),
                    Placement::new("J3", "terminal", "terminal", (0.76, 0.82), (10.0, 7.0)),
                    Placement::new("J4", "terminal", "terminal", (0.92, 0.66), (10.0, 7.0)),
                ]);
                (
                    "双路驱动",
                    "电机驱动与输出端子；负载供电及热设计待验证",
                    vec!["U2", "J3", "J4"],
                )
            }
            _ => {
                placements.extend([
                    Placement::new("U2", "sht30", "sensor", (0.80, 0.47), (6.0, 6.0)),
                    Placement::new("U3", "bh1750", "sensor", (0.80, 0.69), (5.0, 4.0)),
                    Placement::new("R4", "r4k7", "resistor", (0.67, 0.47), (3.0, 1.6)),
                    Placement::new("R5", "r4k7", "resistor", (0.67, 0.58), (3.0, 1.6)),
                ]);
                (
                    "环境采集",
                    "温湿度与光照采集，I²C 连接示意",
                    vec!["U2", "U3", "R4", "R5"],
                )
            }
        };
        let footprints: Vec<Footprint> = placements
            .iter()
            .map(|placement| Footprint {
                reference: placement.reference.into(),
                part_id: placement.part.into(),
                label: parts
                    .iter()
                    .find(|part| part.id == placement.part)
                    .unwrap()
                    .name
                    .clone(),
                kind: placement.kind.into(),
                x_mm: placement.position.0 * f64::from(constraints.width_mm),
                y_mm: placement.position.1 * f64::from(constraints.height_mm),
                width_mm: placement.size.0,
                height_mm: placement.size.1,
            })
            .collect();
        let bom: Vec<BomItem> = parts
            .into_iter()
            .filter_map(|part| {
                let references: Vec<String> = footprints
                    .iter()
                    .filter(|footprint| footprint.part_id == part.id)
                    .map(|footprint| footprint.reference.clone())
                    .collect();
                if references.is_empty() {
                    None
                } else {
                    Some(BomItem {
                        quantity: references.len() as u32,
                        part,
                        references,
                    })
                }
            })
            .collect();
        let estimated_cost_cents = bom
            .iter()
            .map(|item| item.quantity * item.part.unit_price_cents)
            .sum();
        let mut nets: Vec<Net> = [
            ("USB_5V", "J1", "U1", "front"),
            ("+3V3", "U1", "M1", "front"),
            ("GND", "J1", "J2", "back"),
            ("GPIO_EXP", "M1", "J2", "front"),
            ("STATUS", "M1", "R3", "front"),
            ("LED", "R3", "D1", "front"),
            ("BUTTON", "SW1", "M1", "back"),
            ("DECOUPLE", "U1", "C1", "front"),
        ]
        .into_iter()
        .map(|(name, from_ref, to_ref, layer)| Net {
            name: name.into(),
            from_ref: from_ref.into(),
            to_ref: to_ref.into(),
            layer: layer.into(),
        })
        .collect();
        let peripheral_nets = match template.id.as_str() {
            "irrigation" => vec![
                ("SOIL_ADC", "M1", "J3"),
                ("PUMP_CTRL", "M1", "Q1"),
                ("PUMP_OUT", "Q1", "J4"),
                ("FLYBACK", "D2", "J4"),
            ],
            "motion" => vec![
                ("PWM_CTRL", "M1", "U2"),
                ("MOTOR_A", "U2", "J3"),
                ("MOTOR_B", "U2", "J4"),
            ],
            _ => vec![
                ("I2C_SDA", "M1", "U2"),
                ("I2C_SCL", "M1", "U3"),
                ("SDA_PULLUP", "R4", "U2"),
                ("SCL_PULLUP", "R5", "U3"),
            ],
        };
        nets.extend(
            peripheral_nets
                .into_iter()
                .map(|(name, from_ref, to_ref)| Net {
                    name: name.into(),
                    from_ref: from_ref.into(),
                    to_ref: to_ref.into(),
                    layer: "front".into(),
                }),
        );
        let modules = [
            (
                "power",
                "电源管理",
                "USB 5 V → 3.3 V 逻辑电源假设",
                vec!["J1", "U1", "C1", "C2", "C4"],
                "power",
            ),
            (
                "control",
                "无线主控",
                "ESP32-C3 参考模块；通信固件未实现",
                vec!["M1", "R1", "R2", "C3"],
                "control",
            ),
            (
                "peripheral",
                peripheral_name,
                peripheral_description,
                peripheral_refs,
                "sensor",
            ),
            (
                "interface",
                "交互与扩展",
                "状态指示、功能按键和 GPIO 扩展",
                vec!["D1", "R3", "SW1", "J2"],
                "interface",
            ),
        ]
        .into_iter()
        .map(|(id, name, description, references, kind)| DesignModule {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            references: references.into_iter().map(String::from).collect(),
            kind: kind.into(),
        })
        .collect();
        let out_of_bounds: Vec<&str> = footprints
            .iter()
            .filter(|footprint| {
                footprint.x_mm - footprint.width_mm / 2.0 < 0.0
                    || footprint.y_mm - footprint.height_mm / 2.0 < 0.0
                    || footprint.x_mm + footprint.width_mm / 2.0 > f64::from(constraints.width_mm)
                    || footprint.y_mm + footprint.height_mm / 2.0 > f64::from(constraints.height_mm)
            })
            .map(|footprint| footprint.reference.as_str())
            .collect();
        let mut overlaps = Vec::new();
        for (index, first) in footprints.iter().enumerate() {
            for second in footprints.iter().skip(index + 1) {
                if (first.x_mm - second.x_mm).abs() < (first.width_mm + second.width_mm) / 2.0
                    && (first.y_mm - second.y_mm).abs() < (first.height_mm + second.height_mm) / 2.0
                {
                    overlaps.push(format!("{}/{}", first.reference, second.reference));
                }
            }
        }
        let references: HashSet<&str> = footprints
            .iter()
            .map(|footprint| footprint.reference.as_str())
            .collect();
        let references_valid = references.len() == footprints.len()
            && nets.iter().all(|net| {
                references.contains(net.from_ref.as_str())
                    && references.contains(net.to_ref.as_str())
            });
        let mut checks = vec![
            check(
                "template",
                "需求与模板覆盖",
                CheckStatus::Warning,
                if irrigation || motion || environment {
                    "按关键词选择参考模板，仅覆盖摘要列出的模块；自由文本中的其他功能不会自动实现。"
                } else {
                    "没有识别到支持的应用类型，暂以环境感知模板展示。你的原始需求尚未被完整实现。"
                },
            ),
            check(
                "references",
                "示意位号一致性",
                if references_valid {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Blocked
                },
                "检查位号唯一性和模块级网络引用；不检查真实管脚或电气连接。",
            ),
            check(
                "bounds",
                "示意板框边界",
                if out_of_bounds.is_empty() {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Blocked
                },
                if out_of_bounds.is_empty() {
                    "所有示意器件包围框位于板框内；不包含真实封装公差。".into()
                } else {
                    format!(
                        "这些器件超出示意板框：{}。请增大板尺寸。",
                        out_of_bounds.join("、")
                    )
                },
            ),
            check(
                "overlap",
                "示意器件重叠",
                if overlaps.is_empty() {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Blocked
                },
                if overlaps.is_empty() {
                    "示意包围框未重叠；不替代真实器件间距 DRC。".into()
                } else {
                    format!(
                        "包围框重叠：{}。请增大板尺寸后重新生成。",
                        overlaps.join("、")
                    )
                },
            ),
            check(
                "budget",
                "演示 BOM 预算",
                if estimated_cost_cents <= constraints.budget_cny * 100 {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Warning
                },
                format!(
                    "内置估值 ¥{:.2} / 预算 ¥{}。不含 PCB、加工、运费及外部负载，非实时报价。",
                    f64::from(estimated_cost_cents) / 100.0,
                    constraints.budget_cny
                ),
            ),
            check(
                "erc",
                "电气规则检查 · ERC",
                CheckStatus::Pending,
                "未生成真实原理图，未运行 ERC；电源、CC、上下拉与保护等外围尚未完成工程设计。",
            ),
            check(
                "drc",
                "布线规则检查 · DRC",
                CheckStatus::Pending,
                "当前网络是模块级示意连线，不是管脚级网表；未完成真实布线或运行 DRC。",
            ),
            check(
                "simulation",
                "电路仿真",
                CheckStatus::Pending,
                "未执行 SPICE、信号完整性、热分析或功耗仿真。",
            ),
            check(
                "dfm",
                "可制造性检查 · DFM",
                CheckStatus::Pending,
                "尚未连接制造商工艺规则，禁止将本次导出用于制造。",
            ),
        ];
        if matches(&[
            "220v", "220 v", "110v", "110 v", "市电", "高压", "医疗", "mains", "medical",
        ]) {
            checks.push(check("unsupported_safety", "超出安全适用范围", CheckStatus::Blocked, "此模板不支持市电、高压或医疗用途。请改为低压概念需求，并由专业工程师独立设计与验证。"));
        }
        if matches(&[
            "低功耗",
            "电池",
            "续航",
            "battery",
            "low power",
            "12v",
            "12 v",
            "24v",
            "24 v",
        ]) {
            checks.push(check(
                "power_assumption",
                "供电 / 功耗需求未实现",
                CheckStatus::Warning,
                "模板仍采用 USB 5 V 逻辑供电假设，未设计充电、升降压、电池保护或低功耗策略。",
            ));
        }
        if template.id == "irrigation" || template.id == "motion" {
            checks.push(check(
                "load_power",
                "独立负载电源待设计",
                CheckStatus::Warning,
                "负载电源、驱动电流、续流、隔离与热设计未验证。不要直接连接电机或水泵。",
            ));
        }
        if constraints.layers == 4 {
            checks.push(check(
                "stackup",
                "四层叠层未实现",
                CheckStatus::Warning,
                "已记录四层板意向，但演示只绘制顶/底层示意连线，内层和实际叠层需后续 EDA 设计。",
            ));
        }
        Design {
            circuit: None,
            engineering: None,
            engine: "rule-based-demo-v1".into(),
            ai: None,
            template_id: template.id,
            template_name: template.name.clone(),
            summary: format!(
                "已将需求映射为「{}」参考方案：USB 逻辑供电、ESP32 无线主控、{}和交互扩展，共 {} 个示意元件。你可以查看布局、调整板框，再确认概念方案。",
                template.name,
                peripheral_name,
                footprints.len()
            ),
            notice: DEMO_NOTICE.into(),
            manufacturing_ready: false,
            requirements: vec![
                Requirement {
                    label: "应用方向".into(),
                    value: template.name,
                },
                Requirement {
                    label: "核心控制".into(),
                    value: "ESP32-C3 · 模板选型".into(),
                },
                Requirement {
                    label: "逻辑供电".into(),
                    value: "USB 5 V · 待验证假设".into(),
                },
                Requirement {
                    label: "目标板框".into(),
                    value: format!(
                        "{} × {} mm · {} 层",
                        constraints.width_mm, constraints.height_mm, constraints.layers
                    ),
                },
            ],
            modules,
            bom,
            footprints,
            nets,
            checks,
            estimated_cost_cents,
        }
    }
}
