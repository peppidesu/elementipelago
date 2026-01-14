import { get } from "svelte/store";
import { model } from "../stores/model.js";
import { Rng } from "../../graph.js";

function normalize(s) {
    s = s.trim().toLowerCase();
    s = s.split(/\s+/).join(" ");
    return s;
}

function utf8Bytes(str) {
    return new TextEncoder().encode(str);
}

function fnv1a32(bytes) {
    let h = 0x811c9dc5; // 2166136261
    for (let i = 0; i < bytes.length; i++) {
        h ^= bytes[i];
        // h *= 16777619 (mod 2^32)
        h = Math.imul(h, 0x01000193) >>> 0;
    }
    return h >>> 0;
}

function* charNgrams(text, nmin, nmax) {
    const s = "^" + text + "$";
    for (let n = nmin; n <= nmax; n++) {
        for (let i = 0; i + n <= s.length; i++) {
            yield s.slice(i, i + n);
        }
    }
}

function featurize(text, B, nmin, nmax) {
    const t = normalize(text);
    const counts = new Map(); // bucket -> count
    for (const ng of charNgrams(t, nmin, nmax)) {
        const idx = fnv1a32(utf8Bytes(ng)) % B;
        counts.set(idx, (counts.get(idx) || 0) + 1);
    }
    return counts;
}

export async function loadIconModel(metaUrl, binUrl) {
    const meta = await fetch(metaUrl).then((r) => r.json());
    const buf = await fetch(binUrl).then((r) => r.arrayBuffer());
    const dv = new DataView(buf);

    let off = 0;
    const magic = dv.getUint32(off, true);
    off += 4;
    const version = dv.getUint32(off, true);
    off += 4;
    if (magic !== 0x424E4349 || version !== 1) {
        throw new Error("Bad model file");
    }

    const C = dv.getUint32(off, true);
    off += 4;
    const B = dv.getUint32(off, true);
    off += 4;
    const scale = dv.getFloat32(off, true);
    off += 4;

    // priors
    const priors = new Float32Array(buf, off, C);
    off += C * 4;

    // qweights int8, length C*B
    const qweights = new Int8Array(buf, off, C * B);

    return { meta, C, B, scale, priors, qweights };
}

export function predictIcon(model, text, { returnTopK = 1 } = {}) {
    const { meta, C, B, scale, priors, qweights } = model;
    const counts = featurize(text, B, meta.nmin, meta.nmax);

    // Score each class:
    // score = prior + sum_{bucket} count(bucket) * (qweight * scale)
    const scores = new Float32Array(C);
    for (let c = 0; c < C; c++) scores[c] = priors[c];

    for (const [b, cnt] of counts.entries()) {
        const base = b; // offset within each class row
        for (let c = 0; c < C; c++) {
            const w = qweights[c * B + base]; // int8
            scores[c] += cnt * (w * scale);
        }
    }

    // topK
    const idx = Array.from(scores, (_, i) => i);
    idx.sort((a, b) => scores[b] - scores[a]);
    const top = idx.slice(0, returnTopK).map((i) => ({
        iconKey: meta.classes[i],
        score: scores[i],
    }));

    const margin = top.length >= 2 ? (top[0].score - top[1].score) : Infinity;

    return {
        best: top[0],
        top,
        margin,
    };
}

/**
 * @param {import("archipelago.js").Item} location
 */
export function iconForLocation(location) {
    const game = location.game;
    const kind = "item";
    const name = location.name;

    return iconForText(`[game=${game}][kind=${kind}] ${name}`);
}

/**
 * @param {import("archipelago.js").Item} item
 */
export function iconForItem(item) {
    const game = item.locationGame;
    const kind = "location";
    const name = item.locationName;

    return iconForText(`[game=${game}][kind=${kind}] ${name}`);
}

let rng = new Rng(2718281828n);
function iconForText(text) {
    //const res = predictIcon(get(model), text, { returnTopK: 1 });

    const icons = [
        "apple",
        "armmor",
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
        "void",
        "water",
    ];

    const res = icons[Number(rng.get_random()) % icons.length];

    const iconKey = res;
    return iconKey;
}
