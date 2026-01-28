import { derived, get, readable, writable } from "svelte/store";
import { Client } from "archipelago.js";
import { createSubscriber, SvelteMap, SvelteSet } from "svelte/reactivity";
import { element_to_name, parse_element } from "../../utils";
import { iconForItem, iconForLocation } from "../machine-learning/iconml";
import { draw } from "svelte/transition";
import { INTERMEDIATE_AMOUNT, LOCATION_AMOUNT, NON_ELEMENT_ITEMS } from "../../consts";
import { get_name, init_naming } from "./names.js";
import { ElementKind } from "../graph.js";
import { sendElementToasts, sendUpgradeToasts } from "./toast";

/**
 * @import { Graph, ElementID } from "../graph.js"
 * @import { Writable, Readable } from "svelte/store";
 * @import { Hint, Item } from "archipelago.js";

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
 *  @type {SvelteMap<string, {
 *      ingredient_1: string,
 *      ingredient_2: string,
 *      result: string,
 *      found: boolean
 *      }
 *  >}
 */
export const hintedElements = new SvelteMap();

/**
 *  @type {SvelteMap<string, {
 *      ingredient_1: ElementData,
 *      ingredient_2: ElementData,
 *      product: ElementData
 *      found: boolean
 *      }
 *  >}
 */
export const hints = new SvelteMap();

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

export const upgrades = $state({
    progressive_filter: 0,
    field_size: 10,
});

/**
 * @param {string} name
 */
function default_data(name) {
    let elem_id = parse_element(name);
    return {
        icon: "/sprites/elements/void.png",
        alt: "void",
        name: name,
        elem_id: elem_id,
        location: name,
        player:
            elem_id.kind === ElementKind.INTERMEDIATE
                ? get(apclient).players.self.alias
                : "Unknown",
        game:
            elem_id.kind === ElementKind.INTERMEDIATE ? get(apclient).players.self.game : "Unknown",
    };
}

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
    hints.clear();
    hintedElements
        .entries()
        .map(([name, hint]) => {
            return [
                name,
                {
                    ingredient_1:
                        elementData.get(hint.ingredient_1) ?? default_data(hint.ingredient_1),
                    ingredient_2:
                        elementData.get(hint.ingredient_2) ?? default_data(hint.ingredient_2),
                    product: elementData.get(hint.result) ?? default_data(hint.result),
                    found: hint.found,
                },
            ];
        })
        .forEach(([name, val]) => hints.set(name, val));
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
            const elem_id = parse_element(item.locationName);
            const loc = elem_id.kind === ElementKind.INTERMEDIATE ? get_name() : item.name;
            const icon_name = iconForLocation(item.game, loc);
            elementData.set(item.locationName, {
                elem_id,
                name: item.locationName,
                icon: "/sprites/elements/" + icon_name + ".png",
                alt: icon_name,
                location: loc,
                player: item.receiver.alias,
                game: item.receiver.game,
            });
        }
    }
    client.items.hints.forEach((hint) => extendReceivedHints(hint));
    updateSets();
    checkForGoal([]);

    client.items.on("hintsInitialized", (hints) => {
        hints.forEach((hint) => extendReceivedHints(hint));
        updateSets();
    });
    client.items.on("hintReceived", (hint) => {
        extendReceivedHints(hint);
        updateSets();
    });
    client.items.on("hintFound", (hint) => {
        if (hintedElements.has(hint.item.locationName)) {
            hintedElements.get(hint.item.locationName).found = true;
        }
        updateSets();
    });

    client.items.on("itemsReceived", extendReceivedElements);
    client.items.on("itemsReceived", updateSets);

    client.room.on("locationsChecked", extendSentElements);
    client.room.on("locationsChecked", updateSets);
    client.room.on("locationsChecked", checkForGoal);
}

/**
 * @param {Hint} hint
 */
function extendReceivedHints(hint) {
    const item = hint.item;
    const elem_name = item.locationName;
    if (hintedElements.has(elem_name)) {
        // we already have this compound in the recipe tree, no need to add it again
        return;
    }

    let gr = get(graph);
    if (gr == null) return;

    let ways_its_been_made = 0;

    for (const [[i1, i2], ps] of gr.recipes.entries()) {
        for (const p of ps) {
            const prod_name = element_to_name(p);
            if (prod_name == elem_name) {
                const i1_name = element_to_name(i1);
                const i2_name = element_to_name(i2);
                hintedElements.set(elem_name + " " + ways_its_been_made, {
                    found: hint.found,
                    ingredient_1: i1_name,
                    ingredient_2: i2_name,
                    result: elem_name,
                });
                ways_its_been_made += 1;
            }
        }
    }
}

/**
 * @param {Item[]} items
 */
function extendReceivedElements(items) {
    const game_id = get(apclient).room.seedName + "_" + get(apclient).name;

    const slotDataMap = JSON.parse(localStorage.getItem("ap.slotData")) ?? {};
    if (!slotDataMap.hasOwnProperty(game_id)) {
        slotDataMap[game_id] = { receivedItems: [], upgrades, date: new Date() };

        let entries = Object.entries(slotDataMap);
        if (entries.length > 100) {
            let [oldest, _] = entries
                .map(([k, v]) => [k, v.date])
                .reduce(([ka, da], [kc, dc]) => (da > dc ? [kc, dc] : [ka, da]));

            delete slotDataMap[oldest];
        }
    }
    /** @type string[] */
    const localReceived = slotDataMap[game_id].receivedItems;

    const newLocalReceived = [...localReceived];
    for (const item of items) {
        // it isn't an element, but an upgrade or todo instead
        if (item.id < NON_ELEMENT_ITEMS) {
            if (item.name == "Progressive Filter") {
                upgrades.progressive_filter += 1;
            }
            if (item.name == "Progressive Item Limit") {
                upgrades.field_size += 1;
            }
            continue;
        }
        if (!newLocalReceived.includes(item.name)) {
            newLocalReceived.push(item.name);
        }

        let elem_id = parse_element(item.name);
        const loc =
            elem_id.kind === ElementKind.INTERMEDIATE || item.locationGame === "Archipelago"
                ? get_name()
                : item.locationName;
        let icon_name = iconForItem(item.game, loc);
        receivedElements.add(item.name);

        if (elementData.has(item.name)) {
            continue;
        }

        elementData.set(item.name, {
            elem_id,
            name: item.name,
            icon: "/sprites/elements/" + icon_name + ".png",
            alt: icon_name,
            location: loc,
            player: item.sender.alias,
            game: item.sender.game,
        });
    }
    const oldUpgrades = slotDataMap[game_id].upgrades;
    slotDataMap[game_id].upgrades = upgrades;
    slotDataMap[game_id].receivedItems = newLocalReceived;
    localStorage.setItem("ap.slotData", JSON.stringify(slotDataMap));

    const newItems = items.filter(
        (item) => item.id >= NON_ELEMENT_ITEMS && !localReceived.includes(item.name),
    );
    sendElementToasts(newItems);
    sendUpgradeToasts(oldUpgrades, upgrades);
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
