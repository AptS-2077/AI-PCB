use ai_pcb::{
    circuit::{self, CircuitSpec},
    engine::{DemoEngine, DesignEngine},
    model::{Constraints, Project},
};

fn rc_spec(resistance_ohm: f64, capacitance_nf: f64) -> CircuitSpec {
    CircuitSpec {
        kind: "rc_lowpass".into(),
        resistance_ohm,
        capacitance_nf,
        max_input_v: 5.0,
    }
}

fn board_without_values(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("(fp_text value "))
        .collect()
}

fn segments(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter(|line| line.trim_start().starts_with("(segment "))
        .collect()
}

#[test]
#[ignore = "Read-only audit requires an explicit AI_PCB_AUDIT_PROJECT path"]
fn audit_existing_concept_project_against_demo_template() {
    let path = std::env::var_os("AI_PCB_AUDIT_PROJECT")
        .unwrap_or_else(|| panic!("AI_PCB_AUDIT_PROJECT must be explicitly set"));
    let source =
        std::fs::read(path).unwrap_or_else(|_| panic!("Cannot read the selected audit project"));
    let project: Project = serde_json::from_slice(&source)
        .unwrap_or_else(|_| panic!("Selected audit file is not a valid Project"));
    let design = project
        .design
        .as_ref()
        .unwrap_or_else(|| panic!("Selected audit project has no design"));
    assert!(
        design.circuit.is_none(),
        "This audit only supports concept designs without a circuit"
    );
    assert!(
        matches!(
            design.template_id.as_str(),
            "environment" | "irrigation" | "motion"
        ),
        "This audit requires a supported concept template"
    );
    let reference = DemoEngine.build(
        &project.prompt,
        &project.constraints,
        Some(&design.template_id),
    );
    let footprints_equal = serde_json::to_value(&design.footprints).unwrap()
        == serde_json::to_value(&reference.footprints).unwrap();
    let nets_equal = serde_json::to_value(&design.nets).unwrap()
        == serde_json::to_value(&reference.nets).unwrap();
    let bom_equal =
        serde_json::to_value(&design.bom).unwrap() == serde_json::to_value(&reference.bom).unwrap();
    println!("footprints_equal={footprints_equal}");
    println!("nets_equal={nets_equal}");
    println!("bom_equal={bom_equal}");
    let footprint_structure_equal = design.footprints.len() == reference.footprints.len()
        && design
            .footprints
            .iter()
            .zip(&reference.footprints)
            .all(|(stored, rebuilt)| {
                stored.reference == rebuilt.reference
                    && stored.part_id == rebuilt.part_id
                    && stored.label == rebuilt.label
                    && stored.kind == rebuilt.kind
            });
    let mut max_absolute_difference_mm = 0.0_f64;
    let mut numeric_values_finite = true;
    for (stored, rebuilt) in design.footprints.iter().zip(&reference.footprints) {
        for (stored_value, rebuilt_value) in [
            (stored.x_mm, rebuilt.x_mm),
            (stored.y_mm, rebuilt.y_mm),
            (stored.width_mm, rebuilt.width_mm),
            (stored.height_mm, rebuilt.height_mm),
        ] {
            let difference = (stored_value - rebuilt_value).abs();
            numeric_values_finite &=
                stored_value.is_finite() && rebuilt_value.is_finite() && difference.is_finite();
            max_absolute_difference_mm = max_absolute_difference_mm.max(difference);
        }
    }
    let geometry_within_tolerance =
        footprint_structure_equal && numeric_values_finite && max_absolute_difference_mm <= 1e-9;
    println!("footprint_structure_equal={footprint_structure_equal}");
    println!("numeric_values_finite={numeric_values_finite}");
    println!("max_absolute_difference_mm={max_absolute_difference_mm:.17e}");
    println!("geometry_within_1e_9_mm={geometry_within_tolerance}");
    let serialized_reference = serde_json::to_vec_pretty(&reference.footprints).unwrap();
    let round_tripped_reference: Vec<ai_pcb::model::Footprint> =
        serde_json::from_slice(&serialized_reference).unwrap();
    let stored_matches_round_tripped_reference = serde_json::to_value(&design.footprints).unwrap()
        == serde_json::to_value(&round_tripped_reference).unwrap();
    println!("stored_matches_round_tripped_reference={stored_matches_round_tripped_reference}");
    assert!(
        footprint_structure_equal,
        "Footprint count, order or non-numeric fields differ"
    );
    assert!(
        numeric_values_finite,
        "Footprint coordinates or dimensions are not finite"
    );
    assert!(
        geometry_within_tolerance,
        "Footprint geometry differs by more than 1e-9 mm"
    );
    assert!(nets_equal, "Stored nets differ from the rebuilt template");
    assert!(bom_equal, "Stored BOM differs from the rebuilt template");
}

