import { get } from "svelte/store";
import { model } from "../stores/model.js";
// Todo: actual machine learning

/**
 * @param {import("archipelago.js").Item} location
 */
export async function iconForLocation(location) {
  const game = location.game;
  const kind = "item";
  const name = location.name;

  return iconForText(`[game=${game}][kind=${kind}] ${name}`);
}

/**
 * @param {import("archipelago.js").Item} item
 */
export async function iconForItem(item) {
  const game = item.locationGame;
  const kind = "location";
  const name = item.locationName;

  return iconForText(`[game=${game}][kind=${kind}] ${name}`);
}

async function iconForText(text) {
  //const res = predictIcon(get(model), text, { returnTopK: 1 });

  const icons = [
    "apple",
    "armor",
    "ball",
    "berry",
    "boat",
    "book",
    "boots",
    "car",
    "chest",
    "coin",
    "egg",
    "element",
    "emerald",
    "fire",
    "hat",
    "heart",
    "hourglass",
    "house",
    "ice",
    "key",
    "leaf",
    "magic",
    "map",
    "marker",
    "metal",
    "piece",
    "potion",
    "ring",
    "rock",
    "sand",
    "sign",
    "skull",
    "sword",
    "tree",
    "water",
  ];
  const enc = new TextEncoder();
  const txtbuf = enc.encode(text);
  const buffer = new Uint32Array(await crypto.subtle.digest("SHA-1", txtbuf));
  const res = icons[buffer[buffer.length - 1] % icons.length];

  const iconKey = res;
  return iconKey;
}
