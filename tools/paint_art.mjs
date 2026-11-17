// Genera el arte pixel del juego hablando JSON-RPC con el MCP aseprite-asset-mcp.
// Uso: node tools/paint_art.mjs
import { spawn } from "node:child_process";
import fs from "node:fs";
import readline from "node:readline";

const MCP_SERVER =
  process.env.ASEPRITE_MCP ??
  "C:/Users/jdsal/Documents/Programming-personal/aseprite-asset-mcp/dist/interfaces/mcp-server.js";

const server = spawn(process.execPath, [MCP_SERVER], {
  cwd: process.cwd(),
  env: {
    ...process.env,
    ASEPRITE_PATH:
      process.env.ASEPRITE_PATH ??
      "C:/Program Files (x86)/Steam/steamapps/common/Aseprite/Aseprite.exe",
  },
  stdio: ["pipe", "pipe", "inherit"],
});
const lines = readline.createInterface({ input: server.stdout });
let nextId = 1;
const pending = new Map();
lines.on("line", (line) => {
  let message;
  try {
    message = JSON.parse(line);
  } catch {
    return;
  }
  if (message.id && pending.has(message.id)) {
    pending.get(message.id)(message);
    pending.delete(message.id);
  }
});

function send(method, params) {
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, (message) =>
      message.error ? reject(new Error(JSON.stringify(message.error))) : resolve(message.result),
    );
    server.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
  });
}

async function tool(name, args) {
  const result = await send("tools/call", { name, arguments: args });
  if (result?.isError) throw new Error(`${name}: ${result.content?.[0]?.text}`);
  return result;
}

// ---------- pincel ----------
const blank = (w, h) => Array.from({ length: h }, () => Array(w).fill(null));

function rect(grid, x, y, w, h, color) {
  for (let py = y; py < y + h; py++) {
    for (let px = x; px < x + w; px++) {
      if (grid[py]?.[px] !== undefined) grid[py][px] = color;
    }
  }
}

function pixel(grid, x, y, color) {
  if (grid[y]?.[x] !== undefined) grid[y][x] = color;
}

function line(grid, x0, y0, x1, y1, color, thickness = 1) {
  const dx = Math.abs(x1 - x0);
  const dy = Math.abs(y1 - y0);
  const sx = x0 < x1 ? 1 : -1;
  const sy = y0 < y1 ? 1 : -1;
  let error = dx - dy;
  let x = x0;
  let y = y0;
  for (;;) {
    rect(grid, x, y, thickness, thickness, color);
    if (x === x1 && y === y1) break;
    const doubled = error * 2;
    if (doubled > -dy) {
      error -= dy;
      x += sx;
    }
    if (doubled < dx) {
      error += dx;
      y += sy;
    }
  }
}

function rng(seed) {
  let state = seed >>> 0;
  return () => {
    state = (state * 1664525 + 1013904223) >>> 0;
    return state / 4294967296;
  };
}

function pixels(grid) {
  const list = [];
  grid.forEach((row, y) =>
    row.forEach((color, x) => {
      if (color) list.push({ x, y, color });
    }),
  );
  return list;
}

// ---------- personajes ----------
const palettes = {
  player: { outline: "#2a2320", tunic: "#c9b184", belt: "#6a5136", skin: "#e0a458", hair: "#8a7452", boots: "#4a3828", steel: "#cfd4dc", steelDark: "#8a919c", hood: "#7a6647", scarf: "#a53b3b" },
  bandit: { outline: "#241a18", tunic: "#8a3f3f", belt: "#4a2a24", skin: "#c98f4f", hair: "#2e2018", boots: "#3a2a22", steel: "#b9bec8", steelDark: "#7a808a", bandana: "#c25a4a" },
  guard: { outline: "#1f2430", tunic: "#3f5d8a", belt: "#2a3342", skin: "#d99a54", hair: "#9aa7b5", boots: "#2a3342", steel: "#d6dbe4", steelDark: "#8a94a3" },
  merchant: { outline: "#241f18", tunic: "#4f7a3a", belt: "#3a2a18", skin: "#e0a458", hair: "#3f5a2a", boots: "#3a2a18", steel: "#cfd4dc", steelDark: "#8a919c", cap: "#3f5a2a", capDark: "#2a3d1c" },
  civilian: { outline: "#2a2218", tunic: "#b08a5a", belt: "#6a5136", skin: "#d9a05a", hair: "#4a3828", boots: "#4a3828", steel: "#cfd4dc", steelDark: "#8a919c", straw: "#d9c27a", strawDark: "#b39a55" },
};

