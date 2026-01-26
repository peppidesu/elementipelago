#!/usr/bin/env bash

ROOT="$(git rev-parse --show-toplevel)"
SRC="$ROOT/sprites"
TMP="$(mktemp -d)/sprites"
DEST="$ROOT/public/sprites"

sub_inputs=(220022 440044 aa00aa ff00ff)

declare -A palettes=(
    [lightblue]="305182 4192c3 61d3e3 a2fff3"
    [turqoise]="165950 189572 37da94 a0ffc2"
    [green]="006130 19a239 5ff040 ccffad"
    [lime]="263e0a 6e8d1b bee344 edff89"
    [yellow]="6d3c05 b87a0c ffd42d fff392"
    [orange]="732200 c64b01 ff8b17 ffdba2"
    [warmred]="681d27 ae2b28 ff5d32 ffae93"
    [red]="641b3b 9d193c ff4656 ff9ba1"
    [magenta]="612b79 9c37ba f75aff ffa9f0"
    [purple]="342561 6b42a7 b76cff e2abff"
    [indigo]="1f1d51 3e3c8b 6979e8 88baff"
    [blue]="19295b 305faa 3f9eff 8ad1ff"
    [white]="535353 8a8a8a c6c6c6 ffffff"
)

find "$SRC" -type f -name '*.aseprite' | while read -r file; do
  rel="${file#"$SRC"/}"
  out="$TMP/${rel%.aseprite}.png"

  mkdir -p "$(dirname "$out")"

  aseprite \
    --batch "$file" \
    --save-as "$out"
done

run_with_palette() {
    local in="$1" out="$2" palette_str="$3"
    local s1 s2 s3 s4
    read -r s1 s2 s3 s4 <<< "$palette_str"

    png-subs "$in" "$out" \
      --subs "${sub_inputs[0]}" "$s1" \
      --subs "${sub_inputs[1]}" "$s2" \
      --subs "${sub_inputs[2]}" "$s3" \
      --subs "${sub_inputs[3]}" "$s4"
}


find "$TMP" -type f -name '*.png' | while read -r file; do
  rel="${file#"$TMP"/}"
  out_rel="${rel//\/substitute\//\/}"
  dest_base="$DEST/${out_rel%.png}"

  mkdir -p "$(dirname "$dest_base")"

  if [[ "$rel" == *"/substitute/"* ]]; then
      for name in "${!palettes[@]}"; do
          out="${dest_base}-${name}.png"
          run_with_palette "$file" "$out" "${palettes[$name]}"
      done
  else
      out="${dest_base}.png"
      png-subs "$file" "$out"
  fi
done
