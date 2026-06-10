// Generates 1024x1024 source PNG (lightning bolt design) then runs
// `cargo tauri icon` to produce all app icon sizes.
const fs   = require("fs");
const path = require("path");
const zlib = require("zlib");
const { execSync } = require("child_process");

const W = 1024, H = 1024;
const pixels = Buffer.alloc(W * H * 4);

// ── PNG writer ────────────────────────────────────────────────────────────────
function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return (~c) >>> 0;
}
function pngChunk(type, data) {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length, 0);
  const t   = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(Buffer.concat([t, data])), 0);
  return Buffer.concat([len, t, data, crc]);
}
function writePng(buf, w, h, outPath) {
  const raw = Buffer.alloc(h * (1 + w * 4));
  for (let y = 0; y < h; y++) {
    raw[y * (1 + w * 4)] = 0;
    buf.copy(raw, y * (1 + w * 4) + 1, y * w * 4, (y + 1) * w * 4);
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(h, 4);
  ihdr[8] = 8; ihdr[9] = 6;
  fs.writeFileSync(outPath, Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    pngChunk("IHDR", ihdr),
    pngChunk("IDAT", zlib.deflateSync(raw)),
    pngChunk("IEND", Buffer.alloc(0)),
  ]));
}

// ── Alpha blend into pixels ───────────────────────────────────────────────────
function blend(idx, r, g, b, a) {
  const sa = a / 255, da = pixels[idx + 3] / 255;
  const oa = sa + da * (1 - sa);
  if (oa < 0.001) return;
  pixels[idx]     = ((r * sa + pixels[idx]     * da * (1 - sa)) / oa) | 0;
  pixels[idx + 1] = ((g * sa + pixels[idx + 1] * da * (1 - sa)) / oa) | 0;
  pixels[idx + 2] = ((b * sa + pixels[idx + 2] * da * (1 - sa)) / oa) | 0;
  pixels[idx + 3] = (oa * 255) | 0;
}

// ── Scanline polygon fill ─────────────────────────────────────────────────────
function fillPoly(verts, colorFn) {
  const n    = verts.length;
  const minY = Math.max(0, Math.floor(Math.min(...verts.map(v => v[1]))));
  const maxY = Math.min(H - 1, Math.ceil(Math.max(...verts.map(v => v[1]))));
  for (let y = minY; y <= maxY; y++) {
    const yc = y + 0.5;
    const xs = [];
    for (let i = 0; i < n; i++) {
      const [x1, y1] = verts[i], [x2, y2] = verts[(i + 1) % n];
      if (y1 === y2) continue;
      if (yc >= Math.min(y1, y2) && yc < Math.max(y1, y2))
        xs.push(x1 + (yc - y1) / (y2 - y1) * (x2 - x1));
    }
    xs.sort((a, b) => a - b);
    for (let i = 0; i + 1 < xs.length; i += 2) {
      const x1 = Math.max(0, Math.ceil(xs[i]));
      const x2 = Math.min(W - 1, Math.floor(xs[i + 1]));
      for (let x = x1; x <= x2; x++)
        blend((y * W + x) * 4, ...colorFn(x, y));
    }
  }
}

// ── Background: dark navy #0c0e14 + warm radial glow ─────────────────────────
for (let i = 0; i < W * H; i++) {
  pixels[i * 4] = 12; pixels[i * 4 + 1] = 14;
  pixels[i * 4 + 2] = 20; pixels[i * 4 + 3] = 255;
}
const CX = W / 2, CY = H / 2;
for (let y = 0; y < H; y++) {
  for (let x = 0; x < W; x++) {
    const g = Math.max(0, 1 - Math.hypot(x - CX, y - CY) / (W * 0.52)) ** 2;
    const idx = (y * W + x) * 4;
    pixels[idx]     = Math.min(255, pixels[idx]     + (g * 40) | 0);
    pixels[idx + 1] = Math.min(255, pixels[idx + 1] + (g * 18) | 0);
  }
}

// ── Lightning bolt (Lucide "zap", 24×24 → centered 1024×1024) ─────────────────
const S = 42.5, OX = CX - 9.5 * S, OY = CY - 12 * S;
const bolt = [[11,2],[4,13],[9,13],[8,22],[15,11],[10,11]]
  .map(([x, y]) => [x * S + OX, y * S + OY]);

const boltMinY = Math.min(...bolt.map(v => v[1]));
const boltMaxY = Math.max(...bolt.map(v => v[1]));

// Glow halos (soft, expanding)
for (const [scale, alpha] of [[2.4, 5], [1.9, 10], [1.5, 18], [1.25, 30]]) {
  const g = bolt.map(([x, y]) => [(x - CX) * scale + CX, (y - CY) * scale + CY]);
  fillPoly(g, () => [249, 100, 15, alpha]);
}

// Main bolt — amber (#ffbe00) at top → deep orange (#d23a00) at bottom
fillPoly(bolt, (x, y) => {
  const t = (y - boltMinY) / (boltMaxY - boltMinY);
  return [
    (255 * (1 - t) + 210 * t) | 0,
    (190 * (1 - t) +  58 * t) | 0,
    0,
    255,
  ];
});

// Specular inner highlight (bright streak, top-left of bolt)
const hiScale = 0.28;
const hi = bolt.map(([x, y]) => [
  (x - CX) * hiScale + CX - 28,
  (y - CY) * hiScale + CY - 10,
]);
fillPoly(hi, () => [255, 245, 180, 70]);

// ── Write source PNG ──────────────────────────────────────────────────────────
const iconsDir = path.join(__dirname, "..", "src-tauri", "icons");
fs.mkdirSync(iconsDir, { recursive: true });
const src = path.join(iconsDir, "icon-source.png");
writePng(pixels, W, H, src);
console.log("wrote", src);

// ── Generate all Tauri icon sizes ─────────────────────────────────────────────
console.log("running cargo tauri icon ...");
execSync("cargo tauri icon src-tauri/icons/icon-source.png", {
  cwd: path.join(__dirname, ".."),
  stdio: "inherit",
  shell: true,
});
console.log("done");
