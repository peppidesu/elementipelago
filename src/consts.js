export const LOCATION_AMOUNT = 2000;
export const INTERMEDIATE_AMOUNT = 1000;
export const NON_ELEMENT_ITEMS = 100;

export const APWORLD_VERSIONS = ["0.3.x", "1.x.x"];

export const APWORLD_VERSION_REGEX = new RegExp(
    APWORLD_VERSIONS.map((s) => `^(${s.replace("x", "\\d+")}-?.*)`).join("|"),
);
