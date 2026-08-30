/* kombi grid editor — vanilla JS.
 *
 * Produces the interchange format consumed by kombilib/src/main.rs:
 *
 *   // id len dir (L R U D)
 *   1 6 d
 *   ---
 *   // cross: id_a id_b c_a c_b
 *   1 4  5 2
 *
 * Cell indices are 1-based: cell 1 of a box is its starting cell.
 */

const PDFJS_URL = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/4.0.379/pdf.min.mjs';
const PDFJS_WORKER = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/4.0.379/pdf.worker.min.mjs';

const $ = (id) => document.getElementById(id);
const cv = $('cv');
const ctx = cv.getContext('2d');

const state = {
  bitmap: null,          // ImageBitmap | HTMLCanvasElement of the page/screenshot
  pdf: null,             // pdf.js document, when a PDF is loaded
  grid: { cell: 30, x0: 0, y0: 0 },
  zoom: 1,
  showGrid: true,
  mode: 'grid',
  boxes: [],             // { id, col, row, dir, len }
  nextId: 1,
  selected: null,        // box id
  drag: null,            // in-flight interaction
};

/* ---------------------------------------------------------------- geometry */

const DIRS = { r: [1, 0], l: [-1, 0], d: [0, 1], u: [0, -1] };

/** All cells a box occupies, in index order (index 0 === cell 1). */
function cellsOf(box) {
  const [dx, dy] = DIRS[box.dir];
  const out = [];
  for (let i = 0; i < box.len; i++) out.push([box.col + dx * i, box.row + dy * i]);
  return out;
}

/** Image-space point -> grid cell (floor, so it works for negative coords too). */
function pointToCell(x, y) {
  const g = state.grid;
  return [Math.floor((x - g.x0) / g.cell), Math.floor((y - g.y0) / g.cell)];
}

/** Mouse event -> image-space coordinates, undoing the CSS zoom. */
function eventToImage(e) {
  const r = cv.getBoundingClientRect();
  return [(e.clientX - r.left) / state.zoom, (e.clientY - r.top) / state.zoom];
}

/**
 * Turn a drag from one cell to another into a box definition.
 *
 * A hand-drawn drag is never perfectly straight, so this decides:
 *  - which axis the user meant (dominant delta),
 *  - the resulting direction, and
 *  - the length in cells (inclusive of both endpoints).
 *
 * Returns null when the drag is too short to be a box.
 */
function resolveDrag(startCell, endCell) {
  const dx = endCell[0] - startCell[0];
  const dy = endCell[1] - startCell[1];

  // Dominant axis wins; ties (a diagonal drag) fall back to horizontal.
  const horizontal = Math.abs(dx) >= Math.abs(dy);
  const delta = horizontal ? dx : dy;
  if (Math.abs(delta) < 1) return null;

  const dir = horizontal ? (delta > 0 ? 'r' : 'l') : (delta > 0 ? 'd' : 'u');
  return { col: startCell[0], row: startCell[1], dir, len: Math.abs(delta) + 1 };
}

/* ------------------------------------------------------------- calibration */

/**
 * Derive cell size and origin from a rectangle the user dragged corner-to-corner
 * across `cols` × `rows` whole cells.
 *
 * The clicked corner becomes the origin, so the grid lines land exactly on the
 * puzzle borders instead of being nudged into place by hand.
 */
function applyCalibration(rect, cols, rows) {
  const cw = rect.w / cols;
  const ch = rect.h / rows;
  state.grid.cell = (cw + ch) / 2;
  state.grid.x0 = rect.x;
  state.grid.y0 = rect.y;

  // Cells are square in a crossword; a big width/height split means the span
  // counts are wrong (or the scan is skewed), so surface it rather than hide it.
  const skew = Math.abs(cw - ch) / Math.max(cw, ch);
  $('calResult').textContent =
    `cell ≈ ${round(state.grid.cell)}px  (w ${round(cw)} × h ${round(ch)})` +
    (skew > 0.05 ? `  ⚠ ${Math.round(skew * 100)}% off-square — check cols/rows` : '');
}

/** Best guess at how many cells a dragged rectangle spans, using the current cell size. */
function guessSpan(rect) {
  return [
    Math.max(1, Math.round(rect.w / state.grid.cell)),
    Math.max(1, Math.round(rect.h / state.grid.cell)),
  ];
}

