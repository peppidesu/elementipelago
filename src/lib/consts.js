export const LOCATION_AMOUNT = 2000;
export const INTERMEDIATE_AMOUNT = 1000;
export const NON_ELEMENT_ITEMS = 100;

export const APWORLD_VERSIONS = ["0.3.x", "1.x.x"];

export const APWORLD_VERSION_REGEX = new RegExp(
    "^(" +
        APWORLD_VERSIONS.map((s) => `(${s.replaceAll("x", "\\d+").replaceAll(".", "\\.")})`).join(
            "|",
        ) +
        ")(-.*)?$",
);

export const SUBSTITUTE_ICONS = [
    "apple",
    "armor",
    "ball",
    "berry",
    "boat",
    "book",
    "boots",
    "car",
    "chest",
    "element",
    "emerald",
    "hat",
    "heart",
    "leaf",
    "magic",
    "marker",
    "metal",
    "music",
    "potion",
    "sand",
    "wand",
];
export const ELEMENT_ICONS = [
    "bow",
    "cave",
    "coin",
    "desert",
    "egg",
    "fire",
    "gun",
    "hammer",
    "hills",
    "hourglass",
    "house",
    "ice",
    "island",
    "key",
    "map",
    "money",
    "mountains",
    "piece",
    "planet",
    "quest",
    "ring",
    "rock",
    "shop",
    "sign",
    "skull",
    "spear",
    "sword",
    "tree",
    "upgrade",
    "void",
    "water",
];
export const ALL_ICONS = [...SUBSTITUTE_ICONS, ...ELEMENT_ICONS];
export const ICON_PALETTES = [
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
    "white",
];

export const ELEMENT_SYNONYMS = ["chemical", "element", "ingredient", "material", "substance"];
