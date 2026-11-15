#!/usr/bin/env bash
# Renders the README's demo GIFs and screenshots from docs/demos/*.tape.
#
#   docs/demos/render.sh            # everything
#   docs/demos/render.sh hero quiz  # just these
#
# Needs: cargo, python3, node with Playwright and its Chromium, ffmpeg, the
# JetBrains Mono font, and Linux with Noto Color Emoji
# (fonts-noto-color-emoji) as fontconfig's emoji font. The first run installs
# xterm.js from the repository's package.json.
#
# The tapes are written in the language of VHS (https://github.com/charmbracelet/vhs),
# but they are not played by it. VHS draws through ttyd, and ttyd older than
# 1.7.7 draws emoji one cell wide, which knocks every box border out of line.
# So pty_record.py plays a tape into a real shell and notes down the output,
# and record.cjs replays that into xterm.js with current Unicode widths and takes
# the frames. Only the commands the tapes use are understood.
#
# Recordings use a fixed TERMINAL_OF_TINSEL_SEED (DEMO_SEED, default 25), the
# tapes that take a date pass one, and every tape starts from the same saved
# stocking, so re-recording gives the same questions, portraits and showdowns.
# Two things still follow the real clock: the hero tape's intro (its
# greeting and notes), and any badge a game earns for the night it's played.
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"
demos=docs/demos
assets=docs/assets
work=$(mktemp -d)
# Frames are written under docs/demos/.frames; clear them however this ends.
trap 'rm -rf "$work" "$root/$demos/.frames"' EXIT

# Stills keep only their final frame, as a PNG.
stills=" list random countdown tonight advent stocking "

for tool in cargo python3 node npm ffmpeg fc-match; do
  command -v "$tool" > /dev/null || { echo "missing $tool" >&2; exit 1; }
done
# The recordings draw emoji with the system's font. Noto's licence (OFL 1.1)
# lets its glyphs ship in the GIFs; Apple's and Microsoft's emoji don't. On
# Linux the browser picks emoji through fontconfig, so ask it which font it
# would use.
[ "$(uname -s)" = Linux ] || { echo "record on Linux: other systems draw emoji that can't be redistributed" >&2; exit 1; }
emoji_font=$(fc-match -f '%{family}' 'emoji:charset=1f384')
[ "$emoji_font" = "Noto Color Emoji" ] || {
  echo "emoji would be drawn with '$emoji_font': install fonts-noto-color-emoji and make it the preferred emoji font" >&2
  exit 1
}
[ -n "$(fc-list 'JetBrains Mono')" ] || { echo "missing the JetBrains Mono font" >&2; exit 1; }
# xterm.js, the terminal the frames are drawn with. Playwright isn't in
# package.json: it comes with a browser, so use the one already installed.
[ -d node_modules/@xterm/xterm ] || npm install --no-audit --no-fund --silent
node -e 'require("playwright")' 2> /dev/null || { echo "missing Playwright: npm install -g playwright && npx playwright install chromium" >&2; exit 1; }

# The recorded shell gets a fake home, so saved paths read ~/.config/...
config="$work/home/.config/terminal-of-tinsel"
mkdir -p "$work/bin" "$config"

# The recorder starts "bash" from PATH; this one has a tree for a prompt.
real_bash=$(command -v bash)
printf '%s\n' "PS1='\\[\\e[38;2;82;183;136m\\]🎄 \\[\\e[0m\\]'" \
  "export HOME='$work/home' TERMINAL_OF_TINSEL_HOME='$config'" \
  "export TERMINAL_OF_TINSEL_SEED='${DEMO_SEED:-25}'" > "$work/rc"
printf '#!%s\nexec "%s" --noprofile --rcfile "%s" -i +o history\n' \
  "$real_bash" "$real_bash" "$work/rc" > "$work/bin/bash"
chmod +x "$work/bin/bash"

cargo build --release --quiet
cp target/release/terminal-of-tinsel "$work/bin/"

# A stocking with some history, so the screenshot has something to show. It
# is put back before every tape: the games save to it as they are played.
seed_stocking() {
cat > "$config/stocking.json" <<'JSON'
{
  "version": 1,
  "knowledge": {"krampus": 3, "yule-cat": 3, "marley": 3, "tomte": 3, "gryla": 2, "mari-lwyd": 1, "befana": 2},
  "captured": {
    "krampus": "2026-12-05T21:13:00",
    "yule-cat": "2026-12-07T22:40:00",
    "marley": "2026-12-09T23:05:00",
    "tomte": "2026-12-12T20:31:00"
  },
  "badges": {
    "first-cracker": "2026-12-05T21:13:00",
    "stocking-filler": "2026-12-05T21:13:00",
    "krampusnacht": "2026-12-05T21:13:00",
    "frost-reader": "2026-12-06T20:02:00",
    "snowball-referee": "2026-12-08T19:44:00",
    "matinee-regular": "2026-12-09T23:05:00"
  },
  "seen": {"krampus": true, "yule-cat": true, "marley": true, "tomte": true, "gryla": true},
  "quizzesPlayed": 9,
  "bestQuizScore": 90,
  "guessesPlayed": 4,
  "bestGuessScore": 420,
  "mashesPlayed": 6
}
JSON
}

mkdir -p "$assets"
export PATH="$work/bin:$PATH"

names=("$@")
if [ ${#names[@]} -eq 0 ]; then
  for tape in "$demos"/*.tape; do names+=("$(basename "$tape" .tape)"); done
fi

for name in "${names[@]}"; do
  echo "🎬 $name"
  seed_stocking
  frames="$demos/.frames/$name"
  rm -rf "$frames"
  node "$demos/record.cjs" "$demos/$name.tape" "$frames"
  [ -e "$frames/frame-00001.png" ] || { echo "no frames for $name" >&2; exit 1; }
  if [[ "$stills" == *" $name "* ]]; then
    last=$(find "$frames" -name 'frame-*.png' | sort | tail -1)
    # Keep the part of the screen the output reached.
    ffmpeg -loglevel error -y -i "$last" -vf "crop=$(cat "$frames/crop.txt")" "$assets/$name.png"
    echo "   → $assets/$name.png"
  else
    # record.cjs draws 15 frames a second. The palette is built from whole
    # frames and nothing is dithered: a palette built only from what changes
    # between frames loses the colours of text that sits still, and dither
    # speckles flat terminal colours.
    ffmpeg -loglevel error -y -framerate 15 -start_number 1 -i "$frames/frame-%05d.png" -filter_complex \
      "split[a][b];[a]palettegen=max_colors=256:stats_mode=full[p];[b][p]paletteuse=dither=none:diff_mode=rectangle" \
      "$assets/$name.gif"
    echo "   → $assets/$name.gif ($(du -h "$assets/$name.gif" | cut -f1))"
  fi
  rm -rf "$frames"
done
