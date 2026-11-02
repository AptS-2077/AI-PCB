use crate::model::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CircuitSpec {
    pub kind: String,
    pub resistance_ohm: f64,
    pub capacitance_nf: f64,
    pub max_input_v: f64,
}

impl CircuitSpec {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.kind != "rc_lowpass"
            || !self.resistance_ohm.is_finite()
            || !(1000.0..=1_000_000.0).contains(&self.resistance_ohm)
            || !self.capacitance_nf.is_finite()
            || !(1.0..=1000.0).contains(&self.capacitance_nf)
            || !self.max_input_v.is_finite()
            || !(0.1..=5.0).contains(&self.max_input_v)
        {
            return Err("工程模式仅支持 0.1-5V 无源 RC 低通：R=1k-1M ohm，C=1-1000 nF。");
        }
        Ok(())
    }

    pub fn cutoff_hz(&self) -> f64 {
        1.0 / (2.0 * PI * self.resistance_ohm * self.capacitance_nf * 1e-9)
    }

    pub fn spice(&self) -> String {
        let fc = self.cutoff_hz();
        format!(
            "AI-PCB passive RC lowpass - ideal unloaded small-signal model\nV1 IN 0 DC 0 AC 1\nR1 IN OUT {}\nC1 OUT 0 {}n\n.ac dec 100 {} {}\n.meas ac gain_low FIND vm(OUT) AT={}\n.meas ac gain_cutoff FIND vm(OUT) AT={fc}\n.meas ac gain_high FIND vm(OUT) AT={}\n.end\n",
            self.resistance_ohm,
            self.capacitance_nf,
            fc / 1000.0,
            fc * 1000.0,
            fc / 10.0,
            fc * 10.0
        )
    }
}

pub fn design(spec: &CircuitSpec, constraints: &Constraints) -> Design {
    let items = [
        (
            "J1",
            "input",
            "Input / GND",
            "2.54 mm header",
            10.0,
            15.0,
            "interface",
        ),
        (
            "R1",
            "resistor",
            "R",
            "Axial 7.62 mm",
            25.0,
            15.0,
            "passive",
        ),
        ("C1", "capacitor", "C", "Radial 5 mm", 42.0, 20.0, "passive"),
        (
            "J2",
            "output",
            "Output / GND",
            "2.54 mm header",
            55.0,
            15.0,
            "interface",
        ),
    ];
    let scale_x = f64::from(constraints.width_mm) / 80.0;
    let scale_y = f64::from(constraints.height_mm) / 50.0;
    Design {
        engine: "validated-circuit-compiler-v1".into(), ai: None,
        circuit: Some(spec.clone()), engineering: None,
        template_id: "rc_lowpass".into(), template_name: "无源 RC 低通滤波板".into(),
        summary: format!("无源一阶 RC 低通，R={} ohm，C={} nF，理想空载截止频率 {:.2} Hz。", spec.resistance_ohm, spec.capacitance_nf, spec.cutoff_hz()),
        notice: "工程文件由参数化 RC 编译器生成，使用固定拓扑与布局规则；不是任意需求自动设计。修改 R/C 会改变元件值和仿真，但同尺寸下布局通常不变。仅适用低压、低能量信号；仿真为空载理想模型，外部源阻抗和负载会改变响应。工具报告通过及人工复核前禁止生产导出。".into(),
        manufacturing_ready: false,
        requirements: vec![Requirement { label: "输入上限".into(), value: format!("{} V，非电源滤波器", spec.max_input_v) }, Requirement { label: "截止频率".into(), value: format!("{:.2} Hz（理想空载）", spec.cutoff_hz()) }],
        modules: vec![DesignModule { id: "filter".into(), name: "无源低通".into(), description: "J1.1 -> R1 -> J2.1，C1 从输出接地；J1.2/J2.2 共地。".into(), references: vec!["J1".into(), "R1".into(), "C1".into(), "J2".into()], kind: "interface".into() }],
        bom: items.iter().map(|(reference, id, label, package, _, _, _)| BomItem {
            part: Part { id: (*id).into(), name: match *reference { "R1" => format!("{} ohm 1% >=0.25W", spec.resistance_ohm), "C1" => format!("{} nF >=16V non-polar", spec.capacitance_nf), _ => (*label).into() }, description: "通用参数规格；尚未绑定供应商料号，不含采购报价。".into(), category: "工程器件".into(), package: (*package).into(), unit_price_cents: 0 }, references: vec![(*reference).into()], quantity: 1,
        }).collect(),
        footprints: items.iter().map(|(reference, id, label, _, x, y, kind)| Footprint { reference: (*reference).into(), part_id: (*id).into(), label: (*label).into(), kind: (*kind).into(), x_mm: x * scale_x, y_mm: y * scale_y, width_mm: 8.0, height_mm: 4.0 }).collect(),
        nets: vec![Net { name: "IN".into(), from_ref: "J1".into(), to_ref: "R1".into(), layer: "front".into() }, Net { name: "OUT".into(), from_ref: "R1".into(), to_ref: "C1".into(), layer: "front".into() }, Net { name: "OUT".into(), from_ref: "C1".into(), to_ref: "J2".into(), layer: "front".into() }],
        checks: vec![DesignCheck { id: "engineering".into(), name: "工程工具链".into(), status: CheckStatus::Pending, detail: "以工程页中的实际工具报告为准。".into() }], estimated_cost_cents: 0,
    }
}

