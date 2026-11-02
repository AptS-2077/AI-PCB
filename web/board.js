const escapeXml = (value) => String(value).replace(/[&<>"']/g, (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[character]);

export function engineeringGeometry(project) {
  const report = project?.design?.engineering?.artifacts?.find((file) => file.name === "pads.json" && !file.production);
  try {
    const geometry = JSON.parse(report?.content ?? "null");
    if (!Array.isArray(geometry?.pad_geometry) || !Array.isArray(geometry?.routes)) return null;
    const valid = (value) => typeof value === "number" && Number.isFinite(value) && Math.abs(value) < 10000;
    if (!geometry.pad_geometry.length || !geometry.pad_geometry.every((pad) => pad && [pad.x, pad.y, pad.width, pad.height, pad.drill].every(valid))) return null;
    if (!geometry.routes.every((route) => route && [route.x1, route.y1, route.x2, route.y2, route.width].every(valid))) return null;
    return geometry;
  } catch { return null; }
}

export function boardProvenance(project) {
  const design = project?.design;
  if (!design) return { kind: "empty", title: "尚未生成 PCB", detail: "生成后会明确显示模板示意或工程文件来源。" };
  if (!design.circuit) return {
    kind: "concept",
    title: "内置模板示意 · 未生成实际 PCB",
    detail: `模板 ${design.template_id}：器件、布局和模块连线来自内置数据，并非模型设计的管脚网表。${design.ai ? "模型只分析需求并选择模板。" : "本版未调用模型。"}同一模板在相同板尺寸下外观可能完全相同；改需求不会自动实现新器件、布线或丝印。`,
  };
  const spec = design.circuit;
  const geometry = engineeringGeometry(project);
  const cutoff = 1 / (2 * Math.PI * spec.resistance_ohm * spec.capacitance_nf * 1e-9);
  return {
    kind: geometry ? "engineering" : "missing",
    title: geometry ? "参数化 RC 工程 · 固定拓扑" : "RC 工程 · 尚无有效 PCB 几何报告",
    detail: `R=${spec.resistance_ohm} Ω，C=${spec.capacitance_nf} nF，理想截止频率 ${Number.isFinite(cutoff) ? cutoff.toFixed(2) : "未知"} Hz。Rust 按参数生成 KiCad 与 SPICE 文件；同尺寸下仅改变 R/C 数值通常不会改变布局。${geometry ? "画布来自本版 pads.json 中的实际管脚与铜线。" : "不使用模板图补位；请到工程页查看失败或未执行步骤。"}${design.engineering?.verified ? "工具检查通过，仍未生产放行。" : "尚未通过完整工程检查。"}`,
  };
}

function rectangle(left, top, width, height, fill, radius = 0.2, extra = "") {
  return `<rect x="${left}" y="${top}" width="${width}" height="${height}" rx="${radius}" fill="${fill}" ${extra}/>`;
}

function componentShape(component) {
  const width = component.width_mm;
  const height = component.height_mm;
  const left = -width / 2;
  const top = -height / 2;
  let shape = "";
  if (component.kind === "module") {
    for (let index = 0; index < 10; index += 1) {
      const pinTop = top + 4.8 + index * 1.37;
      shape += rectangle(left - .65, pinTop, 1.3, .7, "#c9b77a", .08);
      shape += rectangle(width / 2 - .65, pinTop, 1.3, .7, "#c9b77a", .08);
    }
    shape += rectangle(left, top, width, height, "#28453a", .45, 'stroke="#b3ae7e" stroke-width=".14"');
    shape += rectangle(left + .65, top + 4.9, width - 1.3, height - 5.7, "url(#shield)", .45, 'stroke="#777d72" stroke-width=".14"');
    shape += `<path d="M ${left + 2} ${top + 3.5} v-2.2 h2.1 v2.2 h2.1 v-2.2 h2.1 v2.2 h2.1 v-2.2 h2.1 v2.2 h2.1" stroke="#b9ad6e" stroke-width=".5" fill="none"/>`;
    shape += `<g text-anchor="middle" fill="#697368" font-family="monospace"><text y="2" font-size="1.6" font-weight="bold">ESP32-C3</text><text y="4.5" font-size=".95">MINI-1</text><text y="7" font-size=".65">REFERENCE MODULE</text></g>`;
    shape += `<circle cx="${left + 1.4}" cy="${top + 6.2}" r=".4" fill="#818c7a"/>`;
  } else if (component.kind === "header") {
    shape += rectangle(left, top, width, height, "#27372d", .25, 'stroke="#868b65" stroke-width=".13"');
    const count = Math.max(3, Math.round(width / 2.6));
    for (let index = 0; index < count; index += 1) {
      const center = left + (index + .5) * width / count;
      shape += rectangle(center - .55, -.55, 1.1, 1.1, "#baaa68", .07);
      shape += rectangle(center - .25, -.25, .5, .5, "#5f6949", .02);
    }
  } else if (component.kind === "usb") {
    shape += rectangle(left + .5, top - .35, width - 1, height + .7, "#949b8d", .65);
    shape += rectangle(left, top, width, height, "url(#shield)", .65, 'stroke="#6b7868" stroke-width=".22"');
    shape += rectangle(left + .6, top + .8, 1.2, height - 1.6, "#536052", .45);
    shape += rectangle(left + 2.2, top + 1, width - 3.4, height - 2, "#bbc2b1", .3);
    for (let index = 0; index < 6; index += 1) shape += rectangle(width / 2 - .1, top + 1.1 + index * 1.1, 1.1, .35, "#c4b879", .04);
  } else if (component.kind === "terminal") {
    shape += rectangle(left, top, width, height, "#83a189", .4, 'stroke="#b2c6a9" stroke-width=".18"');
    for (const center of [-width / 4, width / 4]) {
      shape += `<circle cx="${center}" cy="0" r="1.65" fill="#425b49"/><circle cx="${center}" cy="0" r="1.1" fill="#bfc3ad"/><path d="M${center - .7} -.7 l1.4 1.4" stroke="#68725e" stroke-width=".3"/>`;
    }
  } else if (component.kind === "button") {
    shape += rectangle(left, top, width, height, "#b2b6a2", .4, 'stroke="#88957f" stroke-width=".2"');
    shape += `<circle r="${width * .32}" fill="#4e5848"/><circle r="${width * .25}" fill="#59624f"/>`;
    for (const offset of [-width / 2, width / 2 - .5]) shape += rectangle(offset, -1.5, .7, 3, "#cbc7a9", .08);
  } else if (["resistor", "capacitor", "diode", "led"].includes(component.kind)) {
    const fill = { resistor: "#344136", capacitor: "#b29c66", diode: "#404c3e", led: "#b9d796" }[component.kind];
    shape += rectangle(left - .3, top, width + .6, height, "#b8bc9a", .12);
    shape += rectangle(left + .55, top - .07, width - 1.1, height + .14, fill, .1);
    if (component.kind === "led") shape += `<circle r=".45" fill="#e1f6b9"/>`;
    if (component.kind === "resistor") shape += `<text text-anchor="middle" y=".3" font-size=".65" font-family="monospace" fill="#a6b095">${escapeXml(component.label.split(" ")[0])}</text>`;
  } else {
    for (let index = 0; index < 4; index += 1) {
      const offset = top + .5 + index * (height - 1) / 3;
      shape += rectangle(left - .6, offset - .2, 1, .5, "#b9b88e", .04);
      shape += rectangle(width / 2 - .4, offset - .2, 1, .5, "#b9b88e", .04);
    }
    shape += rectangle(left, top, width, height, "#364637", .35, 'stroke="#88916c" stroke-width=".12"');
    shape += `<circle cx="${left + .7}" cy="${top + .7}" r=".23" fill="#98aa82"/>`;
    shape += `<text text-anchor="middle" y=".4" font-size="${width > 6 ? .95 : .8}" fill="#a4b38e" font-family="monospace">${escapeXml(component.label.split("-")[0].slice(0, 7))}</text>`;
  }
  return `<g class="board-component" role="button" tabindex="0" aria-label="${escapeXml(component.reference)} ${escapeXml(component.label)}" data-reference="${escapeXml(component.reference)}" transform="translate(${component.x_mm} ${component.y_mm})"><title>${escapeXml(component.reference)} · ${escapeXml(component.label)}</title>${rectangle(left - 1.2, top - 1.2, width + 2.4, height + 2.4, "none", .6, 'class="selection-outline" stroke="#e2ecb1" stroke-width=".35" stroke-dasharray="1 .5"')}${shape}<text class="silkscreen" x="0" y="${top - 1.5}" text-anchor="middle" fill="#c4d2a9" font-family="monospace" font-size="1.35" letter-spacing=".1">${escapeXml(component.reference)}</text></g>`;
}

export class BoardCanvas {
  constructor(container, onSelect) {
    this.container = container;
    this.onSelect = onSelect;
    this.zoom = 1;
    this.offset = { x: 0, y: 0 };
    this.mode = "select";
    this.layers = { front: true, back: true, silk: true };
    this.showNets = true;
    this.selection = null;
    this.drag = null;
    this.container.addEventListener("click", (event) => {
      if (this.dragged || this.mode === "pan") return;
      const target = event.target.closest("[data-reference]");
      this.select(target?.dataset.reference ?? null);
    });
    this.container.addEventListener("keydown", (event) => {
      const target = event.target.closest("[data-reference]");
      if (target && ["Enter", " "].includes(event.key)) {
        event.preventDefault();
        this.select(target.dataset.reference);
      }
    });
    this.container.addEventListener("wheel", (event) => {
      if (!this.project || !event.ctrlKey) return;
      event.preventDefault();
      this.setZoom(this.zoom * (event.deltaY < 0 ? 1.1 : .9));
    }, { passive: false });
    this.container.addEventListener("pointerdown", (event) => {
      if (!this.project || (event.button !== 0 && event.button !== 1)) return;
      if (this.mode !== "pan" && event.target.closest("[data-reference]") && event.button !== 1) return;
      const matrix = this.svg.getScreenCTM();
      if (!matrix) return;
      this.dragged = false;
      this.drag = { x: event.clientX, y: event.clientY, original: { ...this.offset }, scale: matrix.a };
      this.container.setPointerCapture(event.pointerId);
    });
    this.container.addEventListener("pointermove", (event) => {
      if (!this.drag) return;
      const deltaX = event.clientX - this.drag.x;
      const deltaY = event.clientY - this.drag.y;
      if (Math.abs(deltaX) + Math.abs(deltaY) < 4 && !this.dragged) return;
      this.dragged = true;
      this.container.classList.add("dragging");
      this.offset = { x: this.drag.original.x + deltaX / this.drag.scale, y: this.drag.original.y + deltaY / this.drag.scale };
      this.updateTransform();
    });
    const endDrag = (event) => {
      this.drag = null;
      this.container.classList.remove("dragging");
      if (this.container.hasPointerCapture(event.pointerId)) this.container.releasePointerCapture(event.pointerId);
      setTimeout(() => { this.dragged = false; }, 0);
    };
    this.container.addEventListener("pointerup", endDrag);
    this.container.addEventListener("pointercancel", endDrag);
  }

  render(project) {
    this.project = project;
    if (!project) { this.svg = null; this.selection = null; this.container.innerHTML = '<div class="empty-state"><p>尚无电路</p></div>'; return; }
    if (project.design.circuit) return this.renderEngineering(project);
    const design = project.design;
    const width = project.constraints.width_mm;
    const height = project.constraints.height_mm;
    this.zoom = 1;
    this.offset = { x: 0, y: 0 };
    this.selection = null;
    const nets = design.nets.map((net, index) => {
      const source = design.footprints.find((part) => part.reference === net.from_ref);
      const destination = design.footprints.find((part) => part.reference === net.to_ref);
      if (!source || !destination) return "";
      const middle = (source.x_mm + destination.x_mm) / 2 + (index % 3 - 1) * 1.4;
      const direction = destination.y_mm >= source.y_mm ? 1 : -1;
      const bend = Math.min(1.5, Math.abs(destination.y_mm - source.y_mm) / 3);
      const path = `M ${source.x_mm} ${source.y_mm} H ${middle - bend} L ${middle} ${source.y_mm + direction * bend} V ${destination.y_mm - direction * bend} L ${middle + bend} ${destination.y_mm} H ${destination.x_mm}`;
      return `<path class="board-net" data-layer="${escapeXml(net.layer)}" data-from="${escapeXml(net.from_ref)}" data-to="${escapeXml(net.to_ref)}" d="${path}" fill="none" stroke="${net.layer === "back" ? "#78a5aa" : "#b7ac61"}" stroke-opacity="${net.layer === "back" ? ".48" : ".72"}" stroke-width="${net.name.includes("5V") || net.name === "+3V3" ? .65 : .28}" stroke-linejoin="round"><title>${escapeXml(net.name)} · 模块级示意</title></path>`;
    }).join("");
    const holes = [[3.3, 3.3], [width - 3.3, 3.3], [3.3, height - 3.3], [width - 3.3, height - 3.3]].map(([horizontal, vertical]) => `<circle cx="${horizontal}" cy="${vertical}" r="1.65" fill="#a9b08a"/><circle cx="${horizontal}" cy="${vertical}" r="1.05" fill="#e3e9d9"/><circle cx="${horizontal}" cy="${vertical}" r=".85" fill="#f4f7ef"/>`).join("");
    const vias = design.footprints.filter((part) => ["ic", "sensor", "module"].includes(part.kind)).map((part) => `<circle cx="${part.x_mm + part.width_mm / 2 + 1.2}" cy="${part.y_mm + 1.2}" r=".45" fill="#a8b680"/><circle cx="${part.x_mm + part.width_mm / 2 + 1.2}" cy="${part.y_mm + 1.2}" r=".18" fill="#345a43"/>`).join("");
    this.container.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="-14 -17 ${width + 28} ${height + 34}" aria-label="${escapeXml(project.name)} 概念 PCB 布局"><defs><linearGradient id="board-fill" x2=".8" y2="1"><stop stop-color="#386c50"/><stop offset="1" stop-color="#285540"/></linearGradient><linearGradient id="shield" x2="1" y2="1"><stop stop-color="#c1c7b4"/><stop offset=".5" stop-color="#abb4a0"/><stop offset="1" stop-color="#ced2bf"/></linearGradient><filter id="board-shadow" x="-30%" y="-40%" width="170%" height="190%"><feDropShadow dx="0" dy="2.4" stdDeviation="2.2" flood-color="#3b553b" flood-opacity=".2"/></filter><clipPath id="board-clip"><rect width="${width}" height="${height}" rx="2.2"/></clipPath></defs><g id="board-transform"><rect y=".55" width="${width}" height="${height}" rx="2.2" fill="#2c4531" filter="url(#board-shadow)"/><rect width="${width}" height="${height}" rx="2.2" fill="url(#board-fill)" stroke="#73915b" stroke-width=".28"/><rect x="1" y="1" width="${width - 2}" height="${height - 2}" rx="1.6" fill="none" stroke="#a3b87c" stroke-opacity=".2" stroke-width=".15"/><g clip-path="url(#board-clip)">${nets}${vias}<g class="silkscreen" fill="#becda1" font-family="monospace"><text x="7" y="6.2" font-size="2.1" font-weight="bold" letter-spacing=".3">AI·PCB</text><text x="7" y="8.3" font-size=".85" fill="#9ab98a">CONCEPT STUDIO / V${project.revision}</text><text x="${width / 2}" y="${height - 2}" font-size=".8" text-anchor="middle" letter-spacing=".7">${escapeXml(design.template_id.toUpperCase())} · DEMO ONLY</text><text x="4.5" y="${height * .56}" font-size=".9">USB 5V</text></g>${design.footprints.map(componentShape).join("")}</g>${holes}<g fill="none" stroke="#a6b89a" stroke-width=".12"><path d="M0 ${height + 2.2}v4.8 M${width} ${height + 2.2}v4.8 M0 ${height + 5}H${width}"/><path d="M-2 0h-5 M-2 ${height}h-5 M-5 0V${height}"/></g><g fill="#9aab8c" font-family="monospace" font-size="1.5"><text x="${width / 2}" y="${height + 8.1}" text-anchor="middle">${width}.0 mm</text><text transform="translate(-7.2 ${height / 2}) rotate(-90)" text-anchor="middle">${height}.0 mm</text></g></g></svg>`;
    this.svg = this.container.querySelector("svg");
    this.updateTransform();
    this.updateVisibility();
  }

  renderEngineering(project) {
    const { width_mm: width, height_mm: height } = project.constraints;
    const geometry = engineeringGeometry(project);
    this.zoom = 1;
    this.offset = { x: 0, y: 0 };
    this.selection = null;
    const valid = (value) => typeof value === "number" && Number.isFinite(value) && Math.abs(value) < 10000;
    const pads = (geometry?.pad_geometry ?? []).filter((pad) => [pad.x, pad.y, pad.width, pad.height, pad.drill].every(valid));
    const routes = (geometry?.routes ?? []).filter((route) => [route.x1, route.y1, route.x2, route.y2, route.width].every(valid));
    const tracks = routes.map((route) => route.via
      ? `<circle cx="${route.x1}" cy="${route.y1}" r="${route.width / 2}" fill="#a9b9ad"/>`
      : `<path data-layer="${route.layer === "back" ? "back" : "front"}" d="M${route.x1} ${route.y1} L${route.x2} ${route.y2}" stroke="${route.layer === "back" ? "#789db4" : "#e0ba70"}" stroke-width="${route.width}" stroke-linecap="round"><title>${escapeXml(route.net)}</title></path>`).join("");
    const padShapes = pads.map((pad) => `<g class="board-component" data-reference="${escapeXml(pad.reference)}" role="button" tabindex="0" aria-label="${escapeXml(pad.reference)}.${escapeXml(pad.number)} ${escapeXml(pad.net)}"><title>${escapeXml(pad.reference)}.${escapeXml(pad.number)} · ${escapeXml(pad.net)}</title><rect x="${pad.x - pad.width / 2}" y="${pad.y - pad.height / 2}" width="${pad.width}" height="${pad.height}" rx="${pad.rect ? 0 : pad.width / 2}" fill="#dfc58e"/><circle cx="${pad.x}" cy="${pad.y}" r="${pad.drill / 2}" fill="#142a24"/><text x="${pad.x}" y="${pad.y - 2}" font-size="1.25" text-anchor="middle" fill="#e8ede7">${escapeXml(pad.reference)}.${escapeXml(pad.number)}</text></g>`).join("");
    const label = !pads.length ? "尚无 KiCad 几何报告 · 不显示虚构铜线" : routes.length ? "实际管脚与铜线 · 以 DRC 报告为准" : "实际管脚位置 · 尚未布线";
    const spec = project.design.circuit;
    const values = `R1=${escapeXml(spec.resistance_ohm)} Ω · C1=${escapeXml(spec.capacitance_nf)} nF`;
    this.container.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="-8 -8 ${width + 16} ${height + 16}" aria-label="工程 PCB 几何预览"><g id="board-transform"><rect width="${width}" height="${height}" fill="#244b3b" stroke="#9da989" stroke-width=".2"/>${tracks}${padShapes}<text x="${width / 2}" y="${height - 7}" text-anchor="middle" font-size="1.5" fill="#d0dcd0">${values}</text><text x="${width / 2}" y="${height - 4}" text-anchor="middle" font-size="1.5" fill="#d0dcd0">${label}</text></g></svg>`;
    this.svg = this.container.querySelector("svg");
    this.updateTransform();
    this.updateVisibility();
  }

  setZoom(value) {
    if (!this.project) return;
    this.zoom = Math.min(3, Math.max(.5, value));
    this.updateTransform();
  }

  fit() {
    this.offset = { x: 0, y: 0 };
    this.setZoom(1);
  }

  setMode(mode) {
    this.mode = mode;
    this.container.classList.toggle("pan-mode", mode === "pan");
  }

  updateTransform() {
    if (!this.project) return;
    const centerX = this.project.constraints.width_mm / 2;
    const centerY = this.project.constraints.height_mm / 2;
    this.container.querySelector("#board-transform").setAttribute("transform", `translate(${centerX + this.offset.x} ${centerY + this.offset.y}) scale(${this.zoom}) translate(${-centerX} ${-centerY})`);
    document.getElementById("zoom-value").textContent = `${Math.round(this.zoom * 100)}%`;
  }

  updateVisibility() {
    this.container.querySelectorAll("[data-layer]").forEach((element) => { element.style.display = this.showNets && this.layers[element.dataset.layer] ? "" : "none"; });
    this.container.querySelectorAll(".silkscreen").forEach((element) => { element.style.display = this.layers.silk ? "" : "none"; });
  }

  select(reference) {
    this.selection = reference;
    this.container.querySelectorAll("[data-reference]").forEach((element) => { element.classList.toggle("selected", element.dataset.reference === reference); });
    this.container.querySelectorAll(".board-net").forEach((element) => {
      const connected = element.dataset.from === reference || element.dataset.to === reference;
      element.classList.toggle("highlighted", Boolean(reference) && connected);
      element.classList.toggle("dimmed", Boolean(reference) && !connected);
    });
    this.onSelect(this.project?.design.footprints.find((component) => component.reference === reference) ?? null);
  }
}
