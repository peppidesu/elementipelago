import { LOCATION_AMOUNT } from "./consts";

/**
 * @enum {number}
 */
export const ElementKind = {
  INPUT: 1,
  INTERMEDIATE: 2,
  OUTPUT: 3,
};

/**
 * @typedef {{ id: number, kind: ElementKind } } ElementID
 */

/**
 * @param {string} name
 *
 * @returns {ElementID}
 */
export function parse_element(name) {
  const m = name.match(/^(Element|Intermediate|Compound)\s+(\d+)$/);

  if (m == null) {
    return null;
  }

  const [, type, numStr] = m;
  switch (type) {
    case "Element":
      return { id: Number(numStr), kind: ElementKind.INPUT };
    case "Intermediate":
      return { id: Number(numStr), kind: ElementKind.INTERMEDIATE };
    case "Compound":
      return { id: Number(numStr), kind: ElementKind.OUTPUT };
  }
  return null;
}

/**
 * @param {ElementID} elem
 */
export function element_to_name(elem) {
  switch (elem.kind) {
    case ElementKind.INPUT:
      return "Element " + elem.id;
    case ElementKind.INTERMEDIATE:
      return "Intermediate " + elem.id;
    case ElementKind.OUTPUT:
      return "Compound " + elem.id;
  }
}

/**
 * @param {ElementID} elem
 */
export function element_to_location_id(elem) {
  switch (elem.kind) {
    case ElementKind.INPUT:
      throw "Not a valid location";
    case ElementKind.INTERMEDIATE:
      return LOCATION_AMOUNT + elem.id;
    case ElementKind.OUTPUT:
      return elem.id;
  }
}
