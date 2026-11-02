import { BoardCanvas, boardProvenance } from "./board.js";
import { mountSettings } from "./settings.js";

const paths = {
  chip: '<rect x="6" y="6" width="12" height="12" rx="2"/><path d="M9 2v4m6-4v4M9 18v4m6-4v4M2 9h4m-4 6h4m12-6h4m-4 6h4"/><rect x="9" y="9" width="6" height="6" rx="1"/>',
  layout: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M8 3v18M8 8h13"/>',
  folder: '<path d="M3 7V5a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/>',
  grid: '<rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/>',
  chevrons: '<path d="m8 8 4-4 4 4m-8 8 4 4 4-4"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  minus: '<path d="M5 12h14"/>',
  close: '<path d="m6 6 12 12M6 18 18 6"/>',
  sparkles: '<path d="m12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5Z"/><path d="m20 2 .7 2.3L23 5l-2.3.7L20 8l-.7-2.3L17 5l2.3-.7Z"/>',
  "arrow-right": '<path d="M4 12h16m-6-6 6 6-6 6"/>',
  "arrow-up": '<path d="M12 20V4m-6 6 6-6 6 6"/>',
  "arrow-up-right": '<path d="M6 18 18 6M6 6h12v12"/>',
  help: '<circle cx="12" cy="12" r="9"/><path d="M9.5 9a2.5 2.5 0 1 1 4.2 1.8c-1 .6-1.7 1-1.7 2.7M12 17h.01"/>',
  home: '<path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1Z"/>',
  board: '<rect x="3" y="4" width="18" height="16" rx="2"/><rect x="9" y="8" width="6" height="7" rx="1"/><path d="M6 7h.01M18 7h.01M6 17h.01M18 17h.01M6 11h3m6 0h3M12 4v4m0 7v5"/>',
  download: '<path d="M12 3v12m-5-5 5 5 5-5M4 15v5a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-5"/>',
  panel: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16m3-12h5m-5 4h5"/>',
  more: '<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>',
  paperclip: '<path d="m9 15 7-7a2 2 0 0 0-3-3l-8 8a4 4 0 0 0 6 6L21 9"/>',
  workflow: '<rect x="3" y="3" width="6" height="6" rx="1"/><rect x="15" y="15" width="6" height="6" rx="1"/><path d="M9 6h6a3 3 0 0 1 3 3v6M6 9v9h9"/>',
  list: '<path d="M9 6h11M9 12h11M9 18h11M4 6h.01M4 12h.01M4 18h.01"/>',
  shield: '<path d="m12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6Z"/><path d="m8 12 3 3 5-6"/>',
  cursor: '<path d="m4 3 6 18 3-8 8-3Z"/>',
  hand: '<path d="M8 13V5a2 2 0 0 1 4 0v7-8a2 2 0 0 1 4 0v8-6a2 2 0 0 1 4 0v9c0 4-2 7-6 7h-2c-2 0-3-1-4-2L3 14a2 2 0 0 1 3-2l2 2"/>',
  eye: '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7h.01"/>',
  maximize: '<path d="M8 3H3v5m13-5h5v5M3 16v5h5m13-5v5h-5"/>',
  coins: '<ellipse cx="10" cy="6" rx="7" ry="3"/><path d="M3 6v4c0 4 14 4 14 0V6M3 10v4c0 3 8 4 12 2m2-7c5 0 7 3 2 5m-7 2v3c0 3 9 3 9-1v-6"/>',
  sliders: '<path d="M4 6h4m4 0h8M4 12h10m4 0h2M4 18h2m4 0h10"/><circle cx="10" cy="6" r="2"/><circle cx="16" cy="12" r="2"/><circle cx="8" cy="18" r="2"/>',
  refresh: '<path d="M20 7v5h-5M4 17v-5h5"/><path d="M5 8a8 8 0 0 1 13-3l2 2M4 17l2 2a8 8 0 0 0 13-3"/>',
  layers: '<path d="m12 3 10 5-10 5L2 8Zm-10 9 10 5 10-5M2 17l10 5 10-5"/>',
  crosshair: '<circle cx="12" cy="12" r="7"/><path d="M12 2v5m0 10v5M2 12h5m10 0h5"/>',
  check: '<path d="m5 12 4 4L19 6"/>',
  search: '<circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/>',
  code: '<path d="m8 5-6 7 6 7m8-14 6 7-6 7m-3-16-2 18"/>',
  file: '<path d="M14 2H5a1 1 0 0 0-1 1v18a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1V8Z"/><path d="M14 2v6h6M8 12h8m-8 4h6"/>',
  lock: '<rect x="5" y="10" width="14" height="12" rx="2"/><path d="M8 10V6a4 4 0 0 1 8 0v4m-4 5v3"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M5 19l1.5-1.5m11-11L19 5"/>',
  leaf: '<path d="M20 3C12 3 3 5 3 13a7 7 0 0 0 12 5c4-4 5-9 5-15Z"/><path d="M3 21 15 9"/>',
  motor: '<rect x="5" y="6" width="14" height="12" rx="3"/><path d="M2 10h3m14 2h4M9 6V3m6 3V3M9 18v3m6-3v3"/><circle cx="12" cy="12" r="3"/>',
  bolt: '<path d="m13 2-9 12h7l-1 8 10-13h-7Z"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  warning: '<path d="m12 3 10 18H2Z"/><path d="M12 9v5m0 3h.01"/>',
};

