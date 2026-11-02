use crate::{
    circuit::{self, CircuitSpec},
    model::{Constraints, timestamp},
};
use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::Path, process::Stdio, time::Duration};
use tokio::process::Command;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolConfig {
    pub kicad_cli: String,
    pub kicad_python: String,
    pub ngspice: String,
    pub java: String,
    pub freerouting_jar: String,
}

fn locate(variable: &str, name: &str) -> String {
    if let Ok(path) = env::var(variable) {
        return path;
    }
    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .map(|dir| {
            dir.join(if cfg!(windows) {
                format!("{name}.exe")
            } else {
                name.into()
            })
        })
        .find(|path| path.is_file())
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            kicad_cli: locate("AI_PCB_KICAD_CLI", "kicad-cli"),
            kicad_python: env::var("AI_PCB_KICAD_PYTHON").unwrap_or_default(),
            ngspice: locate("AI_PCB_NGSPICE", "ngspice"),
            java: locate("AI_PCB_JAVA", "java"),
            freerouting_jar: env::var("AI_PCB_FREEROUTING_JAR").unwrap_or_default(),
        }
    }
}

impl ToolConfig {
    pub fn validate(&self) -> io::Result<()> {
        for (path, jar) in [
            (&self.kicad_cli, false),
            (&self.kicad_python, false),
            (&self.ngspice, false),
            (&self.java, false),
            (&self.freerouting_jar, true),
        ] {
            if path.is_empty() {
                continue;
            }
            let p = Path::new(path);
            if !p.is_absolute()
                || !p.is_file()
                || path.len() > 1000
                || path.chars().any(char::is_control)
                || (jar && p.extension().and_then(|s| s.to_str()) != Some("jar"))
                || (cfg!(windows) && !jar && p.extension().and_then(|s| s.to_str()) != Some("exe"))
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "工具路径必须是已有可执行文件的绝对路径；布线器为 .jar，可留空。",
                ));
            }
        }
        Ok(())
    }

    pub fn availability(&self) -> serde_json::Value {
        serde_json::json!({
            "kicad": Path::new(&self.kicad_cli).is_file(),
            "pcbnew": Path::new(&self.kicad_python).is_file(),
            "ngspice": Path::new(&self.ngspice).is_file(),
            "freerouting": Path::new(&self.java).is_file() && Path::new(&self.freerouting_jar).is_file(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub production: bool,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineeringRun {
    pub finished_at: u64,
    pub verified: bool,
    pub released_at: Option<u64>,
    pub steps: Vec<Step>,
    pub artifacts: Vec<Artifact>,
}

impl EngineeringRun {
    fn step(&mut self, id: &str, name: &str, status: &str, detail: impl Into<String>) {
        self.steps.push(Step {
            id: id.into(),
            name: name.into(),
            status: status.into(),
            detail: detail.into(),
        });
    }
}

async fn execute(root: &Path, executable: &str, args: &[String], log: &str) -> Result<(), String> {
    if executable.is_empty() || !Path::new(executable).is_file() {
        return Err("缺少工具，请在工程设置中配置路径。".into());
    }
    let file = fs::File::create(root.join(log)).map_err(|_| "无法创建工具日志".to_string())?;
    let mut command = Command::new(executable);
    command
        .args(args)
        .current_dir(root)
        .kill_on_drop(true)
        .env("KICAD_CONFIG_HOME", root.join("kicad-config"))
        .env("KICAD_DOCUMENTS_HOME", root.join("kicad-documents"))
        .stdout(Stdio::from(
            file.try_clone().map_err(|_| "无法创建日志".to_string())?,
        ))
        .stderr(Stdio::from(file));
    for (name, _) in env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_uppercase()
            .ends_with("_API_KEY")
        {
            command.env_remove(name);
        }
    }
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let mut child = command
        .spawn()
        .map_err(|_| "无法启动工具，请检查路径和权限。".to_string())?;
    match tokio::time::timeout(Duration::from_secs(120), child.wait()).await {
        Ok(Ok(status)) if status.success() => Ok(()),
        Ok(Ok(status)) => Err(format!("工具退出码 {:?}，详见 {log}。", status.code())),
        Ok(Err(_)) => Err("工具进程异常。".into()),
        Err(_) => {
            let _ = child.kill().await;
            Err("工具运行超过 120 秒，已终止。".into())
        }
    }
}

async fn stage(
    run: &mut EngineeringRun,
    root: &Path,
    executable: &str,
    id: &str,
    name: &str,
    args: &[&str],
) -> bool {
    let args = args.iter().map(|s| (*s).into()).collect::<Vec<_>>();
    match execute(root, executable, &args, &format!("{id}.log")).await {
        Ok(()) if matches!(id, "erc" | "drc") && !check_report(root, id) => {
            run.step(
                id,
                name,
                "blocked",
                "工具返回成功，但检查报告缺失、格式不符或仍有违规。禁止放行。",
            );
            false
        }
        Ok(()) => {
            run.step(id, name, "passed", "工具执行成功，报告随工程保存。");
            true
        }
        Err(detail) => {
            run.step(id, name, "blocked", detail);
            false
        }
    }
}

fn check_report(root: &Path, id: &str) -> bool {
    let Ok(content) = fs::read(root.join(format!("{id}.json"))) else {
        return false;
    };
    let Ok(report) = serde_json::from_slice::<serde_json::Value>(&content) else {
        return false;
    };
    let empty = |value: &serde_json::Value| value.as_array().is_some_and(Vec::is_empty);
    match id {
        "erc" => {
            report["source"] == "board.kicad_sch"
                && report["sheets"].as_array().is_some_and(|sheets| {
                    !sheets.is_empty() && sheets.iter().all(|sheet| empty(&sheet["violations"]))
                })
        }
        "drc" => {
            report["source"] == "board.kicad_pcb"
                && ["violations", "unconnected_items", "schematic_parity"]
                    .iter()
                    .all(|key| empty(&report[key]))
        }
        _ => false,
    }
}

fn initialize_kicad_config(root: &Path, version: &str) -> Result<(), String> {
    let major = version
        .lines()
        .find_map(|line| {
            let mut parts = line.trim().split('.');
            let major = parts.next()?;
            let minor = parts.next()?;
            let patch = parts.next()?.split('-').next()?;
            if [major, minor, patch]
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
                && parts.next().is_none()
            {
                Some(major)
            } else {
                None
            }
        })
        .unwrap_or("");
    if major.len() > 2
        || !major.bytes().all(|byte| byte.is_ascii_digit())
        || major.parse::<u8>().map_or(true, |number| number < 9)
    {
        return Err("无法识别 KiCad 版本，需要 KiCad 9 或更新版本。".into());
    }
    let config = root.join("kicad-config").join(format!("{major}.0"));
    fs::create_dir_all(&config).map_err(|_| "无法创建隔离的 KiCad 配置目录。")?;
    for (name, content) in [
        ("sym-lib-table", "(sym_lib_table (version 7))\n"),
        ("fp-lib-table", "(fp_lib_table (version 7))\n"),
    ] {
        fs::write(config.join(name), content).map_err(|_| "无法初始化 KiCad 库表。")?;
    }
    Ok(())
}

fn simulation_passed(text: &str) -> bool {
    [
        ("gain_low", 0.985, 1.005),
        ("gain_cutoff", 0.697, 0.718),
        ("gain_high", 0.09, 0.11),
    ]
    .iter()
    .all(|(key, low, high)| {
        text.lines()
            .filter_map(|line| line.trim().strip_prefix(key))
            .filter_map(|suffix| suffix.trim_start().strip_prefix('='))
            .filter_map(|value| value.split_whitespace().next()?.parse::<f64>().ok())
            .any(|value| value.is_finite() && value >= *low && value <= *high)
    })
}

fn collect_files(
    root: &Path,
    directory: &Path,
    run: &mut EngineeringRun,
    total: &mut usize,
) -> io::Result<()> {
    for item in fs::read_dir(directory)? {
        let item = item?;
        if item.file_type()?.is_symlink() {
            continue;
        }
        let path = item.path();
        if path.is_dir() {
            if matches!(
                path.file_name().and_then(|s| s.to_str()),
                Some("production" | "previews" | "AI_PCB.pretty")
            ) {
                collect_files(root, &path, run, total)?;
            }
            continue;
        }
        let name = path
            .strip_prefix(root)
            .map_err(io::Error::other)?
            .to_string_lossy()
            .replace('\\', "/");
        if name == "kicad_bridge.py" || name == "spice_bridge.py" {
            continue;
        }
        let size = item.metadata()?.len() as usize;
        if size > 2 * 1024 * 1024 || *total + size > 8 * 1024 * 1024 {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            *total += size;
            let production = name.starts_with("production/");
            run.artifacts.push(Artifact {
                production,
                name,
                content: if production { String::new() } else { content },
            });
        }
    }
    run.artifacts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(())
}

pub async fn build(
    spec: &CircuitSpec,
    constraints: &Constraints,
    tools: &ToolConfig,
) -> io::Result<EngineeringRun> {
    spec.validate()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    if constraints.layers != 2 || constraints.width_mm < 60 || constraints.height_mm < 40 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "RC 工程要求双层板且至少 60x40 mm。",
        ));
    }
    let temp = tempfile::tempdir()?;
    let root = temp.path().to_path_buf();
    fs::write(root.join("board.kicad_sch"), circuit::schematic(spec))?;
    fs::write(
        root.join("board.kicad_pcb"),
        circuit::board(spec, constraints),
    )?;
    fs::write(root.join("analysis.cir"), spec.spice())?;
    fs::write(
        root.join("spice_bridge.py"),
        include_str!("../tools/spice_bridge.py"),
    )?;
    fs::write(root.join("AI_PCB.kicad_sym"), circuit::symbol_library())?;
    fs::write(
        root.join("sym-lib-table"),
        "(sym_lib_table (version 7) (lib (name \"AI_PCB\") (type \"KiCad\") (uri \"${KIPRJMOD}/AI_PCB.kicad_sym\") (options \"\") (descr \"Local RC symbols\")))\n",
    )?;
    fs::write(
        root.join("fp-lib-table"),
        "(fp_lib_table (version 7) (lib (name \"AI_PCB\") (type \"KiCad\") (uri \"${KIPRJMOD}/AI_PCB.pretty\") (options \"\") (descr \"Local RC footprints\")))\n",
    )?;
    fs::create_dir(root.join("AI_PCB.pretty"))?;
    for (name, content) in circuit::footprint_libraries(spec) {
        fs::write(root.join("AI_PCB.pretty").join(name), content)?;
    }
    fs::write(
        root.join("board.kicad_pro"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "meta": {"filename": "board.kicad_pro", "version": 1},
            "net_settings": {"meta": {"version": 3}, "classes": [{
                "name": "Default", "clearance": 0.2, "track_width": 0.4,
                "via_diameter": 0.8, "via_drill": 0.4,
                "diff_pair_width": 0.25, "diff_pair_gap": 0.25,
                "diff_pair_via_gap": 0.25, "microvia_diameter": 0.3, "microvia_drill": 0.1,
                "bus_width": 12, "wire_width": 6, "line_style": 0,
                "pcb_color": "rgba(0, 0, 0, 0.000)", "schematic_color": "rgba(0, 0, 0, 0.000)"
            }]}
        }))?,
    )?;
    fs::create_dir(root.join("previews"))?;
    fs::write(
        root.join("kicad_bridge.py"),
        include_str!("../tools/kicad_bridge.py"),
    )?;
    fs::write(root.join("circuit.json"), serde_json::to_vec_pretty(spec)?)?;
    let mut run = EngineeringRun {
        finished_at: timestamp(),
        verified: false,
        released_at: None,
        steps: Vec::new(),
        artifacts: Vec::new(),
    };
    run.step(
        "compile",
        "生成管脚级工程源文件",
        "passed",
        "4 个器件、8 个管脚、IN/OUT/GND 三个网络。生成文件尚需 KiCad 解析检查。",
    );
    let setup = execute(
        &root,
        &tools.kicad_cli,
        &["--version".into()],
        "kicad_version.log",
    )
    .await
    .and_then(|()| {
        let version = fs::read_to_string(root.join("kicad_version.log"))
            .map_err(|_| "无法读取 KiCad 版本。".to_string())?;
        initialize_kicad_config(&root, &version)
    });
    let kicad_cli = match setup {
        Ok(()) => {
            run.step(
                "kicad_setup",
                "初始化隔离的 KiCad 库表",
                "passed",
                "使用项目本地库，不读取或修改用户全局库表。",
            );
            tools.kicad_cli.as_str()
        }
        Err(detail) => {
            run.step("kicad_setup", "初始化隔离的 KiCad 库表", "blocked", detail);
            ""
        }
    };
    let erc = stage(
        &mut run,
        &root,
        kicad_cli,
        "erc",
        "原理图 ERC",
        &[
            "sch",
            "erc",
            "--format",
            "json",
            "--severity-all",
            "--exit-code-violations",
            "--output",
            "erc.json",
            "board.kicad_sch",
        ],
    )
    .await;
    stage(
        &mut run,
        &root,
        kicad_cli,
        "schematic_preview",
        "KiCad 原理图预览",
        &[
            "sch",
            "export",
            "svg",
            "--exclude-drawing-sheet",
            "--output",
            "previews/",
            "board.kicad_sch",
        ],
    )
    .await;
    let netlist = stage(
        &mut run,
        &root,
        kicad_cli,
        "netlist",
        "导出原理图网表",
        &[
            "sch",
            "export",
            "netlist",
            "--format",
            "kicadxml",
            "--output",
            "netlist.xml",
            "board.kicad_sch",
        ],
    )
    .await;
    let parity = netlist
        && stage(
            &mut run,
            &root,
            &tools.kicad_python,
            "parity",
            "原理图 / PCB 管脚一致性",
            &[
                "kicad_bridge.py",
                "netlist",
                "board.kicad_pcb",
                "netlist.xml",
            ],
        )
        .await;
    let internal_router = tools.freerouting_jar.is_empty();
    let dsn = !internal_router
        && stage(
            &mut run,
            &root,
            &tools.kicad_python,
            "dsn",
            "KiCad 布线交换文件",
            &["kicad_bridge.py", "export", "board.kicad_pcb", "board.dsn"],
        )
        .await;
    let route = if internal_router {
        fs::write(
            root.join("board.kicad_pcb"),
            circuit::routed_board(spec, constraints),
        )?;
        run.step("route", "RC 固定拓扑自动布线", "passed", "受约束双层曼哈顿布线：0.4 mm 铜线，无过孔；不支持任意电路。仍须 KiCad DRC 和连通性检查。" );
        true
    } else if dsn && Path::new(&tools.freerouting_jar).is_file() {
        stage(
            &mut run,
            &root,
            &tools.java,
            "route",
            "Freerouting 自动布线",
            &[
                "-Djava.awt.headless=true",
                "-jar",
                &tools.freerouting_jar,
                "-de",
                "board.dsn",
                "-do",
                "board.ses",
                "--gui.enabled=false",
                "--api_server.enabled=false",
                "--user_data_path=router-data",
                "-da",
                "-dl",
                "-mp",
                "10",
                "-mt",
                "1",
            ],
        )
        .await
            && root.join("board.ses").is_file()
    } else {
        run.step(
            "route",
            "Freerouting 自动布线",
            "blocked",
            "缺少 DSN 或 Freerouting JAR，未进行自动布线。",
        );
        false
    };
    let imported = route
        && (internal_router
            || stage(
                &mut run,
                &root,
                &tools.kicad_python,
                "import",
                "导入真实布线结果",
                &["kicad_bridge.py", "import", "board.kicad_pcb", "board.ses"],
            )
            .await);
    let inspected = stage(
        &mut run,
        &root,
        &tools.kicad_python,
        "inspect",
        "布线后管脚核验",
        &["kicad_bridge.py", "inspect", "board.kicad_pcb", "pads.json"],
    )
    .await;
    stage(
        &mut run,
        &root,
        kicad_cli,
        "pcb_preview",
        "KiCad PCB 层预览",
        &[
            "pcb",
            "export",
            "svg",
            "--layers",
            "F.Cu,B.Cu,F.SilkS,Edge.Cuts",
            "--mode-single",
            "--page-size-mode",
            "2",
            "--output",
            "previews/pcb.svg",
            "board.kicad_pcb",
        ],
    )
    .await;
    let drc = if imported && inspected {
        stage(
            &mut run,
            &root,
            kicad_cli,
            "drc",
            "PCB DRC / 未连接检查",
            &[
                "pcb",
                "drc",
                "--format",
                "json",
                "--severity-all",
                "--exit-code-violations",
                "--output",
                "drc.json",
                "board.kicad_pcb",
            ],
        )
        .await
    } else {
        run.step(
            "drc",
            "PCB DRC / 未连接检查",
            "blocked",
            "依赖布线和管脚核验成功。",
        );
        false
    };
    let bundled_spice = tools.ngspice.is_empty() && !tools.kicad_python.is_empty();
    let spice = stage(
        &mut run,
        &root,
        if bundled_spice {
            &tools.kicad_python
        } else {
            &tools.ngspice
        },
        "spice",
        "ngspice AC 仿真",
        if bundled_spice {
            &["spice_bridge.py", "analysis.cir", "simulation.txt"]
        } else {
            &["-b", "-o", "simulation.txt", "analysis.cir"]
        },
    )
    .await;
    let sim_ok = spice
        && fs::read_to_string(root.join("simulation.txt"))
            .is_ok_and(|text| simulation_passed(&text));
    run.step(
        "response",
        "低频 / 截止 / 高频增益断言",
        if sim_ok { "passed" } else { "blocked" },
        if sim_ok {
            "理想空载响应满足 0.995 / 0.707 / 0.0995 附近的容差要求；不替代实物测试。"
        } else {
            "缺少有效测量数据或响应超差，不能批准生产。"
        },
    );
    if erc && parity && drc && sim_ok {
        fs::create_dir(root.join("production"))?;
        let gerbers = stage(
            &mut run,
            &root,
            kicad_cli,
            "gerber",
            "Gerber 制造层输出",
            &[
                "pcb",
                "export",
                "gerbers",
                "--layers",
                "F.Cu,B.Cu,F.Mask,B.Mask,F.SilkS,Edge.Cuts",
                "--output",
                "production/",
                "board.kicad_pcb",
            ],
        )
        .await;
        let drills = stage(
            &mut run,
            &root,
            kicad_cli,
            "drill",
            "Excellon 钻孔输出",
            &[
                "pcb",
                "export",
                "drill",
                "--output",
                "production/",
                "board.kicad_pcb",
            ],
        )
        .await;
        run.verified = gerbers && drills && fs::read_dir(root.join("production"))?.count() >= 7;
    }
    run.step(
        "sourcing",
        "实时选型与供应链",
        "pending",
        "通用参数 BOM 尚未绑定供应商料号、机械尺寸或实时库存。采购查询结果不自动等于可替换器件。",
    );
    run.step(
        "release",
        "生产放行",
        "blocked",
        "尚未完成具体料号、封装尺寸、额定值和负载条件的工程签核。生产文件保持锁定。",
    );
    run.finished_at = timestamp();
    collect_files(&root, &root, &mut run, &mut 0)?;
    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn isolated_libraries_are_initialized_for_detected_version() {
        let root = tempfile::tempdir().unwrap();
        initialize_kicad_config(root.path(), "Registry warning\n9.0.9\n").unwrap();
        for name in ["sym-lib-table", "fp-lib-table"] {
            assert!(root.path().join("kicad-config/9.0").join(name).is_file());
        }
        for version in ["", "warning", "../9", "8.0.9", "9.x.1", "999.0.0"] {
            assert!(initialize_kicad_config(root.path(), version).is_err());
        }
    }

    #[test]
    fn rule_checks_require_complete_zero_violation_reports() {
        let root = tempfile::tempdir().unwrap();
        assert!(!check_report(root.path(), "erc"));
        for report in [
            "{}",
            r#"{"source":"board.kicad_sch","sheets":[]}"#,
            r#"{"source":"board.kicad_sch","sheets":[{}]}"#,
            r#"{"source":"board.kicad_sch","sheets":[{"violations":[{"severity":"warning"}]}]}"#,
        ] {
            fs::write(root.path().join("erc.json"), report).unwrap();
            assert!(!check_report(root.path(), "erc"));
        }
        fs::write(
            root.path().join("erc.json"),
            r#"{"source":"board.kicad_sch","sheets":[{"violations":[]}]}"#,
        )
        .unwrap();
        assert!(check_report(root.path(), "erc"));
        let mut report = serde_json::json!({
            "source": "board.kicad_pcb", "violations": [],
            "unconnected_items": [], "schematic_parity": []
        });
        fs::write(root.path().join("drc.json"), report.to_string()).unwrap();
        assert!(check_report(root.path(), "drc"));
        report["unconnected_items"] = serde_json::json!([{}]);
        fs::write(root.path().join("drc.json"), report.to_string()).unwrap();
        assert!(!check_report(root.path(), "drc"));
    }

    #[test]
    fn simulation_requires_three_numeric_assertions() {
        assert!(!simulation_passed("success"));
        assert!(!simulation_passed(
            "gain_low = NaN\ngain_cutoff = .707\ngain_high = .0995"
        ));
        assert!(simulation_passed(
            "gain_low = .995\ngain_cutoff = .7071\ngain_high = .0995"
        ));
        assert!(!simulation_passed(
            "gain_low = .995\ngain_cutoff = .8071\ngain_high = .0995"
        ));
    }

    #[tokio::test]
    async fn missing_tools_never_pass_production() {
        let tools = ToolConfig {
            kicad_cli: String::new(),
            kicad_python: String::new(),
            ngspice: String::new(),
            java: String::new(),
            freerouting_jar: String::new(),
        };
        let spec = CircuitSpec {
            kind: "rc_lowpass".into(),
            resistance_ohm: 10000.0,
            capacitance_nf: 100.0,
            max_input_v: 5.0,
        };
        let run = build(&spec, &Constraints::default(), &tools).await.unwrap();
        assert!(!run.verified);
        assert!(run.released_at.is_none());
        assert!(
            run.steps
                .iter()
                .any(|step| step.id == "erc" && step.status == "blocked")
        );
        assert_eq!(
            run.artifacts
                .iter()
                .filter(|a| a.name.ends_with("kicad_sch"))
                .count(),
            1
        );
        assert!(!run.artifacts.iter().any(|a| a.production));
    }
}
