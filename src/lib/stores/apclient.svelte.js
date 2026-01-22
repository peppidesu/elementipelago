import { derived, get, readable, writable } from "svelte/store";
import { Client } from "archipelago.js";
import { createSubscriber, SvelteMap, SvelteSet } from "svelte/reactivity";
import { element_to_name, parse_element } from "../../utils";
import { iconForItem, iconForLocation } from "../machine-learning/iconml";
import { draw } from "svelte/transition";
import { INTERMEDIATE_AMOUNT, LOCATION_AMOUNT } from "../../consts";
import { get_name, init_naming } from "./names.js";
import { ElementKind } from "../graph.js";

/**
 * @import { Graph, ElementID } from "../graph.js"
 * @import { Writable, Readable } from "svelte/store";
 * @import { Item } from "archipelago.js";

 * @typedef {{
    elem_id: ElementID
    name: string,
    icon: string,
    alt: string,
    location: string,
    player: string,
    game: string,
  }} ElementData
*/

export const apclient = writable(new Client());
export const slotdata = writable(null);
/**
 * @type {Writable<Graph>}
 */
export const graph = writable(null);

/**
 * @type {SvelteMap<string, ElementData>}
 */
const elementData = new SvelteMap();

/**
 * @returns {SvelteMap<string, ElementData>}
 */
export function getElementData() {
    return elementData;
}

/**
 * @type {SvelteSet<string>}
 */
const receivedElements = new SvelteSet();
/**
 * @type {SvelteSet<string>}
 */
const sentElements = new SvelteSet();

let drawerElements = $state(new Set());
/**
 * @returns {Set<string>}
 */
export function getDrawerElements() {
    return drawerElements;
}

/**
 * @type {SvelteSet<string>}
 */
const explorableElements = new SvelteSet();

/**
 * @param {string} el
 * @returns {boolean}
 */
export function isExplorable(el) {
    return explorableElements.has(el);
}

/**
 * @type {Set<string>}
 */
const exhaustedElements = new SvelteSet();

/**
 * @param {string} el
 * @returns {boolean}
 */
export function isExhausted(el) {
    return exhaustedElements.has(el);
}

/**
 * @type {Set<string>}
 */
const neededToGoal = new SvelteSet();

/**
 * @param {number[]} locations
 */
function checkForGoal(locations) {
    let client = get(apclient);
    for (const location of locations) {
        neededToGoal.delete(client.package.lookupLocationName("Elementipelago", location));
    }
    if (neededToGoal.size == 0) {
        client.goal();
    }
}

export function updateSets() {
    let gr = get(graph);
    if (gr == null) return;

    if (get(slotdata).compounds_are_ingredients) {
        drawerElements = receivedElements.union(sentElements);
    } else {
        drawerElements = receivedElements;
    }

    explorableElements.clear();
    exhaustedElements.clear();
    drawerElements.forEach((e) => exhaustedElements.add(e));

    for (const [[i1, i2], ps] of gr.recipes.entries()) {
        const i1_name = element_to_name(i1);
        const i2_name = element_to_name(i2);
        const has_both = drawerElements.has(i1_name) && drawerElements.has(i2_name);

        for (const p of ps) {
            if (!sentElements.has(element_to_name(p))) {
                if (has_both) {
                    explorableElements.add(i1_name);
                    explorableElements.add(i2_name);
                }
                exhaustedElements.delete(i1_name);
                exhaustedElements.delete(i2_name);
            }
        }
    }
}

export async function initElementStores() {
    const client = get(apclient);
    const scoutedLocations = client.scout(
        client.room.allLocations.filter((loc) => loc <= LOCATION_AMOUNT),
        0,
    );

    // This might fit better in a different place, but should happen between
    // login and the stores being filled
    const sd = get(slotdata);
    init_naming(sd.graph_seed);

    extendReceivedElements(client.items.received);
    extendSentElements(client.room.checkedLocations);

    // Add the missing intermediates to `neededToGoal` for the goal condition
    for (const location of client.room.missingLocations) {
        if (location <= LOCATION_AMOUNT || location > LOCATION_AMOUNT + INTERMEDIATE_AMOUNT) {
            // the location is not an "intermediate" so we skip adding it
            continue;
        }
        neededToGoal.add(client.package.lookupLocationName("Elementipelago", location));
    }

    for (const item of await scoutedLocations) {
        if (!elementData.has(item.locationName)) {
            const icon_name = iconForLocation(item);
            const elem_id = parse_element(item.locationName);
            elementData.set(item.locationName, {
                elem_id,
                name: item.locationName,
                icon: "/sprites/elements/" + icon_name + ".png",
                alt: icon_name,
                location: elem_id.kind === ElementKind.INTERMEDIATE ? get_name() : item.name,
                player: item.receiver.alias,
                game: item.receiver.game,
            });
        }
    }
    updateSets();
    checkForGoal([]);

    client.items.on("itemsReceived", extendReceivedElements);
    client.items.on("itemsReceived", updateSets);
    client.room.on("locationsChecked", extendSentElements);
    client.room.on("locationsChecked", updateSets);
    client.room.on("locationsChecked", checkForGoal);
}

/**
 * @param {Item[]} items
 */
function extendReceivedElements(items) {
    for (const item of items) {
        let icon_name = iconForItem(item);
        if (item.id < 100) {
            continue;
        }
        let elem_id = parse_element(item.name);
        receivedElements.add(item.name);
        if (elementData.has(item.name)) {
            continue;
        }

        elementData.set(item.name, {
            elem_id,
            name: item.name,
            icon: "/sprites/elements/" + icon_name + ".png",
            alt: icon_name,
            location:
                elem_id.kind === ElementKind.INTERMEDIATE || item.locationGame === "Archipelago"
                    ? get_name()
                    : item.locationName,
            player: item.sender.alias,
            game: item.sender.game,
        });
    }
}

/**
 * @param {number[]} locations
 */
function extendSentElements(locations) {
    let client = get(apclient);
    for (const location of locations) {
        sentElements.add(client.package.lookupLocationName("Elementipelago", location));
    }
}