const byId = (id) => document.getElementById(id);
const escapeHtml = (value) => String(value ?? "").replace(/[&<>"']/g, (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[character]);
const icon = (name) => `<svg class="icon" viewBox="0 0 24 24" aria-hidden="true">${paths[name] ?? paths.chip}</svg>`;
const money = (cents) => `¥${(cents / 100).toFixed(2)}`;
const timeLabel = (value) => new Date(value).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit", hour12: false });
const dateLabel = (value) => new Date(value).toLocaleString("zh-CN", { month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hour12: false });
const statusLabels = { draft: "草稿", pending_review: "待审核", concept_approved: "概念已确认" };
const checkLabels = { passed: "演示检查通过", warning: "需注意", pending: "未执行", blocked: "阻断" };
const checkIcons = { passed: "check", warning: "warning", pending: "clock", blocked: "close" };
const state = { projects: [], project: null, templates: [], components: [], busy: false, library: "projects", drafts: new Map(), connected: false, configVersion: 0, generation: { provider: "demo", requires_remote_consent: false, timeout_seconds: 0 } };
let toastTimer;

function hydrateIcons(root = document) {
  root.querySelectorAll("[data-icon]").forEach((element) => { element.innerHTML = icon(element.dataset.icon); });
}

function notify(message, error = false) {
  clearTimeout(toastTimer);
  byId("toast").innerHTML = `${icon(error ? "warning" : "check")}<span>${escapeHtml(message)}</span>`;
  byId("toast").classList.remove("hidden");
  byId("toast").classList.toggle("error", error);
  toastTimer = setTimeout(() => byId("toast").classList.add("hidden"), error ? 7000 : 4000);
}

function setConnection(connected) {
  state.connected = connected;
  byId("service-indicator").classList.toggle("offline", !connected);
  byId("service-indicator").innerHTML = `<i></i>${connected ? "本地服务已连接" : "服务连接中断"}`;
}

async function request(path, options = {}, raw = false) {
  const { timeoutMs = 15000, ...fetchOptions } = options;
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(path, { ...fetchOptions, signal: controller.signal, headers: { ...fetchOptions.headers, ...(fetchOptions.body ? { "Content-Type": "application/json" } : {}) } });
    setConnection(true);
    if (!response.ok) {
      const payload = await response.json().catch(() => ({}));
      const error = new Error(payload.error || `请求失败 (${response.status})`);
      error.status = response.status;
      throw error;
    }
    return raw ? response : await response.json();
  } catch (error) {
    if (error.name === "AbortError" || error instanceof TypeError) {
      setConnection(false);
      throw new Error("连接超时或服务已断开。写入结果可能已保存，请刷新确认后重试。");
    }
    throw error;
  } finally {
    clearTimeout(timeout);
  }
}

function setBusy(busy, label = "正在生成参考方案…") {
  if (label === "正在生成参考方案…" && state.generation.requires_remote_consent) label = "OpenAI 兼容 API 正在分析需求…";
  state.busy = busy;
  document.body.classList.toggle("busy", busy);
  document.querySelector(".design-space").setAttribute("aria-busy", String(busy));
  ["send-button", "apply-constraints", "review-button", "export-button", "delete-button"].forEach((id) => { byId(id).disabled = busy || !state.project; });
  byId("send-button").disabled = busy;
  document.querySelectorAll("[data-action=settings], [data-action=engineering], [data-delete-project]").forEach((element) => { element.disabled = busy; });
  ["prompt-input", "width-input", "height-input", "layers-input", "budget-input", "create-button", "new-name", "new-prompt"].forEach((id) => { byId(id).disabled = busy; });
  document.querySelectorAll("[data-project-id], [data-action=new], [data-suggestion], [data-new-template], [data-close]").forEach((element) => { element.disabled = busy; });
  byId("send-button").innerHTML = busy ? '<span class="spinner"></span>' : icon("arrow-up");
  byId("save-state").textContent = busy ? label : state.project ? "已保存到本机" : "尚未载入项目";
  byId("bottom-state").textContent = busy ? label : state.project ? `概念方案 v${state.project.revision} · 本地自动保存` : "请选择或新建一个项目";
  byId("review-button").disabled = busy || !state.project || Boolean(state.project.approval);
  byId("confirm-review").disabled = busy || !byId("review-ack").checked || Boolean(state.project?.design.checks.some((check) => check.status === "blocked"));
}

function showDialog(id) {
  document.querySelectorAll("dialog[open]").forEach((dialog) => dialog.close());
  byId(id).showModal();
}

function renderEngineInfo() {
  const remote = state.generation.requires_remote_consent;
  const badge = document.querySelector(".engine-label > span:last-child");
  badge.textContent = remote ? `OpenAI 兼容 API · ${state.generation.model}` : "本地模板引擎";
  badge.title = remote ? `已配置 ${state.generation.remote_origin}，未自动验证 API 连通性；当前项目来源单独标注。` : "未调用远程模型";
  document.querySelector(".composer-hint > span").textContent = remote ? "发送前确认 API 授权" : "内容仅保存在本机";
  byId("generation-mode").textContent = remote ? "Responses API + REFERENCE LAYOUT" : "RULE-BASED DEMO";
  byId("generation-description").textContent = remote
    ? `请求发送至 ${state.generation.remote_origin}，模型 ${state.generation.model}。工程范围：低压无源 RC 低通；其余参考模板不具备真实电路。工程检查通过前禁止生产。`
    : "尚未配置模型 API。工程生成需要先在设置中配置 API。";
  byId("workspace-mode").textContent = remote ? `${state.generation.model} · ${state.generation.remote_origin}` : "API 未配置";
}

function generationTimeout() {
  return Math.max(15000, (state.generation.timeout_seconds + 1500) * 1000);
}

async function authorizeGeneration(prompt, constraints) {
  if (!state.generation.requires_remote_consent) return true;
  const dialog = byId("remote-dialog");
  const previous = document.querySelector("dialog[open]");
  if (previous) previous.close();
  byId("remote-destination").textContent = `${state.generation.remote_origin} · ${state.generation.model}`;
  byId("remote-summary").textContent = `${Array.from(prompt).length} 个字符的需求（包含累计补充）＋ ${constraints.width_mm} × ${constraints.height_mm} mm、${constraints.layers} 层、预算 ¥${constraints.budget_cny}。不上传本地文件或其他项目。`;
  dialog.returnValue = "cancel";
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => {
      if (previous) previous.showModal();
      resolve(dialog.returnValue === "allow");
    }, { once: true });
    dialog.showModal();
  });
}

