use ai_pcb::{
    circuit::CircuitSpec,
    eda::{ToolConfig, build},
    model::Constraints,
};

#[tokio::test]
#[ignore = "Requires configured KiCad CLI, pcbnew Python and ngspice binaries"]
async fn rc_supported_envelope_toolchain() {
    let tools = ToolConfig::default();
    tools.validate().unwrap();
    for (width_mm, height_mm, resistance_ohm, capacitance_nf, max_input_v) in [
        (60, 40, 1000.0, 1000.0, 0.1),
        (160, 120, 1_000_000.0, 1.0, 5.0),
        (60, 120, 1000.0, 1.0, 5.0),
        (160, 40, 1_000_000.0, 1000.0, 0.1),
    ] {
        let spec = CircuitSpec {
            kind: "rc_lowpass".into(),
            resistance_ohm,
            capacitance_nf,
            max_input_v,
        };
        let constraints = Constraints {
            width_mm,
            height_mm,
            ..Constraints::default()
        };
        let run = build(&spec, &constraints, &tools).await.unwrap();
        for step in &run.steps {
            println!(
                "{width_mm}x{height_mm}: {}: {} - {}",
                step.id, step.status, step.detail
            );
        }
        assert!(
            run.verified,
            "RC envelope failed for {width_mm}x{height_mm}, R={resistance_ohm}, C={capacitance_nf}"
        );
        assert!(run.released_at.is_none());
        assert!(
            run.artifacts
                .iter()
                .filter(|artifact| artifact.production)
                .all(|artifact| artifact.content.is_empty())
        );
    }
}

#[tokio::test]
#[ignore = "Requires configured KiCad CLI, pcbnew Python and ngspice binaries"]
async fn real_rc_toolchain() {
    let tools = ToolConfig::default();
    tools.validate().unwrap();
    let spec = CircuitSpec {
        kind: "rc_lowpass".into(),
        resistance_ohm: 10000.0,
        capacitance_nf: 100.0,
        max_input_v: 5.0,
    };
    let run = build(&spec, &Constraints::default(), &tools).await.unwrap();
    if let Ok(path) = std::env::var("AI_PCB_ACCEPTANCE_OUTPUT") {
        let directory = std::path::Path::new(&path);
        std::fs::create_dir_all(directory).unwrap();
        for artifact in &run.artifacts {
            if artifact.production {
                continue;
            }
            let target = directory.join(&artifact.name);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(target, &artifact.content).unwrap();
        }
        std::fs::write(
            directory.join("acceptance.json"),
            serde_json::to_vec_pretty(&run).unwrap(),
        )
        .unwrap();
    }
    for step in &run.steps {
        println!("{}: {} - {}", step.id, step.status, step.detail);
    }
    for id in [
        "kicad_setup",
        "erc",
        "schematic_preview",
        "netlist",
        "parity",
        "route",
        "inspect",
        "pcb_preview",
        "drc",
        "spice",
        "response",
        "gerber",
        "drill",
    ] {
        assert!(
            run.steps
                .iter()
                .any(|step| step.id == id && step.status == "passed"),
            "Missing passed stage: {id}"
        );
    }
    assert!(run.verified);
    assert!(run.released_at.is_none());
    assert!(
        run.artifacts
            .iter()
            .filter(|artifact| artifact.production)
            .count()
            >= 7
    );
    assert!(
        run.artifacts
            .iter()
            .filter(|artifact| artifact.production)
            .all(|artifact| artifact.content.is_empty())
    );
}