const legPoses = [
  [[-1, 1], [1, -1]],
  [[0, 0], [0, 0]],
  [[1, -1], [-1, 1]],
  [[0, 0], [0, 0]],
];

function headgear(g, hat, palette, top) {
  if (hat === "helmet") {
    rect(g, 9, 4 + top, 14, 6, "#8a97a8");
    rect(g, 9, 9 + top, 14, 1, "#5c6672");
    rect(g, 15, 10 + top, 2, 4, "#5c6672");
    rect(g, 10, 5 + top, 4, 2, "#a8b4c2");
  } else if (hat === "bandana") {
    rect(g, 10, 4 + top, 12, 3, palette.bandana);
    rect(g, 8, 6 + top, 4, 2, palette.bandana);
    rect(g, 7, 8 + top, 2, 3, palette.bandana);
  } else if (hat === "straw") {
    rect(g, 10, 4 + top, 12, 3, palette.straw);
    rect(g, 7, 7 + top, 18, 2, palette.straw);
    rect(g, 8, 9 + top, 16, 1, palette.strawDark);
  } else if (hat === "cap") {
    rect(g, 10, 4 + top, 12, 3, palette.cap);
    rect(g, 9, 7 + top, 14, 1, palette.capDark);
  } else {
    rect(g, 9, 3 + top, 14, 6, palette.hood);
    rect(g, 9, 8 + top, 2, 6, palette.hood);
    rect(g, 21, 8 + top, 2, 6, palette.hood);
    rect(g, 11, 9 + top, 8, 1, "#6a5638");
  }
}

function humanoid(palette, { bob = 0, leg = 0, weapon = "rest", hat = "hood", armed = "sword" }) {
  const g = blank(32, 32);
  const top = bob;

  // piernas
  const [left, right] = legPoses[leg];
  rect(g, 12 + left[0], 24 + left[1], 3, 7 - left[1], palette.boots);
  rect(g, 17 + right[0], 24 + right[1], 3, 7 - right[1], palette.boots);

  // torso
  rect(g, 10, 14 + top, 12, 11, palette.outline);
  rect(g, 11, 15 + top, 10, 9, palette.tunic);
  rect(g, 11, 21 + top, 10, 2, palette.belt);
  rect(g, 15, 13 + top, 3, 2, palette.skin);

  // cabeza
  rect(g, 10, 5 + top, 12, 10, palette.outline);
  rect(g, 11, 6 + top, 10, 8, palette.skin);
  rect(g, 11, 6 + top, 10, 2, palette.hair);
  pixel(g, 15, 10 + top, palette.outline);
  pixel(g, 19, 10 + top, palette.outline);
  headgear(g, hat, palette, top);
  if (palette.scarf) rect(g, 11, 13 + top, 10, 2, palette.scarf);

  // brazos
  rect(g, 8, 16 + top, 3, 8, palette.outline);
  rect(g, 8, 17 + top, 2, 6, palette.tunic);
  const armY = weapon === "up" ? 14 : 16;
  rect(g, 21, armY + top, 3, 7, palette.outline);
  rect(g, 22, armY + 1 + top, 2, 5, palette.tunic);

  // arma en la mano derecha
  const handX = 24;
  const handY = armY + 3 + top;
  const reach = armed === "dagger" ? 5 : 8;
  if (armed !== "none") {
    if (weapon === "up") {
      line(g, handX, handY, handX + reach - 4, handY - reach - 2, palette.steel, 2);
      line(g, handX, handY, handX, handY - 2, palette.steelDark);
    } else if (weapon === "strike") {
      line(g, handX, handY, handX + reach, handY, palette.steel, 2);
      line(g, handX, handY + 1, handX + reach - 1, handY + 1, palette.steelDark);
    } else if (weapon === "low") {
      line(g, handX, handY, handX + reach - 3, handY + reach - 2, palette.steel, 2);
    } else {
      line(g, handX, handY, handX + reach - 3, handY + reach - 3, palette.steel, 2);
    }
  } else {
    rect(g, handX, handY, 2, 2, palette.skin);
  }
  if (hat === "helmet") pixel(g, handX + 1, handY + 1, "#e6c060");

  return g;
}