function renderRecentProjects() {
  byId("project-count").textContent = state.projects.length;
  byId("recent-projects").innerHTML = state.projects.slice(0, 5).map((project) => `<button class="recent-project ${project.id === state.project?.id ? "active" : ""}" data-project-id="${escapeHtml(project.id)}" title="${escapeHtml(project.name)}"><span class="recent-dot"></span><span class="recent-title">${escapeHtml(project.name)}</span><span class="recent-version">v${project.revision}</span></button>`).join("") || '<div class="empty-state">还没有项目</div>';
}

async function refreshProjects() {
  state.projects = await request("/api/projects");
  renderRecentProjects();
}

function checkCounts() {
  return (state.project?.design.checks ?? []).reduce((counts, check) => { counts[check.status] += 1; return counts; }, { passed: 0, warning: 0, pending: 0, blocked: 0 });
}

function checkSummary() {
  return Object.entries(checkCounts()).filter(([, count]) => count > 0).map(([status, count]) => `<span class="check-pill ${status}">${count} ${checkLabels[status]}</span>`).join("");
}

function renderConversation() {
  const project = state.project;
  const design = project.design;
  const analysis = design.ai;
  const engineering = design.engineering;
  const engineeringState = !engineering ? "未执行" : engineering.verified ? "工具通过 · 待选型签核" : "已执行 · 存在阻断";
  const listSection = (title, items) => items.length ? `<details class="ai-detail"><summary>${escapeHtml(title)} · ${items.length}</summary><ul>${items.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}</ul></details>` : "";
  const analysisDetails = analysis ? `<div class="ai-requirements"><strong>模型理解的需求（需核对）</strong><dl>${analysis.plan.requirements.map((requirement) => `<div><dt>${escapeHtml(requirement.label)}</dt><dd>${escapeHtml(requirement.value)}</dd></div>`).join("")}</dl></div>${listSection("假设待确认", analysis.plan.assumptions)}${listSection("模板尚未实现", analysis.plan.unimplemented_requirements)}${listSection("下一步建议", analysis.plan.recommendations)}${listSection("安全疑点", analysis.plan.safety_concerns)}` : "";
  byId("conversation").innerHTML = `
    <article class="chat-message user"><div class="message-label">你的设计需求<time>${timeLabel(project.updated_at)}</time></div><div class="message-body">${escapeHtml(project.prompt)}</div></article>
    <article class="chat-message assistant">
      <div class="message-label source-label">${icon(analysis ? "sparkles" : "file")}需求分析来源：${analysis ? `Responses API · ${escapeHtml(analysis.model)}` : "本地模板 · 未调用模型"}</div>
      <div class="message-body">
        <p class="assistant-intro">${escapeHtml(design.summary)}</p>
        ${analysisDetails}
        <div class="requirement-card"><header>${icon("board")}实际预览：${escapeHtml(design.template_name)}</header><dl>${design.requirements.map((requirement) => `<div><dt>${escapeHtml(requirement.label)}</dt><dd>${escapeHtml(requirement.value)}</dd></div>`).join("")}</dl></div>
        <div class="generation-steps"><div class="generation-step"><span class="step-symbol">${icon("check")}</span>${analysis ? "模型需求分析" : "参考模板与模块划分"}<span>已完成</span></div><div class="generation-step"><span class="step-symbol">${icon("check")}</span>${design.circuit ? "管脚级电路与工程文件" : "内置示意布局与 BOM"}<span>${design.footprints.length} 个元件</span></div><div class="generation-step pending"><span class="step-symbol">${icon("clock")}</span>工程验证与仿真<span>${engineeringState}</span></div></div>
        <p class="inline-demo-note">${escapeHtml(design.notice)}</p><div class="response-actions"><button data-view="checks">${icon("shield")}查看检查</button><button data-action="review">${icon("check")}审核概念</button></div>
      </div>
    </article>`;
  byId("history-count").textContent = project.history.length;
  byId("history-panel").innerHTML = [...project.history].reverse().map((entry, index) => `<article class="history-entry"><header><strong>v${entry.revision}${index === 0 ? " · 当前版本" : ""}</strong><time>${dateLabel(entry.created_at)}</time></header><p>${escapeHtml(entry.prompt)}</p><small>${escapeHtml(entry.summary)}</small></article>`).join("");
  byId("conversation").scrollTop = 0;
}

function renderModules() {
  const modules = state.project.design.modules;
  const moduleIcons = { power: "bolt", control: "chip", sensor: "sun", interface: "workflow" };
  byId("view-modules").innerHTML = `<div class="view-heading"><h3>从想法，到系统结构</h3><span class="tiny-badge">模块级示意</span></div><p class="view-description">展示功能模块及其关系，不是可用于 ERC 的电路原理图。</p><div class="module-flow">${modules.slice(0, 3).map((module, index) => `${index ? `<span>${icon("arrow-right")}</span>` : ""}<div class="module-node ${index === 1 ? "core" : ""}">${icon(moduleIcons[module.kind])}<strong>${escapeHtml(module.name)}</strong><small>${escapeHtml(module.references.slice(0, 2).join(" / "))}</small></div>`).join("")}</div><div class="module-cards">${modules.map((module) => `<article class="module-card"><header>${icon(moduleIcons[module.kind])}<strong>${escapeHtml(module.name)}</strong></header><p>${escapeHtml(module.description)}</p><div>${module.references.map((reference) => `<button class="reference-chip" data-select-ref="${escapeHtml(reference)}">${escapeHtml(reference)}</button>`).join("")}</div></article>`).join("")}</div><p class="inline-demo-note">点击位号定位对应示意元件。接口管脚、电源细节及通信固件均待后续工程设计。</p>`;
}