/* --------------------------------------------------------------- crossings */

/**
 * Derive crossings from grid overlaps: any cell occupied by two boxes is a
 * crossing, reported as `id_a id_b c_a c_b` with 1-based cell indices.
 */
function deriveCrossings() {
  const occ = new Map(); // "col,row" -> [{ id, idx }]
  for (const b of state.boxes) {
    cellsOf(b).forEach(([c, r], i) => {
      const k = `${c},${r}`;
      if (!occ.has(k)) occ.set(k, []);
      occ.get(k).push({ id: b.id, idx: i + 1 });
    });
  }

  const out = [];
  for (const hits of occ.values()) {
    if (hits.length < 2) continue;
    for (let i = 0; i < hits.length; i++) {
      for (let j = i + 1; j < hits.length; j++) {
        const [a, b] = hits[i].id <= hits[j].id ? [hits[i], hits[j]] : [hits[j], hits[i]];
        out.push({ id_a: a.id, id_b: b.id, c_a: a.idx, c_b: b.idx });
      }
    }
  }
  out.sort((p, q) => p.id_a - q.id_a || p.id_b - q.id_b);
  return out;
}

/* ------------------------------------------------------------------ output */

function serialize() {
  const boxes = [...state.boxes].sort((a, b) => a.id - b.id);
  const lines = ['// id len dir (L R U D)'];
  for (const b of boxes) lines.push(`${b.id} ${b.len} ${b.dir}`);
  lines.push('---');
  lines.push('// cross: id_a id_b c_a c_b');
  if ($('autoCross').checked) {
    for (const c of deriveCrossings()) {
      lines.push(`${c.id_a} ${c.id_b} ${c.c_a} ${c.c_b}`);
    }
  }
  return lines.join('\n') + '\n';
}

/* ------------------------------------------------------------------ render */