#[tokio::test]
async fn demo_same_template_ignores_arbitrary_requirements_in_geometry() {
    let constraints = Constraints::default();
    for (template, original_prompt, changed_prompt) in [
        (
            "environment",
            "环境温度监测，增加 OLED 屏幕",
            "环境温度监测，改用 STM32，增加以太网和八路模拟输入，禁止使用 ESP32",
        ),
        (
            "irrigation",
            "土壤浇灌，单个水泵",
            "土壤浇灌，增加十二路独立水泵和隔离反馈",
        ),
        (
            "motion",
            "电机控制，单路输出",
            "电机控制，增加六轴编码器和隔离通信，禁止无线模块",
        ),
    ] {
        let original = DemoEngine
            .generate(original_prompt, &constraints)
            .await
            .unwrap();
        let changed = DemoEngine
            .generate(changed_prompt, &constraints)
            .await
            .unwrap();
        assert_eq!(original.template_id, template);
        assert_eq!(changed.template_id, template);
        assert!(!original.footprints.is_empty());
        assert!(!original.nets.is_empty());
        assert_eq!(
            serde_json::to_value(&original.footprints).unwrap(),
            serde_json::to_value(&changed.footprints).unwrap(),
            "Demo geometry is template-based, not an implementation of arbitrary requirements"
        );
        assert_eq!(
            serde_json::to_value(&original.nets).unwrap(),
            serde_json::to_value(&changed.nets).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&original.bom).unwrap(),
            serde_json::to_value(&changed.bom).unwrap()
        );
        for design in [&original, &changed] {
            assert_eq!(design.engine, "rule-based-demo-v1");
            assert!(design.ai.is_none());
            assert!(design.circuit.is_none());
            assert!(design.engineering.is_none());
            assert!(!design.manufacturing_ready);
        }
    }
}

#[test]
fn rc_each_component_value_changes_kicad_and_spice_sources() {
    let constraints = Constraints::default();
    let original = rc_spec(10_000.0, 100.0);
    let original_schematic = circuit::schematic(&original);
    let original_board = circuit::board(&original, &constraints);
    let original_spice = original.spice();
    assert!(original_schematic.contains("(property \"Value\" \"10000R\""));
    assert!(original_schematic.contains("(property \"Value\" \"100nF\""));
    for changed in [rc_spec(22_000.0, 100.0), rc_spec(10_000.0, 47.0)] {
        changed.validate().unwrap();
        let schematic = circuit::schematic(&changed);
        let board = circuit::board(&changed, &constraints);
        let spice = changed.spice();
        for value in [
            format!("{}R", changed.resistance_ohm),
            format!("{}nF", changed.capacitance_nf),
        ] {
            assert!(schematic.contains(&format!("(property \"Value\" \"{value}\"")));
            assert!(board.contains(&format!("(fp_text value \"{value}\"")));
        }
        let previous_value = if changed.resistance_ohm != original.resistance_ohm {
            "10000R"
        } else {
            "100nF"
        };
        assert!(!schematic.contains(&format!("(property \"Value\" \"{previous_value}\"")));
        assert!(!board.contains(&format!("(fp_text value \"{previous_value}\"")));
        assert!(
            spice
                .lines()
                .any(|line| line == format!("R1 IN OUT {}", changed.resistance_ohm))
        );
        assert!(
            spice
                .lines()
                .any(|line| line == format!("C1 OUT 0 {}n", changed.capacitance_nf))
        );
        assert_ne!(original_board, board);
        assert_ne!(original_spice, spice);
        assert_ne!(original.cutoff_hz(), changed.cutoff_hz());
    }
}

