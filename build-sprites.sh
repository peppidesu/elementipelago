#!/bin/sh

ROOT="$(git rev-parse --show-toplevel)"
SRC="$ROOT/sprites"
TMP="$(mktemp -d)/sprites"
DEST="$ROOT/public/sprites"


find "$SRC" -type f -name '*.aseprite' | while read -r file; do
  rel="${file#"$SRC"/}"
  out="$TMP/${rel%.aseprite}.png"

  mkdir -p "$(dirname "$out")"

  aseprite \
    --batch "$file" \
    --save-as "$out"
done

find "$TMP" -type f -name '*.png' | while read -r file; do
  rel="${file#"$TMP"/}"
  out="$DEST/${rel%.png}.png"

  mkdir -p "$(dirname "$out")"

  png-subs "$file" "$out"
done