function render() {
  const bmp = state.bitmap;
  const w = bmp ? bmp.width : 900;
  const h = bmp ? bmp.height : 600;

  if (cv.width !== w || cv.height !== h) { cv.width = w; cv.height = h; }
  cv.style.width = `${w * state.zoom}px`;
  cv.style.height = `${h * state.zoom}px`;

  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = '#fff';
  ctx.fillRect(0, 0, w, h);
  if (bmp) ctx.drawImage(bmp, 0, 0);

  const g = state.grid;
  if (state.showGrid && g.cell > 3) {
    ctx.save();
    ctx.strokeStyle = 'rgba(0,120,255,0.35)';
    ctx.lineWidth = 1 / state.zoom;
    ctx.beginPath();
    for (let x = g.x0 % g.cell; x < w; x += g.cell) { ctx.moveTo(x, 0); ctx.lineTo(x, h); }
    for (let y = g.y0 % g.cell; y < h; y += g.cell) { ctx.moveTo(0, y); ctx.lineTo(w, y); }
    ctx.stroke();
    ctx.restore();
  }

  for (const b of state.boxes) drawBox(b, b.id === state.selected);
  if (state.drag && state.drag.kind === 'draw' && state.drag.preview) {
    drawBox({ ...state.drag.preview, id: '?' }, true, true);
  }
  if (state.drag && state.drag.kind === 'calib' && state.drag.rect) {
    const r = state.drag.rect;
    ctx.save();
    ctx.strokeStyle = '#f6a96b';
    ctx.lineWidth = 2 / state.zoom;
    ctx.setLineDash([6 / state.zoom, 4 / state.zoom]);
    ctx.strokeRect(r.x, r.y, r.w, r.h);
    ctx.restore();
  }

  // Mark derived crossings so mistakes are visible at a glance.
  ctx.save();
  ctx.fillStyle = 'rgba(255,80,80,0.55)';
  const seen = new Set();
  for (const c of deriveCrossings()) {
    const box = state.boxes.find((b) => b.id === c.id_a);
    if (!box) continue;
    const [col, row] = cellsOf(box)[c.c_a - 1];
    const k = `${col},${row}`;
    if (seen.has(k)) continue;
    seen.add(k);
    const [x, y] = [g.x0 + col * g.cell, g.y0 + row * g.cell];
    ctx.beginPath();
    ctx.arc(x + g.cell / 2, y + g.cell / 2, Math.max(2, g.cell * 0.14), 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.restore();
}

function drawBox(box, selected, preview = false) {
  const g = state.grid;
  ctx.save();
  ctx.fillStyle = preview ? 'rgba(110,231,168,0.35)'
    : selected ? 'rgba(110,231,168,0.45)' : 'rgba(60,160,255,0.28)';
  ctx.strokeStyle = selected || preview ? '#2fbd77' : '#1e6fd9';
  ctx.lineWidth = 2 / state.zoom;

  const cells = cellsOf(box);
  for (const [c, r] of cells) {
    const x = g.x0 + c * g.cell, y = g.y0 + r * g.cell;
    ctx.fillRect(x, y, g.cell, g.cell);
    ctx.strokeRect(x, y, g.cell, g.cell);
  }

  // id label on the starting cell
  const [c0, r0] = cells[0];
  const x = g.x0 + c0 * g.cell, y = g.y0 + r0 * g.cell;
  ctx.fillStyle = '#0b2b1c';
  ctx.font = `bold ${Math.max(9, g.cell * 0.5)}px ui-sans-serif, sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(String(box.id), x + g.cell / 2, y + g.cell / 2);
  ctx.restore();
}

/* ------------------------------------------------------------------- panel */

function syncPanel() {
  $('cell').value = round(state.grid.cell);
  $('x0').value = round(state.grid.x0);
  $('y0').value = round(state.grid.y0);
  $('boxCount').textContent = `(${state.boxes.length})`;

  const list = $('boxList');
  list.innerHTML = '';
  for (const b of [...state.boxes].sort((a, z) => a.id - z.id)) {
    const row = document.createElement('div');
    row.className = 'boxRow' + (b.id === state.selected ? ' sel' : '');

    const id = document.createElement('input');
    id.type = 'number'; id.value = b.id; id.title = 'id';
    id.onchange = () => { b.id = parseInt(id.value, 10) || b.id; refresh(); };

    const len = document.createElement('input');
    len.type = 'number'; len.min = 1; len.value = b.len; len.title = 'length';
    len.onchange = () => { b.len = Math.max(1, parseInt(len.value, 10) || 1); refresh(); };

    const dir = document.createElement('span');
    dir.className = 'dir'; dir.textContent = b.dir;

    const del = document.createElement('button');
    del.className = 'del'; del.textContent = '✕';
    del.onclick = () => { state.boxes = state.boxes.filter((x) => x !== b); refresh(); };

    row.append(id, len, dir, del);
    row.onclick = (e) => {
      if (e.target === id || e.target === len || e.target === del) return;
      state.selected = b.id; refresh();
    };
    list.append(row);
  }

  const crossings = deriveCrossings();
  const dupes = new Set();
  for (const b of state.boxes) {
    if (state.boxes.filter((x) => x.id === b.id).length > 1) dupes.add(b.id);
  }
  $('crossNote').textContent = dupes.size
    ? `⚠ duplicate ids: ${[...dupes].join(', ')}`
    : `${crossings.length} crossing${crossings.length === 1 ? '' : 's'} derived`;

  $('out').value = serialize();
}

const round = (n) => Math.round(n * 100) / 100;
function refresh() { render(); syncPanel(); save(); }

/* -------------------------------------------------------------- interaction */

cv.addEventListener('mousedown', (e) => {
  if (!state.bitmap) return;
  const [x, y] = eventToImage(e);

  if (state.mode === 'grid') {
    state.drag = { kind: 'grid', x, y, x0: state.grid.x0, y0: state.grid.y0 };
  } else if (state.mode === 'draw') {
    state.drag = { kind: 'draw', start: pointToCell(x, y), preview: null };
  } else if (state.mode === 'calibrate') {
    state.drag = { kind: 'calib', x, y, rect: null };
  } else {
    const cell = pointToCell(x, y);
    const hit = state.boxes.find((b) =>
      cellsOf(b).some(([c, r]) => c === cell[0] && r === cell[1]));
    state.selected = hit ? hit.id : null;
    refresh();
  }
  e.preventDefault();
});

window.addEventListener('mousemove', (e) => {
  const d = state.drag;
  if (!d) return;
  const [x, y] = eventToImage(e);

  if (d.kind === 'grid') {
    state.grid.x0 = d.x0 + (x - d.x);
    state.grid.y0 = d.y0 + (y - d.y);
    render(); syncPanel();
  } else if (d.kind === 'draw') {
    const box = resolveDrag(d.start, pointToCell(x, y));
    d.preview = box || { col: d.start[0], row: d.start[1], dir: 'r', len: 1 };
    render();
  } else if (d.kind === 'calib') {
    d.rect = { x: Math.min(d.x, x), y: Math.min(d.y, y), w: Math.abs(x - d.x), h: Math.abs(y - d.y) };
    render();
  }
});

window.addEventListener('mouseup', () => {
  const d = state.drag;
  state.drag = null;
  if (!d) return;
  if (d.kind === 'draw' && d.preview && d.preview.len > 1) {
    const box = { id: state.nextId++, ...d.preview };
    state.boxes.push(box);
    state.selected = box.id;
  }
  if (d.kind === 'calib' && d.rect && d.rect.w > 4 && d.rect.h > 4) {
    state.calibRect = d.rect;
    const [cols, rows] = guessSpan(d.rect);
    $('calCols').value = cols;
    $('calRows').value = rows;
    applyCalibration(d.rect, cols, rows);
  }
  refresh();
});

cv.addEventListener('wheel', (e) => {
  if (state.mode !== 'grid' || !state.bitmap) return;
  e.preventDefault();
  // Resize cells around the cursor so the grid stays anchored where you point.
  const [x, y] = eventToImage(e);
  const g = state.grid;
  const factor = e.deltaY < 0 ? 1.02 : 1 / 1.02;
  const next = Math.max(4, g.cell * factor);
  g.x0 = x - (x - g.x0) * (next / g.cell);
  g.y0 = y - (y - g.y0) * (next / g.cell);
  g.cell = next;
  render(); syncPanel(); save();
}, { passive: false });

window.addEventListener('keydown', (e) => {
  if (['INPUT', 'TEXTAREA'].includes(e.target.tagName)) return;

  if ((e.key === 'Delete' || e.key === 'Backspace') && state.selected != null) {
    state.boxes = state.boxes.filter((b) => b.id !== state.selected);
    state.selected = null;
    refresh();
    return;
  }
  // Arrows nudge the grid origin (shift = whole cell).
  const step = e.shiftKey ? state.grid.cell : 1;
  const nudge = { ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] }[e.key];
  if (nudge) {
    e.preventDefault();
    state.grid.x0 += nudge[0];
    state.grid.y0 += nudge[1];
    refresh();
  }
});

/* ---------------------------------------------------------------- controls */

for (const r of document.querySelectorAll('input[name=mode]')) {
  r.onchange = () => {
    state.mode = r.value;
    cv.style.cursor = state.mode === 'grid' ? 'move' : 'crosshair';
    $('calibBox').hidden = state.mode !== 'calibrate';
  };
}

// Re-derive the grid whenever the span counts change, without re-dragging.
for (const el of [$('calCols'), $('calRows')]) {
  el.oninput = () => {
    if (!state.calibRect) return;
    const cols = Math.max(1, parseInt($('calCols').value, 10) || 1);
    const rows = Math.max(1, parseInt($('calRows').value, 10) || 1);
    applyCalibration(state.calibRect, cols, rows);
    refresh();
  };
}
$('cell').oninput = (e) => { state.grid.cell = Math.max(4, +e.target.value || 4); render(); save(); };
$('x0').oninput = (e) => { state.grid.x0 = +e.target.value || 0; render(); save(); };
$('y0').oninput = (e) => { state.grid.y0 = +e.target.value || 0; render(); save(); };
$('zoom').oninput = (e) => { state.zoom = +e.target.value; render(); };
$('showGrid').onchange = (e) => { state.showGrid = e.target.checked; render(); };

$('renumber').onclick = () => {
  // Reading order: top-to-bottom, then left-to-right, like a printed puzzle.
  const sorted = [...state.boxes].sort((a, b) => a.row - b.row || a.col - b.col);
  sorted.forEach((b, i) => { b.id = i + 1; });
  state.nextId = sorted.length + 1;
  state.selected = null;
  refresh();
};

$('clear').onclick = () => {
  if (state.boxes.length && !confirm('Delete all boxes?')) return;
  state.boxes = []; state.nextId = 1; state.selected = null;
  refresh();
};

$('autoCross').onchange = refresh;

$('copy').onclick = async () => {
  try {
    await navigator.clipboard.writeText(serialize());
    flash('copied to clipboard');
  } catch {
    $('out').removeAttribute('readonly');
    $('out').select();
    flash('press ⌘/Ctrl+C to copy');
  }
};

$('download').onclick = () => {
  const url = URL.createObjectURL(new Blob([serialize()], { type: 'text/plain' }));
  const a = document.createElement('a');
  a.href = url; a.download = 'input';
  a.click();
  URL.revokeObjectURL(url);
};

function flash(msg) {
  $('status').textContent = msg;
  clearTimeout(flash.t);
  flash.t = setTimeout(() => { $('status').textContent = state.fileName || ''; }, 2000);
}

/* ------------------------------------------------------------ file loading */

$('file').onchange = (e) => e.target.files[0] && loadFile(e.target.files[0]);

document.addEventListener('dragover', (e) => e.preventDefault());
document.addEventListener('drop', (e) => {
  e.preventDefault();
  if (e.dataTransfer.files[0]) loadFile(e.dataTransfer.files[0]);
});

async function loadFile(file) {
  state.fileName = file.name;
  $('status').textContent = `loading ${file.name}…`;
  try {
    if (file.type === 'application/pdf' || /\.pdf$/i.test(file.name)) {
      await loadPdf(file);
    } else {
      state.pdf = null;
      $('pdfPages').hidden = true;
      state.bitmap = await createImageBitmap(file);
    }
    $('hint').hidden = true;
    $('status').textContent = `${file.name} — ${state.bitmap.width}×${state.bitmap.height}`;
    fitZoom();
    refresh();
  } catch (err) {
    $('status').textContent = `failed: ${err.message}`;
  }
}

async function loadPdf(file) {
  const pdfjs = await import(PDFJS_URL);
  pdfjs.GlobalWorkerOptions.workerSrc = PDFJS_WORKER;
  state.pdf = await pdfjs.getDocument({ data: await file.arrayBuffer() }).promise;
  $('pdfCount').textContent = state.pdf.numPages;
  $('pdfPage').max = state.pdf.numPages;
  $('pdfPage').value = 1;
  $('pdfPages').hidden = false;
  await renderPdfPage(1);
}

async function renderPdfPage(n) {
  const page = await state.pdf.getPage(n);
  // Render at 2x so the grid can be aligned precisely on thin puzzle borders.
  const viewport = page.getViewport({ scale: 2 });
  const off = document.createElement('canvas');
  off.width = viewport.width;
  off.height = viewport.height;
  await page.render({ canvasContext: off.getContext('2d'), viewport }).promise;
  state.bitmap = off;
}

$('pdfPage').onchange = async (e) => {
  if (!state.pdf) return;
  const n = Math.min(state.pdf.numPages, Math.max(1, +e.target.value || 1));
  e.target.value = n;
  await renderPdfPage(n);
  refresh();
};

function fitZoom() {
  const avail = $('stage').clientWidth - 40;
  state.zoom = Math.min(1, avail / state.bitmap.width);
  $('zoom').value = state.zoom;
}

/* ----------------------------------------------------------- session state */

const KEY = 'kombi-editor';

function save() {
  try {
    localStorage.setItem(KEY, JSON.stringify({
      grid: state.grid, boxes: state.boxes, nextId: state.nextId, calibRect: state.calibRect,
    }));
  } catch { /* quota — not worth failing the edit over */ }
}

function restore() {
  try {
    const s = JSON.parse(localStorage.getItem(KEY) || 'null');
    if (!s) return;
    Object.assign(state.grid, s.grid || {});
    state.boxes = s.boxes || [];
    state.nextId = s.nextId || state.boxes.length + 1;
    state.calibRect = s.calibRect || null;
  } catch { /* ignore corrupt state */ }
}

restore();
refresh();