#[test]
fn rc_parameter_changes_keep_fixed_topology_and_geometry_at_same_size() {
    let constraints = Constraints::default();
    let original = rc_spec(10_000.0, 100.0);
    let changed = rc_spec(22_000.0, 47.0);
    let original_design = circuit::design(&original, &constraints);
    let changed_design = circuit::design(&changed, &constraints);
    assert_eq!(
        original_design
            .footprints
            .iter()
            .map(|footprint| footprint.reference.as_str())
            .collect::<Vec<_>>(),
        ["J1", "R1", "C1", "J2"]
    );
    assert_eq!(
        serde_json::to_value(&original_design.footprints).unwrap(),
        serde_json::to_value(&changed_design.footprints).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&original_design.nets).unwrap(),
        serde_json::to_value(&changed_design.nets).unwrap()
    );
    let original_board = circuit::routed_board(&original, &constraints);
    let changed_board = circuit::routed_board(&changed, &constraints);
    assert!(!segments(&original_board).is_empty());
    assert_eq!(segments(&original_board), segments(&changed_board));
    assert_eq!(
        board_without_values(&original_board),
        board_without_values(&changed_board)
    );
    assert!(!original_design.manufacturing_ready);
    assert!(!changed_design.manufacturing_ready);
    let unsupported = CircuitSpec {
        kind: "arbitrary_topology".into(),
        ..original
    };
    assert!(unsupported.validate().is_err());
}

#[test]
fn rc_board_dimensions_change_outline_placement_and_routing_coordinates() {
    let spec = rc_spec(10_000.0, 100.0);
    let original_size = Constraints {
        width_mm: 80,
        height_mm: 50,
        ..Constraints::default()
    };
    for (width_mm, height_mm, expected_first_segment) in [
        (160, 50, "(segment (start 20 15) (end 20 8)"),
        (80, 100, "(segment (start 10 30) (end 10 16)"),
    ] {
        let changed_size = Constraints {
            width_mm,
            height_mm,
            ..original_size.clone()
        };
        let original_board = circuit::routed_board(&spec, &original_size);
        let changed_board = circuit::routed_board(&spec, &changed_size);
        assert!(original_board.contains("(gr_rect (start 0 0) (end 80 50)"));
        assert!(changed_board.contains(&format!(
            "(gr_rect (start 0 0) (end {width_mm} {height_mm})"
        )));
        assert!(original_board.contains("(segment (start 10 15) (end 10 8)"));
        assert!(changed_board.contains(expected_first_segment));
        assert_eq!(
            segments(&original_board).len(),
            segments(&changed_board).len()
        );
        assert_ne!(segments(&original_board), segments(&changed_board));
        let original_design = circuit::design(&spec, &original_size);
        let changed_design = circuit::design(&spec, &changed_size);
        for (original, changed) in original_design
            .footprints
            .iter()
            .zip(&changed_design.footprints)
        {
            assert_eq!(original.reference, changed.reference);
            assert_eq!(changed.x_mm, original.x_mm * f64::from(width_mm) / 80.0);
            assert_eq!(changed.y_mm, original.y_mm * f64::from(height_mm) / 50.0);
        }
        assert_eq!(
            serde_json::to_value(&original_design.nets).unwrap(),
            serde_json::to_value(&changed_design.nets).unwrap()
        );
    }
}
