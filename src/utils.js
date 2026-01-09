import { LOCATION_AMOUNT } from "./consts";

/**
 * @param {string} name
 */
export function name_to_kind(name) {
    const m = name.match(/^(Element|Intermediate|Compound)\s+(\d+)$/);

    if (m == null) {
        return null;
    }

    const [, type, numStr] = m;
    switch (type) {
        case "Element":
            return { id: Number(numStr), kind: 0 };
        case "Intermediate":
            return { id: Number(numStr), kind: 1 };
        case "Compound":
            return { id: Number(numStr), kind: 2 };
    }
    return null;
}

/**
 * @param {{ kind: any; id: string; }} elem
 */
export function elem_to_name(elem) {
    switch (elem.kind) {
        case 0:
            return "Element " + elem.id;
        case 1:
            return "Intermediate " + elem.id;
        case 2:
            return "Compound " + elem.id;
    }
}

/**
 * @param {{ kind: any; id: number; }} elem
 */
export function elem_to_location_id(elem) {
    switch (elem.kind) {
        case 0:
            throw "Not a valid location";
        case 1:
            return LOCATION_AMOUNT + elem.id;
        case 2:
            return elem.id;
    }
}
