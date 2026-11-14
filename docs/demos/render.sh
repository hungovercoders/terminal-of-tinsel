#!/usr/bin/env bash
# Renders the README's demo GIFs and screenshots from docs/demos/*.tape.
#
#   docs/demos/render.sh            # everything
#   docs/demos/render.sh hero quiz  # just these
#
# Needs: cargo, vhs (https://github.com/charmbracelet/vhs), ffmpeg, ttyd 1.7.7
# or newer, and Linux with Noto Color Emoji (fonts-noto-color-emoji) as
# fontconfig's emoji font.
# Set TTYD=/path/to/ttyd to pick a specific ttyd binary.
#
# Recordings use a fixed TERMINAL_OF_TINSEL_SEED (DEMO_SEED, default 25) and
# a fixed date (DEMO_DATE, default 2026-12-13) where a tape passes one, so
# re-recording gives the same questions, portraits and showdowns every time.
#
# Two workarounds live here rather than in the tapes:
#  - ttyd's browser terminal defaults to Unicode 6 widths, which draws emoji
#    one cell wide and knocks every box border out of line. A small ttyd
#    wrapper turns on Unicode 11.
#  - vhs writes raw frames and we encode them with ffmpeg ourselves, which
#    also keeps the GIFs small (15fps, diffed palette).
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"
demos=docs/demos
assets=docs/assets
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Stills keep only their final frame, as a PNG.
stills=" list random countdown tonight advent stocking "
bg="#0b1410"

for tool in cargo vhs ffmpeg fc-match; do
  command -v "$tool" > /dev/null || { echo "missing $tool" >&2; exit 1; }
done
real_ttyd=${TTYD:-$(command -v ttyd || true)}
[ -n "$real_ttyd" ] || { echo "missing ttyd (1.7.7+)" >&2; exit 1; }
# Older ttyd draws emoji one cell wide however it is started, and every
# border after one lands a column out. Refuse rather than record that.
ttyd_version=$("$real_ttyd" --version | awk '{ print $3 }' | cut -d- -f1)
if [ "$(printf '%s\n' 1.7.7 "$ttyd_version" | sort -V | head -1)" != 1.7.7 ]; then
  echo "ttyd $ttyd_version is too old: 1.7.7 or newer is needed for emoji to line up" >&2
  exit 1
fi
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

# The recorded shell gets a fake home, so saved paths read ~/.config/...
config="$work/home/.config/terminal-of-tinsel"
mkdir -p "$work/bin" "$config"
cat > "$work/bin/ttyd" <<WRAP
#!/bin/sh
exec "$real_ttyd" -t unicodeVersion=11 "\$@"
WRAP
chmod +x "$work/bin/ttyd"

# ttyd starts "bash" from PATH; this one swaps vhs's plain prompt for a tree.
real_bash=$(command -v bash)
printf '%s\n' "PS1='\\[\\e[38;2;82;183;136m\\]🎄 \\[\\e[0m\\]'" \
  "export HOME='$work/home' TERMINAL_OF_TINSEL_HOME='$config'" \
  "export TERMINAL_OF_TINSEL_SEED='${DEMO_SEED:-25}'" > "$work/rc"
printf '#!%s\nexec "%s" --noprofile --rcfile "%s" -i +o history\n' \
  "$real_bash" "$real_bash" "$work/rc" > "$work/bin/bash"
chmod +x "$work/bin/bash"

cargo build --release --quiet
cp target/release/terminal-of-tinsel "$work/bin/"

# A stocking with some history, so the screenshot has something to show.
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

# vhs moves its frames into place with a plain rename, which fails silently
# across filesystems or into a missing directory, so keep its temp files
# next to the output.
mkdir -p "$demos/.frames/tmp" "$assets"
export TMPDIR="$root/$demos/.frames/tmp"

export PATH="$work/bin:$PATH"
export VHS_NO_SANDBOX=${VHS_NO_SANDBOX:-true}

# One line of JSON, ready to drop into each tape.
theme=$(tr -d '\n' < "$demos/theme.json" | tr -s ' ')

names=("$@")
if [ ${#names[@]} -eq 0 ]; then
  for tape in "$demos"/*.tape; do names+=("$(basename "$tape" .tape)"); done
fi

for name in "${names[@]}"; do
  echo "🎬 $name"
  frames="$demos/.frames/$name"
  rm -rf "$frames"
  # Fill in the shared theme, so theme.json is the only place it lives.
  awk -v theme="$theme" '$0 == "Set Theme @theme.json" { print "Set Theme " theme; next } { print }' \
    "$demos/$name.tape" > "$work/$name.tape"
  vhs "$work/$name.tape" > "$work/$name.log" 2>&1 || { cat "$work/$name.log"; exit 1; }
  [ -e "$frames/frame-text-00001.png" ] || { cat "$work/$name.log"; echo "no frames for $name" >&2; exit 1; }

  # Read the frames back at the rate they were captured.
  fps=$(awk '$1 == "Set" && $2 == "Framerate" { print $3 }' "$demos/$name.tape")
  fps=${fps:-50}
  inputs=(-framerate "$fps" -start_number 1 -i "$frames/frame-text-%05d.png"
          -framerate "$fps" -start_number 1 -i "$frames/frame-cursor-%05d.png")
  pad="pad=iw+48:ih+48:24:24:color=$bg"
  if [[ "$stills" == *" $name "* ]]; then
    last=$(find "$frames" -name 'frame-text-*.png' | sort | tail -1)
    ffmpeg -loglevel error -y -i "$last" -vf "$pad" "$assets/$name.png"
    echo "   → $assets/$name.png"
  else
    ffmpeg -loglevel error -y "${inputs[@]}" -filter_complex \
      "[0][1]overlay,$pad,fps=15,split[a][b];[a]palettegen=max_colors=192:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle" \
      "$assets/$name.gif"
    echo "   → $assets/$name.gif ($(du -h "$assets/$name.gif" | cut -f1))"
  fi
  rm -rf "$frames"
done
rm -rf "$demos/.frames"
