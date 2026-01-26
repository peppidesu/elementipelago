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

function iconForText(text) {
    //const res = predictIcon(get(model), text, { returnTopK: 1 });

    const icons = [
        "apple",
        "armor",
        "ball",
        "berry",
        "boat",
        "book",
        "boots",
        "bow",
        "car",
        "cave",
        "chest",
        "coin",
        "desert",
        "egg",
        "element",
        "emerald",
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
    const buffer = new Uint32Array(md5.arrayBuffer(text));
    const res = icons[buffer[buffer.length - 1] % icons.length];

    const iconKey = res;
    return iconKey;
}