function renderBom() {
  const design = state.project.design;
  byId("view-bom").innerHTML = `<div class="view-heading"><h3>一张清单，连接整个方案</h3><button class="button secondary small" data-export="bom">${icon("download")}CSV</button></div><p class="view-description">${design.bom.length} 种参考器件 · ${design.footprints.length} 个示意元件 · 非采购清单</p><table class="bom-table"><thead><tr><th>型号 / 位号</th><th>封装</th><th class="number">数量</th><th class="number">估值</th></tr></thead><tbody>${design.bom.map((item) => `<tr><td><strong>${escapeHtml(item.part.name)}</strong><small>${item.references.map((reference) => `<button class="reference-chip" data-select-ref="${escapeHtml(reference)}">${escapeHtml(reference)}</button>`).join("")}</small></td><td>${escapeHtml(item.part.package)}</td><td class="number">${item.quantity}</td><td class="number">${money(item.quantity * item.part.unit_price_cents)}</td></tr>`).join("")}</tbody></table><div class="table-foot"><span>内置演示估值合计</span><strong>${money(design.estimated_cost_cents)}</strong></div><p class="inline-demo-note">不含 PCB 制造、组装、运费或外部负载。内置价格非实时采购报价，器件规格及可采购性尚未核实。</p>`;
}

function renderChecks() {
  byId("view-checks").innerHTML = `<div class="view-heading"><h3>每一步，都知道边界</h3><button class="button secondary small" data-action="review">${icon("check")}审核概念</button></div><p class="view-description">模板数据检查和真实工程检查分开记录，未运行的检查不会显示为通过。</p><div class="check-summary">${checkSummary()}</div>${state.project.design.checks.map((check) => `<article class="check-row ${check.status}"><span>${icon(checkIcons[check.status])}</span><div><header><strong>${escapeHtml(check.name)}</strong><small>${checkLabels[check.status]}</small></header><p>${escapeHtml(check.detail)}</p></div></article>`).join("")}`;
}

function renderComponent(component) {
  if (!component) {
    byId("component-detail").innerHTML = `<div class="selection-empty">${icon("cursor")}<p>点击板上的元件<br><small>查看型号、封装与参考价格</small></p></div>`;
    return;
  }
  const part = state.project.design.bom.find((item) => item.part.id === component.part_id)?.part;
  if (!part) return;
  byId("component-detail").innerHTML = `<div class="component-card-header"><span>${icon("chip")}</span><div><strong>${escapeHtml(component.reference)}</strong><small>${escapeHtml(part.name)}</small></div></div><div class="component-properties"><div class="property-row"><span>封装</span><strong>${escapeHtml(part.package)}</strong></div><div class="property-row"><span>示意坐标</span><strong>${component.x_mm.toFixed(1)}, ${component.y_mm.toFixed(1)}</strong></div><div class="property-row"><span>单价估值</span><strong>${money(part.unit_price_cents)}</strong></div></div><p class="component-description">${escapeHtml(part.description)}</p>`;
}

const board = new BoardCanvas(byId("board-canvas"), renderComponent);

function renderProject() {
  const project = state.project;
  document.title = `${project.name} · AI-PCB`;
  byId("project-name").textContent = project.name;
  byId("project-version").textContent = `v${project.revision}`;
  byId("project-short-id").textContent = `PCB-${project.id.slice(0, 8).toUpperCase()}`;
  byId("review-status").textContent = statusLabels[project.status];
  byId("review-status").classList.toggle("approved", Boolean(project.approval));
  byId("review-summary-title").textContent = project.approval ? "本版概念已确认" : "方案待你确认";
  byId("review-button").innerHTML = `${icon("check")}${project.approval ? "本版概念已确认" : "审核概念方案"}`;
  byId("width-input").value = project.constraints.width_mm;
  byId("height-input").value = project.constraints.height_mm;
  byId("layers-input").value = project.constraints.layers;
  byId("budget-input").value = project.constraints.budget_cny;
  byId("board-dimensions").textContent = `${project.constraints.width_mm} × ${project.constraints.height_mm} mm`;
  byId("metric-components").innerHTML = `${project.design.footprints.length}<em>个</em>`;
  byId("metric-nets").innerHTML = `${project.design.nets.length}<em>组</em>`;
  byId("metric-cost").textContent = project.design.circuit ? "未报价" : money(project.design.estimated_cost_cents);
  byId("check-count").textContent = project.design.checks.filter((check) => check.status !== "passed").length;
  byId("prompt-input").value = state.drafts.get(project.id) ?? "";
  renderEngineInfo();
  renderConversation();
  renderModules();
  renderBom();
  renderChecks();
  renderComponent(null);
  renderRecentProjects();
  renderEngineering();
  renderSchematic();
  renderBoardProvenance();
  board.render(project);
  history.replaceState(null, "", `#project=${project.id}`);
  setBusy(state.busy);
}

function switchView(view) {
  if (!["board", "modules", "bom", "checks", "engineering", "schematic"].includes(view)) return;
  document.querySelectorAll(".design-tab").forEach((tab) => {
    const active = tab.dataset.view === view;
    tab.classList.toggle("active", active);
    tab.setAttribute("aria-selected", String(active));
    tab.tabIndex = active ? 0 : -1;
  });
  document.querySelectorAll(".design-view").forEach((panel) => { panel.classList.toggle("hidden", panel.id !== `view-${view}`); });
}

async function loadProject(id) {
  if (state.busy) return;
  setBusy(true, "正在读取项目…");
  try {
    const project = await request(`/api/projects/${encodeURIComponent(id)}`);
    state.project = project;
    renderProject();
    document.querySelectorAll("dialog[open]").forEach((dialog) => dialog.close());
  } catch (error) {
    notify(error.message, true);
  } finally {
    setBusy(false);
  }
}

function selectTemplate(id) {
  const template = state.templates.find((entry) => entry.id === id);
  if (!template) return;
  byId("new-name").value = template.name;
  byId("new-prompt").value = template.prompt;
  document.querySelectorAll("[data-new-template]").forEach((button) => { button.classList.toggle("selected", button.dataset.newTemplate === id); });
}

function openNewProject(templateId) {
  if (state.busy) return;
  byId("new-error").textContent = "";
  byId("new-template-options").innerHTML = state.templates.map((template) => `<button type="button" class="template-option" data-new-template="${escapeHtml(template.id)}">${icon(template.icon)}${escapeHtml(template.name)}</button>`).join("");
  if (templateId) selectTemplate(templateId);
  showDialog("new-dialog");
  byId(templateId ? "new-prompt" : "new-name").focus();
}

