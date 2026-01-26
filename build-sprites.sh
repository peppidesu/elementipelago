#!/usr/bin/env bash

ROOT="$(git rev-parse --show-toplevel)"
SRC="$ROOT/sprites"
TMP="$(mktemp -d)/sprites"
DEST="$ROOT/public/sprites"

sub_inputs=(ff00ff aa00aa 440044 110011)

declare -A palettes=(
    [lightblue]="305182 "
    [turqoise]=""
    [green]=""
    [lime]=""
    [yellow]=""
    [orange]=""
    [warmred]=""
    [red]=""
    [magenta]=""
    [purple]=""
    [indigo]=""
    [blue]=""
    [white]=""
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
  dest_base="$DEST/${rel%.png}"
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
