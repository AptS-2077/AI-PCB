# AI-PCB · 本地网页工程工作台

Rust / Axum 后端，原生 JavaScript / CSS / SVG 前端。通过网页配置模型 API、输入需求、管理项目，查看 KiCad 原理图、PCB、检查报告和仿真结果。

> **实际工程范围仅为低压无源一阶 RC 低通。** 其他模板仍是概念方案，不是通用自动 EDA。工具检查通过不等于生产放行；供应商料号、封装尺寸、额定值和负载条件尚未完成工程签核，生产文件禁止下载。

## 已实现与边界

| 能力 | 当前状态 |
| --- | --- |
| 图形化 API 设置 | OpenAI 兼容 API、模型、地址、超时、输出格式、流式、连接测试；运行时生效 |
| 项目管理 | 创建、需求迭代、版本校验、确认删除；空目录不自动生成示例 |
| 文本到真实工程 | AI 提取受约束 RC 参数，Rust 生成管脚、网络、封装和 KiCad 源文件 |
| 原理图与 PCB | 原理图、板文件、工程文件、本地符号/封装库、KiCad SVG 预览、工程 ZIP |
| 自动布线 | 默认固定 RC 拓扑双层布线；配置 Freerouting 后可走外部路由，后者未完成真实验收 |
| 工程检查 | KiCad ERC、网表一致性、管脚检查、DRC、ngspice 理想空载 AC 仿真 |
| 实时器件查询 | Mouser Search API 接口和独立密钥设置已接入；真实账户查询尚未验证，不虚构库存 |
| 生产流程 | 通过工具链后在临时目录生成 Gerber/钻孔；内容不会交付，生产门禁保持锁定 |

RC 范围：输入上限 0.1–5 V，电阻 1 kΩ–1 MΩ，非极性电容 1–1000 nF，双层板且至少 60 × 40 mm。当前通用封装只提供几何起点，不代表任意实物器件可直接装配。外部源阻抗和负载会改变滤波响应。

## 启动

需要支持 Rust 2024 edition 的 Rust 工具链与平台链接器；Windows MSVC 需要 C++ Build Tools。前端不需要安装 npm 依赖或访问 CDN。静态资源嵌入可执行文件，修改 `web/` 后需要重新构建。

Windows 推荐从项目目录运行：

```powershell
.\tools\start-local.ps1
```

打开 `http://127.0.0.1:3000`。脚本在存在时使用本项目 `.runtime/eda/kicad/bin/` 的工具，优先保留显式环境配置，不会下载安装工具。不含本机工具的干净检出请在网页「设置 → 工程工具」填入 KiCad CLI 和包含 `pcbnew` 的 Python 绝对路径。

可指定端口和数据目录：

```powershell
.\tools\start-local.ps1 -Port 3001 -DataDirectory data/my-projects
```

通用启动方式：`cargo run --locked`。发布构建：`cargo build --release --locked`。服务仅绑定回环地址，没有账号鉴权，**不要开放公网**。默认数据目录 `data/projects` 使用进程独占锁；多个服务必须使用不同的数据目录。

## 网页验收流程

1. 打开「设置 → 模型 API」，配置 OpenAI 兼容 API，输入目标平台的密钥、API 地址和有权访问的模型，保存。接入方须支持 Responses API，而非仅支持 Chat Completions。
2. 可勾选授权后测试连接。测试会发送固定需求，可能计费；保存成功本身不代表连接已验证。
3. 在「工程工具」填写 KiCad CLI（9 或更新版）及 KiCad Python。ngspice 可用独立程序，也可尝试 KiCad Python 所在目录的 `ngspice.dll`。工具状态「文件存在」不是运行成功证明。
4. 新建设计，输入：**设计一个 5V 以下信号使用的无源 RC 低通，R=10kΩ、C=100nF，80×50mm 双层板，输入/输出各含地线，不要添加主动器件。**
5. 核对远程授权目标后发送。只有模型返回受支持的 `rc_lowpass` 结构才能进入实际工程路径；若显示概念模板，不要将其当成真实工程。
6. 查看「工程」步骤和报告、「原理图」及 PCB 预览。任何工具缺失、报告缺失或违规都会阻断，不会自动宣称通过。工程 ZIP 包含源文件、本地库和报告，不含生产文件。
7. 删除项目需核对对象并确认不可撤销；删除会检查项目版本。普通人工确认不解除生产门禁。