pub fn board(spec: &CircuitSpec, constraints: &Constraints) -> String {
    let d = design(spec, constraints);
    let mut text = String::from(
        "(kicad_pcb (version 20240108) (generator \"pcbnew\")\n(general (thickness 1.6)) (paper \"A4\")\n(layers (0 \"F.Cu\" signal) (31 \"B.Cu\" signal) (36 \"B.SilkS\" user \"b.silkscreen\") (37 \"F.SilkS\" user \"f.silkscreen\") (38 \"B.Mask\" user) (39 \"F.Mask\" user) (44 \"Edge.Cuts\" user) (47 \"F.CrtYd\" user) (49 \"F.Fab\" user))\n(setup (pad_to_mask_clearance 0))\n(net 0 \"\") (net 1 \"IN\") (net 2 \"OUT\") (net 3 \"GND\")\n",
    );
    for f in &d.footprints {
        let (value, pitch, nets) = match f.reference.as_str() {
            "R1" => (
                format!("{}R", spec.resistance_ohm),
                7.62,
                [(1, "IN"), (2, "OUT")],
            ),
            "C1" => (
                format!("{}nF", spec.capacitance_nf),
                5.0,
                [(2, "OUT"), (3, "GND")],
            ),
            "J1" => ("INPUT".into(), 2.54, [(1, "IN"), (3, "GND")]),
            _ => ("OUTPUT".into(), 2.54, [(2, "OUT"), (3, "GND")]),
        };
        text.push_str(&format!("(footprint \"AI_PCB:{}\" (layer \"F.Cu\") (at {} {}) (attr through_hole)\n(fp_text reference \"{}\" (at 0 -3) (layer \"F.SilkS\") (effects (font (size 1 1) (thickness 0.15))))\n(fp_text value \"{value}\" (at 0 3) (layer \"F.SilkS\") (effects (font (size 0.8 0.8) (thickness 0.12))))\n", f.reference, f.x_mm, f.y_mm, f.reference));
        text.push_str(&footprint_body(&f.reference, pitch));
        for (index, (net, name)) in nets.into_iter().enumerate() {
            text.push_str(&format!("(pad \"{}\" thru_hole {} (at {} 0) (size 1.8 1.8) (drill 0.8) (layers \"*.Cu\" \"*.Mask\") (net {net} \"{name}\"))\n", index + 1, if index == 0 { "rect" } else { "circle" }, index as f64 * pitch));
        }
        text.push_str(")\n");
    }
    text.push_str(&format!("(gr_rect (start 0 0) (end {} {}) (stroke (width 0.05) (type default)) (fill none) (layer \"Edge.Cuts\"))\n)", constraints.width_mm, constraints.height_mm));
    text
}

