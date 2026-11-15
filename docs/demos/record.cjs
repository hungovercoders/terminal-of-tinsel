// Records one demo: plays a .tape into a real shell and draws what it printed.
//
//   node record.cjs <name.tape> <frames-dir>
//
// The tape is played by pty_record.py, which writes down the terminal's
// output with its timing. This script then replays that output into xterm.js
// in a headless Chromium and takes a screenshot for every frame, so the
// frames are drawn at an exact rate however slow the machine is. xterm.js is
// given current Unicode widths, which is what makes every emoji two cells
// wide and keeps every box border in line.
//
// It also writes crop.txt, the size of the part of the last frame that has
// anything on it, for the stills.

const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const { chromium } = require("playwright");

const FPS = 15;
// The margin of background around the terminal, on top of the tape's Padding.
const MARGIN = 24;

function settings(tape) {
  const set = { FontSize: 14, Width: 1000, Height: 600, Padding: 24 };
  for (const line of fs.readFileSync(tape, "utf8").split("\n")) {
    const found = line.trim().match(/^Set (FontSize|Width|Height|Padding) (\d+)$/);
    if (found) set[found[1]] = Number(found[2]);
  }
  return set;
}

// VHS themes say purple where xterm.js says magenta.
function xtermTheme(theme) {
  const out = {};
  for (const [name, colour] of Object.entries(theme)) {
    if (name === "name") continue;
    if (name === "selection") out.selectionBackground = colour;
    else out[name.replace("Purple", "Magenta").replace("purple", "magenta")] = colour;
  }
  return out;
}

async function main() {
  const [tape, frames] = process.argv.slice(2);
  if (!tape || !frames) {
    console.error("usage: node record.cjs <name.tape> <frames-dir>");
    process.exit(2);
  }
  const here = __dirname;
  // xterm.js is in the repository's own package.json, two folders up.
  const modules = path.join(here, "..", "..", "node_modules");
  const set = settings(tape);
  const theme = JSON.parse(fs.readFileSync(path.join(here, "theme.json"), "utf8"));
  const pad = set.Padding + MARGIN;
  const width = set.Width + 2 * MARGIN;
  const height = set.Height + 2 * MARGIN;

  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
  await page.setContent(
    `<!doctype html><meta charset="utf-8"><style>
      html, body { margin: 0; height: 100%; background: ${theme.background}; overflow: hidden; }
      #terminal { position: absolute; inset: ${pad}px; }
    </style><div id="terminal"></div>`
  );
  await page.addStyleTag({ path: path.join(modules, "@xterm/xterm/css/xterm.css") });
  await page.addScriptTag({ path: path.join(modules, "@xterm/xterm/lib/xterm.js") });
  await page.addScriptTag({
    path: path.join(modules, "@xterm/addon-unicode-graphemes/lib/addon-unicode-graphemes.js"),
  });
  await page.addScriptTag({ path: path.join(modules, "@xterm/addon-fit/lib/addon-fit.js") });

  const size = await page.evaluate(
    async ({ fontSize, theme }) => {
      await document.fonts.load(`${fontSize}px "JetBrains Mono"`);
      const term = new Terminal({
        fontFamily: '"JetBrains Mono", "DejaVu Sans Mono", monospace',
        fontSize,
        theme,
        cursorBlink: false,
        allowProposedApi: true,
        scrollback: 0,
      });
      term.loadAddon(new UnicodeGraphemesAddon.UnicodeGraphemesAddon());
      term.unicode.activeVersion = "15-graphemes";
      const fit = new FitAddon.FitAddon();
      term.loadAddon(fit);
      term.open(document.getElementById("terminal"));
      fit.fit();
      window.term = term;
      return { cols: term.cols, rows: term.rows };
    },
    { fontSize: set.FontSize, theme: xtermTheme(theme) }
  );

  // Play the tape for real, at the size the page worked out.
  fs.mkdirSync(frames, { recursive: true });
  const cast = path.join(frames, "cast.json");
  execFileSync("python3", [path.join(here, "pty_record.py"), tape, size.cols, size.rows, cast], {
    stdio: "inherit",
  });
  const { leadIn, duration, chunks } = JSON.parse(fs.readFileSync(cast, "utf8"));
  fs.rmSync(cast);

  // Replay it. Frames start once the prompt is up, and each one holds
  // everything printed by its moment.
  let next = 0;
  let frame = 0;
  for (let t = leadIn; t <= duration; t += 1 / FPS) {
    const due = [];
    while (next < chunks.length && chunks[next].t <= t) due.push(chunks[next++].data);
    if (due.length > 0) {
      await page.evaluate(async (parts) => {
        for (const part of parts) {
          const bytes = Uint8Array.from(atob(part), (ch) => ch.charCodeAt(0));
          await new Promise((done) => window.term.write(bytes, done));
        }
      }, due);
    }
    frame += 1;
    await page.screenshot({
      path: path.join(frames, `frame-${String(frame).padStart(5, "0")}.png`),
    });
  }
  // How much of the screen the output reached, with the margin put back.
  const used = await page.evaluate(() => {
    const buffer = window.term.buffer.active;
    let cols = 0;
    let rows = 0;
    for (let y = 0; y < window.term.rows; y++) {
      const text = buffer.getLine(y).translateToString(true);
      if (text.length > 0) {
        rows = y + 1;
        // Cells, not characters: a wide character takes two.
        let cells = 0;
        const line = buffer.getLine(y);
        for (let x = 0; x < window.term.cols; x++) {
          if (line.getCell(x).getChars() !== "") cells = x + line.getCell(x).getWidth();
        }
        cols = Math.max(cols, cells);
      }
    }
    const screen = document.querySelector(".xterm-screen");
    return {
      width: Math.ceil((cols * screen.clientWidth) / window.term.cols),
      height: Math.ceil((rows * screen.clientHeight) / window.term.rows),
    };
  });
  fs.writeFileSync(
    path.join(frames, "crop.txt"),
    `${Math.min(width, used.width + 2 * pad)}:${Math.min(height, used.height + 2 * pad)}:0:0\n`
  );
  await browser.close();
  console.log(`   ${frame} frames, ${size.cols}x${size.rows} cells`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