function renderLibrary() {
  const search = byId("library-search").value.trim().toLowerCase();
  let markup = "";
  if (state.library === "projects") {
    markup = state.projects.filter((project) => project.name.toLowerCase().includes(search)).map((project) => `<div class="project-list-row"><button class="project-list-card" data-project-id="${escapeHtml(project.id)}"><span class="list-icon">${icon("board")}</span><span class="list-content"><strong>${escapeHtml(project.name)}</strong><small>${project.component_count} 个元件 · ${statusLabels[project.status]} · ${dateLabel(project.updated_at)}</small></span><span>v${project.revision}</span></button><button class="icon-button danger-text" data-delete-project="${escapeHtml(project.id)}" title="删除项目" aria-label="删除 ${escapeHtml(project.name)}">${icon("close")}</button></div>`).join("");
  } else if (state.library === "templates") {
    const templates = state.templates.filter((template) => `${template.name} ${template.description}`.toLowerCase().includes(search));
    if (templates.length) markup = `<div class="template-library">${templates.map((template) => `<button class="template-library-card" data-use-template="${escapeHtml(template.id)}"><span>${icon(template.icon)}</span><div><strong>${escapeHtml(template.name)}</strong><p>${escapeHtml(template.description)}</p><small>使用参考模板 ${icon("arrow-right")}</small></div></button>`).join("")}</div>`;
  } else {
    const components = state.components.filter((part) => `${part.name} ${part.category} ${part.description}`.toLowerCase().includes(search));
    if (components.length) markup = `<div class="catalog-grid">${components.map((part) => `<article class="catalog-part"><header>${icon("chip")}<strong>${escapeHtml(part.name)}</strong></header><p>${escapeHtml(part.description)}</p><footer><span>${escapeHtml(part.category)} · ${escapeHtml(part.package)}</span><strong>${money(part.unit_price_cents)}</strong></footer></article>`).join("")}</div>`;
  }
  byId("library-content").innerHTML = markup || `<div class="empty-state">${icon("search")}<p>${search ? "没有找到匹配结果，试试其他关键词。" : "这里还没有内容。"}</p></div>`;
}

async function openLibrary(type) {
  if (state.busy) return;
  state.library = type;
  const labels = {
    projects: ["我的项目", "YOUR WORKSPACE", "搜索项目名称", "项目保存在服务端本地数据目录，刷新页面或重启服务不会丢失。"],
    templates: ["每个好想法，都有一个起点", "START WITH A POSSIBILITY", "搜索参考模板", "模板仅用于低压概念演示，不提供经过工程验证的完整电路设计。"],
    components: ["元件库", "THE BUILDING BLOCKS", "搜索型号、类别或描述", "内置参考目录，非实时供应链数据。价格与封装仅用于演示，必须重新核对器件手册。"],
  };
  const [title, eyebrow, placeholder, note] = labels[type];
  byId("library-title").textContent = title;
  byId("library-eyebrow").textContent = eyebrow;
  byId("library-search").placeholder = placeholder;
  byId("library-search").value = "";
  byId("library-note").textContent = note;
  showDialog("library-dialog");
  byId("library-content").innerHTML = '<div class="loading-state"><span class="spinner"></span><p>正在读取本地资料…</p></div>';
  try {
    if (type === "projects") await refreshProjects();
    if (type === "components" && !state.components.length) state.components = await request("/api/components");
    renderLibrary();
  } catch (error) {
    byId("library-content").innerHTML = `<div class="empty-state"><p>${escapeHtml(error.message)}</p><button class="button secondary" data-action="${type}">重新读取</button></div>`;
  }
}

function openReview() {
  if (state.busy || !state.project) return;
  if (state.project.approval) return notify("当前版本的概念方案已确认，仍不可直接投产。");
  byId("review-ack").checked = false;
  byId("review-error").textContent = state.project.design.checks.some((check) => check.status === "blocked") ? "存在阻断项，请先增大板框或修改需求，再重新生成。" : "";
  byId("review-checks").innerHTML = checkSummary();
  byId("confirm-review").disabled = true;
  showDialog("review-dialog");
}

async function acceptProject(project, message) {
  state.project = project;
  renderProject();
  try {
    await refreshProjects();
    notify(message);
  } catch {
    notify(`${message}；项目列表刷新失败，可稍后重试。`, true);
  }
}

async function generate(prompt, constraints) {
  if (!state.project || state.busy) return;
  if (Array.from(prompt.trim()).length > 4000) return notify("累计需求已超过 4000 字符，请新建项目整理需求后再试。", true);
  if (!await authorizeGeneration(prompt, constraints)) return;
  setBusy(true);
  const projectId = state.project.id;
  try {
    const project = await request(`/api/projects/${projectId}/generate`, { method: "POST", timeoutMs: generationTimeout(), body: JSON.stringify({ prompt, constraints, config_version: state.configVersion, expected_revision: state.project.revision, allow_remote: state.generation.requires_remote_consent }) });
    state.drafts.delete(projectId);
    await acceptProject(project, `v${project.revision} 已生成并保存，旧审核已清除。`);
  } catch (error) {
    notify(error.message, true);
  } finally {
    setBusy(false);
  }
}

async function download(format) {
  if (!state.project || !["bom", "project", "report", "engineering"].includes(format)) return;
  byId("export-error").textContent = "";
  const project = state.project;
  const buttons = [...document.querySelectorAll("[data-export]")];
  buttons.forEach((button) => { button.disabled = true; });
  try {
    const response = await request(`/api/projects/${project.id}/export/${format}`, {}, true);
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    const extension = { bom: "csv", project: "json", report: "md", engineering: "zip" }[format];
    anchor.download = response.headers.get("Content-Disposition")?.match(/filename="([^"]+)"/)?.[1] ?? `ai-pcb-demo-${project.id}.${extension}`;
    document.body.append(anchor);
    anchor.click();
    anchor.remove();
    setTimeout(() => URL.revokeObjectURL(url), 30000);
    notify(format === "engineering" ? "完整工程源包已准备下载，生产文件仍锁定。" : "项目资料已准备下载，请核对工程检查状态。");
  } catch (error) {
    byId("export-error").textContent = error.message;
    notify(error.message, true);
  } finally {
    buttons.forEach((button) => { button.disabled = false; });
  }
}