pub fn routed_board(spec: &CircuitSpec, constraints: &Constraints) -> String {
    let mut text = board(spec, constraints);
    text.truncate(text.trim_end().len() - 1);
    let scale_x = f64::from(constraints.width_mm) / 80.0;
    let scale_y = f64::from(constraints.height_mm) / 50.0;
    let input_x = 10.0 * scale_x;
    let resistor_x = 25.0 * scale_x;
    let capacitor_x = 42.0 * scale_x;
    let output_x = 55.0 * scale_x;
    let main_y = 15.0 * scale_y;
    let capacitor_y = 20.0 * scale_y;
    let top_y = 8.0 * scale_y;
    let ground_y = 30.0 * scale_y;
    for (net, layer, points) in [
        (
            1,
            "F.Cu",
            vec![
                (input_x, main_y),
                (input_x, top_y),
                (resistor_x, top_y),
                (resistor_x, main_y),
            ],
        ),
        (
            2,
            "F.Cu",
            vec![
                (resistor_x + 7.62, main_y),
                (resistor_x + 7.62, capacitor_y),
                (capacitor_x, capacitor_y),
                (capacitor_x, top_y),
                (output_x, top_y),
                (output_x, main_y),
            ],
        ),
        (
            3,
            "B.Cu",
            vec![
                (input_x + 2.54, main_y),
                (input_x + 2.54, ground_y),
                (capacitor_x + 5.0, ground_y),
                (capacitor_x + 5.0, capacitor_y),
            ],
        ),
        (
            3,
            "B.Cu",
            vec![
                (output_x + 2.54, main_y),
                (output_x + 2.54, ground_y),
                (capacitor_x + 5.0, ground_y),
            ],
        ),
    ] {
        for pair in points.windows(2) {
            text.push_str(&format!(
                "(segment (start {} {}) (end {} {}) (width 0.4) (layer \"{layer}\") (net {net}))\n",
                pair[0].0, pair[0].1, pair[1].0, pair[1].1
            ));
        }
    }
    text.push_str(")\n");
    text
}

fn footprint_body(reference: &str, pitch: f64) -> String {
    let (left, right, half_height): (f64, f64, f64) = match reference {
        "R1" => (1.8, pitch - 1.8, 1.2),
        "C1" => (-2.0, pitch + 2.0, 1.5),
        _ => (-1.3, pitch + 1.3, 1.3),
    };
    let mut text = String::new();
    for (layer, l, r, h, width) in [
        ("F.SilkS", left, right, half_height, 0.12),
        ("F.Fab", left, right, half_height, 0.1),
        (
            "F.CrtYd",
            left.min(-1.0) - 0.25,
            right.max(pitch + 1.0) + 0.25,
            half_height + 0.25,
            0.05,
        ),
    ] {
        text.push_str(&format!("(fp_rect (start {l} -{h}) (end {r} {h}) (stroke (width {width}) (type default)) (fill none) (layer \"{layer}\"))\n"));
    }
    text
}

pub fn footprint_libraries(spec: &CircuitSpec) -> Vec<(String, String)> {
    [("J1", 2.54), ("J2", 2.54), ("R1", 7.62), ("C1", 5.0)]
        .into_iter()
        .map(|(reference, pitch)| {
            let mut text = format!("(footprint \"{reference}\" (version 20240108) (generator \"pcbnew\") (layer \"F.Cu\") (attr through_hole)\n(fp_text reference \"REF**\" (at 0 -3) (layer \"F.SilkS\") (effects (font (size 1 1) (thickness 0.15))))\n");
            text.push_str(&footprint_body(reference, pitch));
            for index in 0..2 {
                text.push_str(&format!("(pad \"{}\" thru_hole {} (at {} 0) (size 1.8 1.8) (drill 0.8) (layers \"*.Cu\" \"*.Mask\"))\n", index + 1, if index == 0 { "rect" } else { "circle" }, f64::from(index) * pitch));
            }
            let value = match reference {
                "R1" => format!("{}R", spec.resistance_ohm),
                "C1" => format!("{}nF", spec.capacitance_nf),
                _ => "Connector".into(),
            };
            text.push_str(&format!("(fp_text value \"{value}\" (at 0 3) (layer \"F.Fab\") (effects (font (size 0.8 0.8) (thickness 0.12))))\n)\n"));
            (format!("{reference}.kicad_mod"), text)
        })
        .collect()
}

fn symbols() -> String {
    let mut text = String::new();
    // Embedded symbols make the document independent of global library paths.
    for (kind, prefix) in [("R", "R"), ("C", "C"), ("Connector", "J")] {
        text.push_str(&format!("(symbol \"AI_PCB:{kind}\" (pin_names (offset 0) hide) (in_bom yes) (on_board yes)\n(property \"Reference\" \"{prefix}\" (at 0 5 0) (effects (font (size 1.27 1.27))))\n(property \"Value\" \"{kind}\" (at 0 -5 0) (effects (font (size 1.27 1.27))))\n(symbol \"{kind}_0_1\" "));
        if kind == "C" {
            for x in [-0.5, 0.5] {
                text.push_str(&format!("(polyline (pts (xy {x} -2) (xy {x} 2)) (stroke (width 0.25) (type default)) (fill (type none)))"));
            }
        } else {
            text.push_str("(rectangle (start -2 -1.5) (end 2 1.5) (stroke (width 0.25) (type default)) (fill (type none)))");
        }
        text.push_str(&format!(") (symbol \"{kind}_1_1\""));
        for (num, x, angle) in [(1, -5.08, 0), (2, 5.08, 180)] {
            text.push_str(&format!("(pin passive line (at {x} 0 {angle}) (length {}) (name \"~\" (effects (font (size 1 1)))) (number \"{num}\" (effects (font (size 1 1)))))", if kind == "C" { 4.58 } else { 3.08 }));
        }
        text.push_str("))\n");
    }
    text
}