function dog(step) {
  const fur = "#6b4a2f";
  const dark = "#3f2a19";
  const light = "#8a6340";
  const g = blank(32, 32);
  const bounce = step % 2 === 1 ? 1 : 0;

  line(g, 9, 18 - bounce, 4, 13 - bounce, fur, 2); // cola
  rect(g, 8, 16 - bounce, 17, 9, dark);
  rect(g, 9, 17 - bounce, 15, 7, fur);
  rect(g, 12, 18 - bounce, 8, 2, light);

  rect(g, 21, 12 - bounce, 10, 11, dark);
  rect(g, 22, 13 - bounce, 8, 9, fur);
  rect(g, 21, 10 - bounce, 3, 3, dark);
  rect(g, 27, 10 - bounce, 3, 3, dark);
  rect(g, 29, 17 - bounce, 3, 3, dark);
  rect(g, 30, 18 - bounce, 2, 1, fur);
  pixel(g, 27, 16 - bounce, dark);
  pixel(g, 28, 20 - bounce, dark);

  const legShift = step % 2 === 0 ? 0 : 1;
  rect(g, 9 + legShift, 24, 3, 5, dark);
  rect(g, 14 - legShift, 24, 3, 5, dark);
  rect(g, 19 + legShift, 24, 3, 5, dark);
  rect(g, 24 - legShift, 24, 3, 5, dark);

  return g;
}

// ---------- tiles y props ----------
function terrain(base, dark, light, seed, extra) {
  const g = blank(32, 32);
  const random = rng(seed);
  rect(g, 0, 0, 32, 32, base);
  for (let i = 0; i < 90; i++) {
    pixel(g, (random() * 32) | 0, (random() * 32) | 0, random() < 0.5 ? dark : light);
  }
  if (extra) extra(g, random);
  return g;
}

function tileGrid(kind) {
  if (kind === "sand") return terrain("#c2a15a", "#b3924f", "#cfae68", 11);
  if (kind === "dirt") return terrain("#8a6f47", "#7a5f3a", "#9a7f57", 23);
  if (kind === "road") {
    return terrain("#b08d5a", "#a07c4c", "#bf9c68", 31, (g, random) => {
      for (let i = 0; i < 10; i++) {
        rect(g, (random() * 28) | 0, (random() * 30) | 0, 3, 2, "#96744a");
      }
    });
  }
  if (kind === "rock") {
    return terrain("#4a4a55", "#3a3a44", "#5c5c68", 41, (g, random) => {
      for (let i = 0; i < 4; i++) {
        const x = (random() * 26) | 0;
        const y = (random() * 26) | 0;
        line(g, x, y, x + 5, y + 3, "#22222a");
      }
    });
  }
  return terrain("#3b6ea5", "#2f5a88", "#4a80b8", 53, (g) => {
    rect(g, 3, 7, 9, 1, "#6fa3d6");
    rect(g, 17, 13, 11, 1, "#6fa3d6");
    rect(g, 6, 22, 8, 1, "#2f5a88");
    rect(g, 20, 27, 7, 1, "#6fa3d6");
  });
}

