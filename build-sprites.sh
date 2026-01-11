#!/bin/sh

ROOT="$(git rev-parse --show-toplevel)"
SRC="$ROOT/assets"
DEST="$ROOT/src/assets"

find "$SRC" -type f -name '*.aseprite' | while read -r file; do
  rel="${file#"$SRC"/}"
  out="$DEST/${rel%.aseprite}.png"

  mkdir -p "$(dirname "$out")"

  aseprite \
    --batch "$file" \
    --save-as "$out"
done