pub fn symbol_library() -> String {
    format!(
        "(kicad_symbol_lib (version 20241209) (generator \"kicad_symbol_editor\")\n{})\n",
        symbols().replace("(symbol \"AI_PCB:", "(symbol \"")
    )
}

pub fn schematic(spec: &CircuitSpec) -> String {
    let root = Uuid::new_v4();
    let mut text = format!(
        "(kicad_sch (version 20250114) (generator \"eeschema\") (uuid {root}) (paper \"A4\")\n(lib_symbols\n{})\n",
        symbols()
    );
    let entries = [
        ("J1", "Connector", "INPUT".into(), 45.72, 50.8, "IN", "GND"),
        (
            "R1",
            "R",
            format!("{}R", spec.resistance_ohm),
            81.28,
            50.8,
            "IN",
            "OUT",
        ),
        (
            "C1",
            "C",
            format!("{}nF", spec.capacitance_nf),
            81.28,
            76.2,
            "OUT",
            "GND",
        ),
        (
            "J2",
            "Connector",
            "OUTPUT".into(),
            116.84,
            50.8,
            "OUT",
            "GND",
        ),
    ];
    for (reference, kind, value, x, y, a, b) in entries {
        let pin_one = Uuid::new_v4();
        let pin_two = Uuid::new_v4();
        text.push_str(&format!("(symbol (lib_id \"AI_PCB:{kind}\") (at {x} {y} 0) (unit 1) (in_bom yes) (on_board yes) (dnp no) (uuid {})\n(property \"Reference\" \"{reference}\" (at {x} {} 0) (effects (font (size 1.27 1.27))))\n(property \"Value\" \"{value}\" (at {x} {} 0) (effects (font (size 1.27 1.27))))\n(property \"Footprint\" \"AI_PCB:{reference}\" (at {x} {y} 0) (effects (font (size 1.27 1.27)) hide))\n(property \"Datasheet\" \"\" (at {x} {y} 0) (effects (font (size 1.27 1.27)) hide))\n(property \"Description\" \"\" (at {x} {y} 0) (effects (font (size 1.27 1.27)) hide))\n(pin \"1\" (uuid {pin_one}))\n(pin \"2\" (uuid {pin_two}))\n(instances (project \"board\" (path \"/{root}\" (reference \"{reference}\") (unit 1))))\n)\n", Uuid::new_v4(), y - 5.08, y + 5.08));
        for (net, px) in [(a, x - 5.08), (b, x + 5.08)] {
            let end = px + if net == a { -2.54 } else { 2.54 };
            text.push_str(&format!("(wire (pts (xy {px} {y}) (xy {end} {y})) (stroke (width 0) (type default)) (uuid {}))\n(label \"{net}\" (at {end} {y} 0) (effects (font (size 1.27 1.27)) (justify left bottom)) (uuid {}))\n", Uuid::new_v4(), Uuid::new_v4()));
        }
    }
    text.push_str("(sheet_instances (path \"/\" (page \"1\")))\n(embedded_fonts no)\n)\n");
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_electrical_envelope() {
        let mut spec = CircuitSpec {
            kind: "rc_lowpass".into(),
            resistance_ohm: 10000.0,
            capacitance_nf: 100.0,
            max_input_v: 5.0,
        };
        assert!(spec.validate().is_ok());
        assert!((spec.cutoff_hz() - 159.1549).abs() < 0.001);
        assert!(spec.spice().contains(".meas ac gain_cutoff"));
        assert_eq!(
            board(&spec, &Constraints::default())
                .matches("(pad ")
                .count(),
            8
        );
        spec.max_input_v = 12.0;
        assert!(spec.validate().is_err());
        spec.max_input_v = 5.0;
        spec.resistance_ohm = f64::NAN;
        assert!(spec.validate().is_err());
    }
}