const actions = {
  sourcing: () => { if (!state.busy) showDialog("sourcing-dialog"); },
  settings: () => settingsPanel.open(),
  delete: () => openDelete(state.project),
  engineering: runEngineering,
  new: () => openNewProject(),
  templates: () => openLibrary("templates"),
  components: () => openLibrary("components"),
  projects: () => openLibrary("projects"),
  about: () => showDialog("about-dialog"),
  review: openReview,
  export: () => { if (state.project && !state.busy) showDialog("export-dialog"); },
  workbench: () => { document.querySelectorAll("dialog[open]").forEach((dialog) => dialog.close()); byId("main").scrollIntoView({ behavior: "smooth" }); },
  retry: () => initialize(),
};

document.addEventListener("click", async (event) => {
  const button = event.target.closest("button");
  if (!button || button.disabled) return;
  try {
    if (button.hasAttribute("data-close")) { button.closest("dialog")?.close(); return; }
    if (button.dataset.action) { await actions[button.dataset.action]?.(); return; }
    if (button.dataset.projectId) { await loadProject(button.dataset.projectId); return; }
    if (button.dataset.deleteProject) { openDelete(state.projects.find((project) => project.id === button.dataset.deleteProject)); return; }
    if (button.dataset.view) { switchView(button.dataset.view); return; }
    if (button.dataset.selectRef) { switchView("board"); board.select(button.dataset.selectRef); return; }
    if (button.dataset.export) { await download(button.dataset.export); return; }
    if (button.dataset.newTemplate) { selectTemplate(button.dataset.newTemplate); return; }
    if (button.dataset.useTemplate || button.dataset.suggestion) { openNewProject(button.dataset.useTemplate || button.dataset.suggestion); return; }
    if (button.dataset.chatTab) {
      document.querySelectorAll("[data-chat-tab]").forEach((tab) => { tab.classList.toggle("active", tab === button); });
      byId("conversation").classList.toggle("hidden", button.dataset.chatTab !== "conversation");
      byId("history-panel").classList.toggle("hidden", button.dataset.chatTab !== "history");
    }
  } catch (error) {
    notify(error.message, true);
  }
});

byId("new-project-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  if (state.busy) return;
  const name = byId("new-name").value.trim();
  const prompt = byId("new-prompt").value.trim();
  byId("new-error").textContent = "";
  if (!name || Array.from(prompt).length < 8) { byId("new-error").textContent = "请填写项目名称，并用至少 8 个字符描述需求。"; return; }
  if (!await authorizeGeneration(prompt, { width_mm: 80, height_mm: 50, layers: 2, budget_cny: 80 })) return;
  setBusy(true, "正在创建概念项目…");
  try {
    const project = await request("/api/projects", { method: "POST", timeoutMs: generationTimeout(), body: JSON.stringify({ name, prompt, config_version: state.configVersion, allow_remote: state.generation.requires_remote_consent }) });
    byId("new-dialog").close();
    byId("new-project-form").reset();
    switchView("board");
    await acceptProject(project, "你的新设计已创建，并保存到本机。");
  } catch (error) {
    byId("new-error").textContent = error.message;
    if (!byId("new-dialog").open) notify(error.message, true);
  } finally {
    setBusy(false);
  }
});

byId("prompt-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  if (state.busy) return;
  const supplement = byId("prompt-input").value.trim();
  if (!supplement) return notify("先写下你想补充或修改的需求。", true);
  if (!state.project) {
    if (!state.generation.requires_remote_consent) { await settingsPanel.open(); return; }
    if (Array.from(supplement).length < 8) return notify("请用至少 8 个字符描述需求。", true);
    const constraints = { width_mm: 80, height_mm: 50, layers: 2, budget_cny: 80 };
    if (!await authorizeGeneration(supplement, constraints)) return;
    setBusy(true, "正在分析需求并运行工程任务…");
    try {
      const project = await request("/api/projects", { method: "POST", timeoutMs: generationTimeout(), body: JSON.stringify({ name: Array.from(supplement).slice(0, 30).join(""), prompt: supplement, constraints, config_version: state.configVersion, allow_remote: true }) });
      await acceptProject(project, "项目已保存，工程状态以工具报告为准。");
      if (project.design.circuit) switchView("engineering");
    } catch (error) { notify(error.message, true); }
    finally { setBusy(false); }
    return;
  }
  await generate(`${state.project.prompt}\n\n补充需求：${supplement}`, state.project.constraints);
});
byId("prompt-input").addEventListener("input", () => { if (state.project) state.drafts.set(state.project.id, byId("prompt-input").value); });
byId("prompt-input").addEventListener("keydown", (event) => {
  if (event.key === "Enter" && (event.ctrlKey || event.metaKey) && !event.isComposing) {
    event.preventDefault();
    byId("prompt-form").requestSubmit();
  }
});
byId("constraints-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  if (state.busy || !state.project) return;
  const constraints = { width_mm: Number(byId("width-input").value), height_mm: Number(byId("height-input").value), layers: Number(byId("layers-input").value), budget_cny: Number(byId("budget-input").value) };
  await generate(state.project.prompt, constraints);
});
byId("review-ack").addEventListener("change", () => setBusy(state.busy));
byId("confirm-review").addEventListener("click", async () => {
  if (state.busy || !state.project || !byId("review-ack").checked) return;
  setBusy(true, "正在保存概念审核…");
  byId("review-error").textContent = "";
  try {
    const project = await request(`/api/projects/${state.project.id}/review`, { method: "POST", body: JSON.stringify({ expected_revision: state.project.revision, acknowledge_demo: true }) });
    byId("review-dialog").close();
    await acceptProject(project, "概念方案已确认；仍须完成真实工程验证。");
  } catch (error) {
    byId("review-error").textContent = error.message;
    if (!byId("review-dialog").open) notify(error.message, true);
  } finally {
    setBusy(false);
  }
});
byId("library-search").addEventListener("input", renderLibrary);
byId("zoom-in").addEventListener("click", () => board.setZoom(board.zoom + .15));
byId("zoom-out").addEventListener("click", () => board.setZoom(board.zoom - .15));
byId("fit-board").addEventListener("click", () => board.fit());
byId("toggle-nets").addEventListener("click", () => {
  board.showNets = !board.showNets;
  byId("toggle-nets").setAttribute("aria-pressed", String(board.showNets));
  board.updateVisibility();
});
["front", "back", "silk"].forEach((layer) => byId(`layer-${layer}`).addEventListener("change", (event) => { board.layers[layer] = event.target.checked; board.updateVisibility(); }));
["select", "pan"].forEach((mode) => byId(`${mode}-tool`).addEventListener("click", () => {
  board.setMode(mode);
  ["select", "pan"].forEach((tool) => { byId(`${tool}-tool`).classList.toggle("active", tool === mode); byId(`${tool}-tool`).setAttribute("aria-pressed", String(tool === mode)); });
}));
document.querySelector(".design-tabs").addEventListener("keydown", (event) => {
  if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
  const tabs = [...document.querySelectorAll(".design-tab")];
  const current = tabs.indexOf(document.activeElement);
  if (current < 0) return;
  event.preventDefault();
  const next = event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 : (current + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
  switchView(tabs[next].dataset.view);
  tabs[next].focus();
});
document.querySelectorAll("dialog").forEach((dialog) => {
  dialog.addEventListener("cancel", (event) => { if (state.busy) event.preventDefault(); });
  dialog.addEventListener("click", (event) => {
    if (event.target !== dialog || state.busy) return;
    const bounds = dialog.getBoundingClientRect();
    if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) dialog.close();
  });
});