若出现“需求包含未实现功能或安全疑点”，表示模型已返回方案，但本地 RC 校验发现模型列出了阻断项，不是 HTTP/API 配置错误。新版提示分类列出模型报告的具体原因；模型判断仍可能有误，不会自动忽略或重试。独立 RC 验收必须新建项目：在已有项目对话中发送会合并旧需求，湿度传感器、自定义丝印等旧要求仍会超出 RC 范围。下游检查尚未执行与真实的不支持功能应分别处理；源/负载和常规验证建议不应被模型当成已存在的危险用途。

图形化输入的密钥只驻留本地服务内存，不写浏览器存储、项目 JSON 或导出；重启后需要重填。也可参考唯一的 `.env.example` 配置启动环境；仅在当前目录没有 `.env` 时复制，已有文件不要覆盖或提交。手工写入 `.env` 的密钥是操作者自行保存的明文配置，不属于应用自动持久化。密钥不要发到聊天中。更换目标平台或 API 地址必须重新输入与新目标匹配的密钥，不能沿用旧目标的凭据。

远程模型配置统一使用 `provider="openai"` 和 `OPENAI_API_KEY`；`AI_PCB_ENGINE` 仅支持 `auto`、`demo`、`openai`。`auto` 在没有密钥时使用本地模板，显式 `openai` 缺少密钥则启动失败。默认地址为 `https://api.openai.com/v1`，可通过 `OPENAI_BASE_URL` 改为可信的 HTTPS、以 `/v1` 结尾的兼容服务地址；`OPENAI_MODEL` 等参数须按目标平台配置，不保证默认模型可用。具体协议与配置见 `docs/API_COMPATIBILITY.md`。

远程请求仅在明确授权后发送需求和约束；本地读取和健康检查不会调用模型。公开记录不提供真实模型或 Mouser 账户联调成功的证明，不把本地模拟通过称为账户已接通。本次文档整理没有调用远程服务或执行付费验证。

## 架构与接口

详见 `docs/ARCHITECTURE.md`。主要接口：

- `GET /api/health`、`GET /api/settings`：本地状态和已脱敏配置。
- `PUT /api/settings`、`POST /api/settings/test`、`DELETE /api/settings`：配置、授权连接测试、清除模型密钥。
- `PUT /api/settings/tools`、`PUT /api/settings/supplier`：工程工具和供应商配置。
- `POST /api/components/search`：显式授权的供应商实时查询；`GET /api/components` 仍为内置参考目录。
- `GET /api/projects`、`POST /api/projects`、`GET /api/projects/{id}`、`DELETE /api/projects/{id}`：项目管理。
- `POST /api/projects/{id}/generate`、`POST /api/projects/{id}/engineering`：需求迭代与重跑工程检查。
- `GET /api/projects/{id}/export/{bom|project|report|engineering}`：资料和工程源包；生产产物不可下载。

写操作使用 `expected_revision` 或 `expected_version` 检测冲突；过期返回 409。远程生成还必须携带与当前设置一致的 `config_version` 和 `allow_remote: true`。失败不覆盖旧版本，不自动重试计费请求。

## 验证

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
node --check web/app.js
node --check web/board.js
node --check web/settings.js
node --test tests/board_provenance.test.mjs
```

真实 EDA 验收需要工具路径，示例按本机实际安装位置调整：

```powershell
$env:AI_PCB_KICAD_CLI = (Resolve-Path .runtime/eda/kicad/bin/kicad-cli.exe).Path
$env:AI_PCB_KICAD_PYTHON = (Resolve-Path .runtime/eda/kicad/bin/python.exe).Path
$env:AI_PCB_ACCEPTANCE_OUTPUT = "$PWD/.runtime/acceptance-local"
cargo test --locked --test eda_acceptance -- --ignored --nocapture
```

2026-11-02 主线程反馈的本轮结果：`cargo test --locked --offline` 共 27 项通过（11 个单元、12 个接口、4 个来源回归），3 项默认忽略未运行；Node 来源测试 5 项通过，fmt、Clippy（`--all-targets -- -D warnings`）及 `app.js`、`settings.js` 语法检查通过。本次没有重跑真实 EDA、浏览器或远程账户验证，不将本地测试当成付费请求成功。2026-10-20 的 KiCad 9.0.9 / ngspice 结果仅为历史记录。完整结果与未验收项见 `docs/VERIFICATION.md`，生成来源与能力边界见 `docs/GENERATION_AUDIT.md`。
