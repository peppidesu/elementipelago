import { md5 } from "js-md5";

/**
 * @type {Set<string>}
 */
const used_names = new Set();

let seed;

const adjectives = [
    "Fool's ?",
    "Powdered ?",
    "King's ?",
    "Pure ?",
    "Impure ?",
    "Glass of ?",
    "Regulus of ?",
    "Resin of ?",
    "Spirit of ?",
    "Butter of ?",
    "Oil of ?",
    "Salt of ?",
    "? salt",
    "? cornu",
    "? vitriol",
    "Lunar ?",
    "Sugar of ?",
    "Flowers of ?",
    "? caustic",
    "Caustic ?",
    "? gum",
    "? spirit",
    "? of sulfur",
    "? alkali",
    "Oriental ?",
    "? essence",
    "Dried ?",
    "Alchemical ?",
    "Crystalline ?",
];

const nouns = [
    "Nigredo",
    "Albedo",
    "Citrinitas",
    "Rubedo",
    "Stibnium",
    "Cuprum",
    "Aurum",
    "Ferrum",
    "Plumbum",
    "Hydragyrum",
    "Argentum",
    "Stannum",
    "Bluestone",
    "Cadmia",
    "Calamine",
    "Calomel",
    "Calx",
    "Calcanthum",
    "Chalk",
    "Chrome green",
    "Chrome orange",
    "Chrome red",
    "Chrome yellow",
    "Cinnabar",
    "Cuprite",
    "Galena",
    "Antimony",
    "Gypsum",
    "Argentum",
    "Lapis Solaris",
    "Quicklime",
    "Marcasite",
    "Massicot",
    "Litharge",
    "Minium",
    "Mercurius Praecipitatus",
    "Orpiment",
    "Nix Alba",
    "Plumbago",
    "Algaroth",
    "Realgar",
    "Potash",
    "Vitriol",
    "Alkahest",
    "Azoth",
    "Aqua",
];

export function init_naming(init_seed) {
    seed = new Uint32Array(md5.arrayBuffer("" + init_seed));
}

export function get_name() {
    let combined;
    do {
        seed = new Uint32Array(md5.arrayBuffer(seed));

        const noun = nouns[seed[seed.length - 1] % nouns.length];
        const adj = adjectives[seed[0] % adjectives.length];

        combined = adj.replace("?", noun);
    } while (used_names.has(combined));
    used_names.add(combined);
    return combined;
}