async function initialize() {
  if (state.busy) return;
  setBusy(true, "正在连接本地服务…");
  try {
    const [health, templates, projects] = await Promise.all([request("/api/health"), request("/api/templates"), request("/api/projects")]);
    state.generation = health.generation ?? state.generation;
    state.configVersion = health.config_version ?? 0;
    renderEngineInfo();
    state.templates = templates;
    state.projects = projects;
    const selectedId = new URLSearchParams(location.hash.slice(1)).get("project");
    const projectId = projects.find((project) => project.id === selectedId)?.id ?? projects[0]?.id;
    if (projectId) {
      state.project = await request(`/api/projects/${projectId}`);
      renderProject();
    } else {
      renderEmpty();
    }
  } catch (error) {
    notify(error.message, true);
    byId("project-name").textContent = "工作空间暂时不可用";
    byId("conversation").innerHTML = `<div class="empty-state">${icon("warning")}<p>${escapeHtml(error.message)}</p><button class="button secondary" data-action="retry">重新连接</button></div>`;
  } finally {
    setBusy(false);
  }
}

let deleteTarget = null;
function openDelete(project) {
  if (!project || state.busy) return;
  deleteTarget = { id: project.id, revision: project.revision };
  byId("delete-description").textContent = `${project.name} · v${project.revision}`;
  byId("delete-ack").checked = false;
  byId("delete-error").textContent = "";
  showDialog("delete-dialog");
}

function renderEmpty() {
  state.project = null;
  document.title = "AI-PCB 设计工作台";
  byId("project-name").textContent = "新建工程";
  byId("project-version").textContent = "";
  byId("project-short-id").textContent = "";
  byId("review-status").textContent = "未开始";
  byId("conversation").innerHTML = '<div class="empty-state"><p>先配置模型 API，再直接输入电路需求。</p><p>当前工程支持 0.1–5 V 无源 RC 低通；例如：设计一个 5V 信号用 RC 低通，R=10kΩ，C=100nF。</p><button class="button secondary" data-action="settings">配置 API</button></div>';
  board.render(null);
  renderBoardProvenance();
  ["view-modules", "view-bom", "view-checks", "history-panel"].forEach((id) => { byId(id).innerHTML = ""; });
  ["metric-components", "metric-nets", "metric-cost", "board-dimensions"].forEach((id) => { byId(id).textContent = "—"; });
  byId("history-count").textContent = "0";
  byId("check-count").textContent = "0";
  byId("prompt-input").value = "";
  history.replaceState(null, "", location.pathname);
  renderEngineering(); renderSchematic(); renderRecentProjects(); renderComponent(null); setBusy(state.busy);
}

byId("delete-project-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  if (state.busy || !deleteTarget || !byId("delete-ack").checked) return;
  setBusy(true, "正在删除项目…");
  const target = deleteTarget;
  try {
    await request(`/api/projects/${target.id}`, { method: "DELETE", body: JSON.stringify({ expected_revision: target.revision }) });
    state.drafts.delete(target.id);
    state.projects = state.projects.filter((project) => project.id !== target.id);
    byId("delete-dialog").close();
    if (state.project?.id === target.id) renderEmpty();
    renderRecentProjects();
    notify("项目及其工程文件已删除。");
  } catch (error) { byId("delete-error").textContent = error.message; }
  finally { setBusy(false); }
});

function renderBoardProvenance() {
  const provenance = boardProvenance(state.project);
  const panel = byId("board-provenance");
  panel.dataset.kind = provenance.kind;
  panel.innerHTML = `<strong>${escapeHtml(provenance.title)}</strong><p>${escapeHtml(provenance.detail)}</p>`;
  byId("tab-board").innerHTML = `${icon("board")}${provenance.kind === "concept" ? "模板示意（非 PCB）" : "PCB 预览"}`;
  document.querySelector(".canvas-corner-label").textContent = provenance.kind === "concept" ? "内置模板 · 连线非实际铜线" : "工程预览 · 生产未放行";
}