function prop(kind) {
  const g = blank(32, 32);
  if (kind === "cactus") {
    rect(g, 13, 8, 7, 22, "#2f5a2a");
    rect(g, 14, 9, 5, 20, "#4f7a3a");
    rect(g, 7, 14, 6, 4, "#4f7a3a");
    rect(g, 7, 14, 4, 9, "#4f7a3a");
    rect(g, 22, 18, 5, 4, "#4f7a3a");
    rect(g, 23, 12, 4, 10, "#4f7a3a");
    for (let y = 10; y < 30; y += 3) {
      pixel(g, 13, y, "#8ab06a");
      pixel(g, 19, y + 1, "#8ab06a");
    }
  } else if (kind === "rock") {
    rect(g, 6, 16, 21, 12, "#3a3a44");
    rect(g, 8, 13, 16, 15, "#4a4a55");
    rect(g, 10, 14, 7, 5, "#5c5c68");
    pixel(g, 9, 17, "#6c6c78");
  } else if (kind === "tent") {
    for (let y = 6; y < 26; y++) {
      const half = Math.floor((y - 6) * 0.8) + 2;
      rect(g, 16 - half, y, half * 2, 1, y % 6 === 0 ? "#8a4a2a" : "#a0603a");
    }
    rect(g, 14, 18, 5, 8, "#3a2418");
    line(g, 16, 5, 16, 27, "#6a3a22");
  } else if (kind === "fire") {
    rect(g, 8, 25, 16, 3, "#5a3a22");
    rect(g, 6, 27, 20, 2, "#4a2e1a");
    rect(g, 12, 17, 8, 9, "#c25a2a");
    rect(g, 13, 13, 6, 9, "#e88a2a");
    rect(g, 15, 10, 3, 8, "#f5c542");
    pixel(g, 14, 15, "#fff0a0");
    pixel(g, 17, 18, "#fff0a0");
  } else if (kind === "crate") {
    rect(g, 7, 9, 18, 18, "#4a3422");
    rect(g, 8, 10, 16, 16, "#8a5f34");
    rect(g, 8, 15, 16, 2, "#6a4526");
    rect(g, 14, 10, 2, 16, "#6a4526");
    pixel(g, 9, 11, "#a87a48");
  } else if (kind === "well") {
    rect(g, 6, 18, 20, 9, "#3a3a44");
    rect(g, 7, 19, 18, 7, "#6a6a78");
    rect(g, 9, 20, 14, 5, "#2a3a4a");
    rect(g, 8, 8, 2, 10, "#6a4526");
    rect(g, 22, 8, 2, 10, "#6a4526");
    rect(g, 6, 7, 20, 2, "#8a5f34");
    line(g, 16, 9, 16, 18, "#3a2a18");
  }
  return g;
}

function item(kind) {
  const g = blank(16, 16);
  if (kind === "food") {
    rect(g, 3, 5, 9, 7, "#8a5f34");
    rect(g, 4, 6, 7, 5, "#b07a42");
    rect(g, 10, 7, 4, 3, "#e8e0d0");
    rect(g, 12, 8, 3, 1, "#f5f0e6");
  } else if (kind === "bandage") {
    rect(g, 3, 4, 10, 8, "#d8d8d0");
    rect(g, 4, 5, 8, 6, "#f0f0e8");
    rect(g, 7, 5, 2, 6, "#c8c8c0");
    rect(g, 5, 7, 6, 2, "#c8c8c0");
  } else if (kind === "coin") {
    rect(g, 4, 4, 8, 8, "#a87a20");
    rect(g, 5, 5, 6, 6, "#e6c060");
    rect(g, 6, 6, 3, 3, "#f5dc8a");
  } else if (kind === "sword") {
    line(g, 3, 12, 12, 3, "#8a919c", 2);
    line(g, 4, 12, 13, 3, "#cfd4dc");
    line(g, 2, 13, 5, 10, "#6a4526", 2);
  }
  return g;
}

