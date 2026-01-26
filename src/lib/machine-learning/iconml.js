import { get } from "svelte/store";
import { model } from "../stores/model.js";
import { md5 } from "js-md5";
import { getElementData } from "../stores/apclient.svelte.js";
// Todo: actual machine learning

/**
 * @param {string} game
 * @param {string} name
 */
export function iconForLocation(game, name) {
    const kind = "item";
    return iconForText(`[game=${game}] ${name}`);
}

/**
 * @param {string} game
 * @param {string} name
 */
export function iconForItem(game, name) {
    const kind = "location";
    return iconForText(`[game=${game}] ${name}`);
}

const colors = [
    "lightblue",
    "turqoise",
    "green",
    "lime",
    "yellow",
    "orange",
    "warmred",
    "red",
    "magenta",
    "purple",
    "indigo",
    "blue",
    "white"
];

const substituteIcons = [
    "apple",
    "armor",
    "ball",
    "berry",
    "boat",
    "book",
    "element",
    "emerald",
    "car",
];

const icons = [
    "boots",
    "bow",
    "cave",
    "chest",
    "coin",
    "desert",
    "egg",
    "fire",
    "gun",
    "hammer",
    "hat",
    "heart",
    "hills",
    "hourglass",
    "house",
    "ice",
    "island",
    "key",
    "leaf",
    "magic",
    "map",
    "marker",
    "metal",
    "money",
    "mountains",
    "music",
    "piece",
    "planet",
    "potion",
    "quest",
    "ring",
    "rock",
    "sand",
    "shop",
    "sign",
    "skull",
    "spear",
    "sword",
    "tree",
    "upgrade",
    "void",
    "wand",
    "water"
];

const combinedIcons = [...icons, ...substituteIcons.flatMap((value) => colors.map((color) => `${value}-${color}`))]

/**
 * @param {String} text
 * @returns {String}
 */
function iconForText(text) {
    //const res = predictIcon(get(model), text, { returnTopK: 1 });
    const buffer = new Uint32Array(md5.arrayBuffer(text));
    const res = combinedIcons[buffer[buffer.length - 1] % combinedIcons.length];

    const iconKey = res;
    return iconKey;
}
