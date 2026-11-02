import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const source = await readFile(new URL("../web/board.js", import.meta.url), "utf8");
const { BoardCanvas, boardProvenance, engineeringGeometry } = await import(`data:text/javascript;base64,${Buffer.from(source).toString("base64")}`);

function rcProject(artifacts = []) {
  return {
    constraints: { width_mm: 80, height_mm: 50 },
    design: {
      circuit: { kind: "rc_lowpass", resistance_ohm: 10000, capacitance_nf: 100, max_input_v: 5 },
      engineering: { verified: false, artifacts },
    },
  };
}

function geometryReport() {
  return {
    name: "pads.json",
    production: false,
    content: JSON.stringify({
      pad_geometry: [{ reference: "R1", number: "1", net: "IN", x: 25, y: 15, width: 1.8, height: 1.8, drill: 0.8 }],
      routes: [{ x1: 25, y1: 15, x2: 25, y2: 8, width: 0.4, layer: "front", net: "IN" }],
    }),
  };
}

function renderEngineering(project) {
  const canvas = {
    container: { innerHTML: "", querySelector: () => null },
    updateTransform() {},
    updateVisibility() {},
  };
  BoardCanvas.prototype.renderEngineering.call(canvas, project);
  return canvas.container.innerHTML;
}

test("AI analysis does not make a concept template a real PCB", () => {
  const result = boardProvenance({ design: { template_id: "environment", circuit: null, ai: { model: "test-model" } } });
  assert.equal(result.kind, "concept");
  assert.match(result.title, /未生成实际 PCB/);
  assert.match(result.detail, /模型只分析需求并选择模板/);
  assert.match(result.detail, /外观可能完全相同/);
});

test("empty workspace and local template disclose their source", () => {
  assert.equal(boardProvenance(null).kind, "empty");
  assert.match(boardProvenance({ design: { template_id: "environment" } }).detail, /本版未调用模型/);
});

test("missing, invalid or production geometry cannot become a PCB preview", () => {
  for (const content of ["not json", "null", "{}", '{"pad_geometry":{},"routes":[]}', '{"pad_geometry":[],"routes":[]}', '{"pad_geometry":[null],"routes":[]}']) {
    const project = rcProject([{ name: "pads.json", production: false, content }]);
    assert.equal(engineeringGeometry(project), null);
    assert.equal(boardProvenance(project).kind, "missing");
    const html = renderEngineering(project);
    assert.match(html, /不显示虚构铜线/);
    assert.doesNotMatch(html, /<path data-layer=/);
  }
  assert.equal(engineeringGeometry(rcProject([{ ...geometryReport(), production: true }])), null);
});

test("RC preview uses stored pad and route geometry and displays actual values", () => {
  const project = rcProject([geometryReport()]);
  const original = renderEngineering(project);
  assert.match(original, /M25 15 L25 8/);
  assert.match(original, /R1=10000 Ω · C1=100 nF/);
  assert.match(boardProvenance(project).detail, /159\.15 Hz/);
  assert.match(boardProvenance(project).detail, /尚未通过完整工程检查/);
  project.design.circuit.resistance_ohm = 20000;
  project.design.circuit.capacitance_nf = 220;
  assert.match(renderEngineering(project), /R1=20000 Ω · C1=220 nF/);
  assert.match(boardProvenance(project).detail, /36\.17 Hz/);
  project.design.engineering.verified = true;
  assert.match(boardProvenance(project).detail, /仍未生产放行/);
});

test("nonfinite or malformed geometry is rejected without substitute tracks", () => {
  const report = geometryReport();
  const geometry = JSON.parse(report.content);
  geometry.routes[0].x1 = "25";
  report.content = JSON.stringify(geometry);
  const project = rcProject([report]);
  assert.equal(engineeringGeometry(project), null);
  assert.doesNotMatch(renderEngineering(project), /<path data-layer=/);
});
