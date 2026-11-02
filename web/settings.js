export function mountSettings({ byId, request, showDialog, state, notify, setBusy, renderEngineInfo, icon, escapeHtml }) {
  const fields = { provider: "api-provider", base_url: "api-base-url", model: "api-model", timeout_seconds: "api-timeout", max_output_tokens: "api-tokens", reasoning_effort: "api-reasoning", output_format: "api-format" };
  const tools = { kicad_cli: "tool-kicad-cli", kicad_python: "tool-kicad-python", ngspice: "tool-ngspice", java: "tool-java", freerouting_jar: "tool-freerouting-jar" };
  let settings;
  let dirty = false;
  function accept(value) {
    settings = value;
    state.configVersion = value.config_version;
    state.generation = value.generation;
    renderEngineInfo();
    for (const [key, id] of Object.entries(fields)) byId(id).value = value.api[key];
    for (const [key, id] of Object.entries(tools)) byId(id).value = value.tools[key];
    byId("api-key").value = "";
    byId("supplier-key").value = "";
    byId("supplier-status").textContent = value.supplier_configured ? "Mouser Search API 已配置" : "Mouser Search API 未配置";
    byId("api-key").placeholder = value.key_configured ? "已设置，地址不变时留空保留" : "输入此 API 地址对应的密钥";
    byId("api-stream").checked = value.api.stream;
    byId("api-status").textContent = value.verified ? "连接测试通过" : value.key_configured ? "已配置，尚未验证连接" : "尚未配置 API";
    byId("tool-status").innerHTML = Object.entries(value.tool_availability).map(([name, ready]) => `<span class="tool-state ${ready ? "ready" : "missing"}">${icon(ready ? "check" : "warning")}${escapeHtml(name)} · ${ready ? "文件存在" : "未配置"}</span>`).join("");
    byId("clear-api").disabled = !value.key_configured;
    byId("test-consent").checked = false;
    byId("test-api").disabled = true;
    dirty = false;
  }
  async function open() {
    if (state.busy) return;
    byId("settings-error").textContent = "";
    byId("tools-error").textContent = "";
    accept(await request("/api/settings"));
    showDialog("settings-dialog");
  }
  async function operation(errorId, task) {
    if (state.busy) return;
    setBusy(true, "正在更新工作空间设置…");
    byId(errorId).textContent = "";
    byId("settings-dialog").querySelectorAll("input,select,button").forEach((input) => { input.disabled = true; });
    try { accept(await task()); notify("设置已生效"); }
    catch (error) { byId(errorId).textContent = error.message; }
    finally {
      byId("settings-dialog").querySelectorAll("input,select,button").forEach((input) => { input.disabled = false; });
      byId("clear-api").disabled = !settings?.key_configured;
      byId("test-api").disabled = dirty || !settings?.key_configured || !byId("test-consent").checked;
      setBusy(false);
    }
  }
  byId("api-settings-form").addEventListener("input", (event) => {
    if (event.target.id !== "test-consent") { dirty = true; byId("api-status").textContent = "存在未保存的修改"; }
    byId("test-api").disabled = dirty || !settings?.key_configured || !byId("test-consent").checked;
  });
  byId("api-base-url").addEventListener("change", () => {
    byId("api-key").value = "";
    byId("api-key").placeholder = "重新输入此 API 地址对应的密钥";
  });
  byId("api-settings-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const api = Object.fromEntries(Object.entries(fields).map(([key, id]) => [key, byId(id).value.trim()]));
    api.timeout_seconds = Number(api.timeout_seconds);
    api.max_output_tokens = Number(api.max_output_tokens);
    api.api_key = byId("api-key").value;
    api.stream = byId("api-stream").checked;
    await operation("settings-error", () => request("/api/settings", { method: "PUT", body: JSON.stringify({ expected_version: settings.config_version, api }) }));
  });
  byId("test-api").addEventListener("click", () => {
    if (!byId("test-consent").checked || dirty) return;
    return operation("settings-error", () => request("/api/settings/test", { method: "POST", timeoutMs: (settings.api.timeout_seconds + 15) * 1000, body: JSON.stringify({ expected_version: settings.config_version, allow_remote: true }) }));
  });
  byId("clear-api").addEventListener("click", () => operation("settings-error", () => request("/api/settings", { method: "DELETE", body: JSON.stringify({ expected_version: settings.config_version, allow_remote: false }) })));
  byId("tools-settings-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const paths = Object.fromEntries(Object.entries(tools).map(([key, id]) => [key, byId(id).value.trim()]));
    await operation("tools-error", () => request("/api/settings/tools", { method: "PUT", body: JSON.stringify({ expected_version: settings.config_version, tools: paths }) }));
  });
  byId("supplier-settings-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const api_key = byId("supplier-key").value;
    await operation("supplier-error", () => request("/api/settings/supplier", { method: "PUT", body: JSON.stringify({ expected_version: settings.config_version, api_key }) }));
  });
  document.querySelectorAll("[data-settings-tab]").forEach((button) => button.addEventListener("click", () => {
    document.querySelectorAll("[data-settings-tab]").forEach((tab) => tab.setAttribute("aria-selected", String(tab === button)));
    byId("api-settings-form").classList.toggle("hidden", button.dataset.settingsTab !== "api");
    byId("tools-settings-form").classList.toggle("hidden", button.dataset.settingsTab !== "tools");
    byId("supplier-settings-form").classList.toggle("hidden", button.dataset.settingsTab !== "supplier");
  }));
  byId("settings-dialog").addEventListener("close", () => { byId("api-key").value = ""; byId("supplier-key").value = ""; byId("test-consent").checked = false; });
  return { open };
}