// ---------- escritura ----------
async function writeSprite(source, output, grid) {
  await tool("create_canvas", { width: grid[0].length, height: grid.length, filename: source });
  const list = pixels(grid);
  if (list.length > 0) {
    await tool("draw_pixels_at", {
      filename: source,
      layer_name: "Layer 1",
      frame_index: 1,
      pixels: list,
    });
  }
  await tool("export_frame", { filename: source, output_filename: output, frame_index: 1, scale: 1 });
}

async function character(name, palette, { hat = "hood", armed = "sword", quadruped = false }) {
  const file = `.art/${name}.aseprite`;
  await tool("create_canvas", { width: 32, height: 32, filename: file });
  await tool("add_frames", { filename: file, count: 8, duration_ms: 120 });
  await tool("set_tag", { filename: file, name: "idle", from_frame: 1, to_frame: 2 });
  await tool("set_tag", { filename: file, name: "walk", from_frame: 3, to_frame: 6 });
  await tool("set_tag", { filename: file, name: "attack", from_frame: 7, to_frame: 9 });

  const frames = [];
  for (let index = 0; index < 9; index++) {
    if (quadruped) {
      frames.push(dog(index <= 1 ? index : index <= 5 ? index - 2 : index - 6));
    } else if (index <= 1) {
      frames.push(humanoid(palette, { bob: index === 1 ? -1 : 0, leg: 0, weapon: "rest", hat, armed }));
    } else if (index <= 5) {
      const step = index - 2;
      frames.push(
        humanoid(palette, {
          bob: step % 2 === 0 ? -1 : 0,
          leg: step,
          weapon: "rest",
          hat,
          armed,
        }),
      );
    } else {
      const step = index - 6;
      const weapon = step === 0 ? "up" : step === 1 ? "strike" : "low";
      frames.push(humanoid(palette, { bob: 0, leg: 0, weapon, hat, armed }));
    }
  }

  for (let index = 0; index < frames.length; index++) {
    const list = pixels(frames[index]);
    await tool("draw_pixels_at", {
      filename: file,
      layer_name: "Layer 1",
      frame_index: index + 1,
      pixels: list,
    });
  }
  for (const tag of ["idle", "walk", "attack"]) {
    await tool("export_spritesheet", {
      filename: file,
      output_filename: `assets/art/chars/${name}_${tag}.png`,
      scale: 1,
      padding: 0,
      tag_name: tag,
      sheet_type: "horizontal",
    });
  }
}

async function main() {
  await send("initialize", {
    protocolVersion: "2024-11-05",
    capabilities: {},
    clientInfo: { name: "paint-art", version: "1.0.0" },
  });
  server.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }) + "\n");

  for (const directory of ["assets/art/chars", "assets/art/tiles", "assets/art/props", "assets/art/items", ".art"]) {
    fs.mkdirSync(directory, { recursive: true });
  }

  for (const [name, palette, options] of [
    ["player", palettes.player, { hat: "hood", armed: "sword" }],
    ["bandit", palettes.bandit, { hat: "bandana", armed: "dagger" }],
    ["guard", palettes.guard, { hat: "helmet", armed: "sword" }],
    ["merchant", palettes.merchant, { hat: "cap", armed: "none" }],
    ["civilian", palettes.civilian, { hat: "straw", armed: "none" }],
    ["dog", palettes.bandit, { quadruped: true }],
  ]) {
    await character(name, palette, options);
  }

  for (const kind of ["sand", "dirt", "road", "rock", "water"]) {
    const file = `.art/tile-${kind}.aseprite`;
    await writeSprite(file, `assets/art/tiles/${kind}.png`, tileGrid(kind));
  }
  for (const kind of ["cactus", "rock", "tent", "fire", "crate", "well"]) {
    const file = `.art/prop-${kind}.aseprite`;
    await writeSprite(file, `assets/art/props/${kind}.png`, prop(kind));
  }
  for (const kind of ["food", "bandage", "coin", "sword"]) {
    const file = `.art/item-${kind}.aseprite`;
    await writeSprite(file, `assets/art/items/${kind}.png`, item(kind));
  }

  console.log("arte generado en assets/art");
  lines.close();
  server.stdin.end();
  server.kill();
}

await main();