function renderSchematic() {
  if (state.project?.design && !state.project.design.circuit) {
    byId("view-schematic").innerHTML = '<div class="empty-state"><p>此项目只有内置概念模板，未生成 KiCad 原理图；配置工具不会把该模板自动转换成真实电路。当前实际工程仅支持无源 RC 低通。</p></div>';
    return;
  }
  const artifacts = state.project?.design.engineering?.artifacts ?? [];
  const index = artifacts.findIndex((file) => file.name.startsWith("previews/") && file.name.endsWith(".svg") && file.name !== "previews/pcb.svg");
  byId("view-schematic").innerHTML = index < 0
    ? '<div class="empty-state"><p>尚无 KiCad 原理图预览。配置 KiCad CLI 后，在工程页重新检查；未生成的预览不会以示意图代替。</p></div>'
    : '<div class="view-heading"><h3>KiCad 实际原理图</h3></div><p class="view-description">由本机 KiCad 解析工程文件后导出；电气连通性以 ERC 和网表一致性报告为准。</p><img class="schematic-preview" alt="KiCad 导出的 RC 原理图" src="/api/projects/' + state.project.id + '/artifacts/' + index + '?expected_revision=' + state.project.revision + '&preview=true">';
}

function renderEngineering() {
  const design = state.project?.design;
  const run = design?.engineering;
  const panel = byId("view-engineering");
  if (!design?.circuit) {
    panel.innerHTML = `<div class="empty-state"><p>${design ? "此项目为概念模板，没有管脚级工程。" : "尚无工程任务"}</p><button class="button secondary" data-action="settings">${icon("sliders")}工程设置</button></div>`;
    return;
  }
  panel.innerHTML = `<div class="engineering-heading"><div><h3>${escapeHtml(design.template_name)}</h3><span class="engineering-badge">${run?.verified ? "工具检查通过 · 待选型签核" : "未通过工程检查"}</span></div><button class="button secondary small" data-export="engineering">${icon("download")}工程 ZIP</button><button class="button secondary small" data-action="engineering">${icon("refresh")}重新检查</button></div>
    <div class="engineering-spec"><span>R ${escapeHtml(design.circuit.resistance_ohm)} Ω</span><span>C ${escapeHtml(design.circuit.capacitance_nf)} nF</span><span>输入 ≤ ${escapeHtml(design.circuit.max_input_v)} V</span></div>
    <div class="engineering-steps">${(run?.steps ?? []).map((step) => `<article class="engineering-step ${escapeHtml(step.status)}"><span>${icon(step.status === "passed" ? "check" : step.status === "blocked" ? "warning" : "clock")}</span><div><strong>${escapeHtml(step.name)}</strong><p>${escapeHtml(step.detail)}</p></div><small>${({ passed: "通过", blocked: "阻断", pending: "待完成" })[step.status] ?? step.status}</small></article>`).join("")}</div>
    <h3 class="artifact-title">工程文件与报告</h3><div class="artifact-list">${(run?.artifacts ?? []).map((file, index) => `<div><span>${icon("file")}${escapeHtml(file.name)}</span>${file.production ? '<span class="danger-text">生产锁定</span>' : `<a href="/api/projects/${state.project.id}/artifacts/${index}?expected_revision=${state.project.revision}" download title="下载 ${escapeHtml(file.name)}" aria-label="下载 ${escapeHtml(file.name)}">${icon("download")}</a>`}</div>`).join("")}</div>`;
}

async function runEngineering() {
  if (state.busy || !state.project?.design.circuit) return;
  setBusy(true, "正在运行本机工程工具…");
  try {
    const project = await request(`/api/projects/${state.project.id}/engineering`, { method: "POST", timeoutMs: 1500000, body: JSON.stringify({ expected_revision: state.project.revision }) });
    await acceptProject(project, "工程检查结束，报告已保存。");
  } catch (error) { notify(error.message, true); }
  finally { setBusy(false); }
}

document.querySelector(".project-actions").insertAdjacentHTML("afterbegin", `<button class="icon-button danger-text" id="delete-button" data-action="delete" title="删除项目" aria-label="删除项目" disabled>${icon("close")}</button>`);
document.querySelector(".design-tabs").insertAdjacentHTML("beforeend", `<button class="design-tab" id="tab-engineering" role="tab" aria-selected="false" aria-controls="view-engineering" data-view="engineering">${icon("workflow")}工程</button>`);
byId("view-checks").insertAdjacentHTML("afterend", '<div id="view-engineering" class="design-view content-view hidden" role="tabpanel" aria-labelledby="tab-engineering"></div>');
document.querySelector(".design-tabs").insertAdjacentHTML("beforeend", '<button class="design-tab" id="tab-schematic" role="tab" aria-selected="false" aria-controls="view-schematic" data-view="schematic">原理图</button>');
byId("view-checks").insertAdjacentHTML("afterend", '<div id="view-schematic" class="design-view content-view hidden" role="tabpanel" aria-labelledby="tab-schematic"></div>');
const settingsPanel = mountSettings({ byId, request, showDialog, state, notify, setBusy, renderEngineInfo, icon, escapeHtml });
byId("sourcing-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  if (state.busy || !byId("part-consent").checked) return;
  setBusy(true, "正在查询 Mouser…");
  byId("sourcing-error").textContent = "";
  byId("sourcing-results").innerHTML = "";
  try {
    const result = await request("/api/components/search", { method: "POST", timeoutMs: 30000, body: JSON.stringify({ keyword: byId("part-query").value.trim(), allow_remote: true }) });
    byId("sourcing-results").innerHTML = `<p class="settings-note">${escapeHtml(result.provider)} · ${dateLabel(result.queried_at)}</p>${result.parts.map((part) => `<article class="source-part"><h3>${escapeHtml(part.mpn)}</h3><small>${escapeHtml(part.manufacturer)} · ${escapeHtml(part.supplier_part)}</small><p>${escapeHtml(part.description)}</p><p>${escapeHtml(part.availability || "库存未返回")} · ${escapeHtml(part.lifecycle)}</p><p>${part.prices.map((price) => `${escapeHtml(price.quantity ?? "")}+: ${escapeHtml(price.price)} ${escapeHtml(price.currency)}`).join(" / ")}</p></article>`).join("") || "无查询结果"}`;
  } catch (error) { byId("sourcing-error").textContent = error.message; }
  finally { setBusy(false); }
});
hydrateIcons();
switchView("board");
initialize();
