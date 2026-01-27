import { Client } from "archipelago.js";
import { createSubscriber } from "svelte/reactivity";
import { elementIdToName, elementNameToId } from "../utils";
import { iconForIntermediate, iconForItem, iconForLocation } from "../icons";
import { INTERMEDIATE_AMOUNT, LOCATION_AMOUNT, NON_ELEMENT_ITEMS } from "../consts";
import { get_name, init_naming } from "../names.js";
import { createGraph, ElementKind } from "../graph.js";
import { get, readable } from "svelte/store";

/**
@import { Graph, ElementID } from "../graph.js"
@import { Writable, Readable } from "svelte/store";
@import { Hint, Item } from "archipelago.js";

@typedef {{
    elem_id: ElementID
    name: string,
    icon: string,
    alt: string,
    location: string,
    player: string,
    game: string,
}} ElementData

@typedef {{
    element_amount: number,
    intermediate_amount: number,
    filler_amount: number,
    compound_amount: number,
    compounds_are_ingredients: number,
    graph_seed: number,
    version: string,
}} SlotData

@typedef {{
    ingredient_1: string,
    ingredient_2: string,
    result: string,
    found: boolean
}} ElementHint

@typedef {
    "elementsReceived" |
    "upgradesReceived"
} APStoreEvent

@typedef { (...args: any[]) => void } EventCallback
*/

class LocalGameData {
    #localGameDataMap;
    #gameId;
    constructor(client, upgrades) {
        this.#gameId = client.room.seedName + "_" + client.name;
        this.#localGameDataMap = JSON.parse(localStorage.getItem("ap.slotData")) ?? {};
        if (!this.#localGameDataMap.hasOwnProperty(this.#gameId)) {
            this.#localGameDataMap[this.#gameId] = {
                receivedItems: [],
                upgrades,
                date: new Date(),
            };

            let entries = Object.entries(this.#localGameDataMap);
            if (entries.length > 100) {
                let [oldest, _] = entries
                    .map(([k, v]) => [k, v.date])
                    .reduce(([ka, da], [kc, dc]) => (da > dc ? [kc, dc] : [ka, da]));

                delete this.#localGameDataMap[oldest];
            }
        }
    }

    /** @returns {{
        receivedItems: string[],
        upgrades: {},
        date: Date,
    }}*/
    get current() {
        return this.#localGameDataMap[this.#gameId];
    }
    /** @param {{
        receivedItems: string[],
        upgrades: {},
        date: Date,
    }} x */
    set current(x) {
        this.#localGameDataMap[this.#gameId] = x;
        localStorage.setItem("ap.slotData", JSON.stringify(this.#localGameDataMap));
    }
}

class EventEmitter {
    /** @type {Record<string, [callback: EventCallback, once: boolean][]>} */
    #events = {};

    /**
     * @param {string} event
     * @param {EventCallback} callback
     * @returns {void}
     */
    addEventListener(event, callback, once = false) {
        // Ensure list is not empty.
        this.#events[event] ??= [];
        this.#events[event].push([callback, once]);
    }

    /**
     * @param {string} event
     * @param {EventCallback} callback
     */
    removeEventListener(event, callback) {
        const callbacks = this.#events[event];
        if (callbacks && callbacks.length > 0) {
            this.#events[event] = callbacks.filter(([cb]) => cb !== callback);
        }
    }

    /**
     * @param {string} event
     * @param {any} data
     * @returns {void}
     */
    dispatchEvent(event, data) {
        const callbacks = this.#events[event] ?? [];
        for (const [callback, once] of callbacks) {
            callback(...data);
            if (once) {
                this.removeEventListener(event, callback);
            }
        }
    }
}

class APStore {
    /** @type {Client} */
    #client;
    /** @type {Graph} */
    #graph;
    /** @type {SlotData} */
    #slotData;

    #eventEmitter = new EventEmitter();

    /** @type {Record<string, ElementData>} */
    #elementData = {};

    /** @type {Set<string>} */
    #receivedElements = new Set();
    /** @type {Set<string>} */
    #sentElements = new Set();
    /** @type {Set<string>} */
    #drawerElements = new Set();
    /** @type {Set<string>} */
    #explorableElements = new Set();
    /** @type {Set<string>} */
    #exhaustedElements = new Set();
    /** @type {Set<string>} */
    #neededToGoal = new Set();

    /** @type {Map<string, ElementHint[]>} */
    #hintedElements = new Map();

    #upgrades = {
        progressive_filter: 0,
        field_size: 10,
    };

    /** @type {() => void} */
    #subItems;
    /** @type {() => void} */
    #subHints;

    #localGameData;

    constructor() {
        this.#client = new Client();
        this.#localGameData = new LocalGameData(this.#client, this.#upgrades);

        this.#subItems = createSubscriber((update) => {
            this.#client.items.on("itemsReceived", (items, _) => {
                this.#extendReceivedElements(items);
                this.#updateElementSets();
                update();
            });
            this.#client.room.on("locationsChecked", (locations) => {
                this.#extendSentElements(locations);
                this.#updateElementSets();
                this.#checkForGoal(locations);
                update();
            });
        });

        this.#subHints = createSubscriber((update) => {
            this.#client.items.on("hintsInitialized", (hints) => {
                hints.forEach((hint) => this.#extendReceivedHints(hint));
                update();
            });
            this.#client.items.on("hintReceived", (hint) => {
                this.#extendReceivedHints(hint);
                update();
            });
            this.#client.items.on("hintFound", (hint) => {
                if (this.#hintedElements.has(hint.item.locationName)) {
                    this.#hintedElements
                        .get(hint.item.locationName)
                        .forEach((h) => (h.found = true));
                    update();
                }
            });
        });
    }

    /**
     * @param {SlotData} slotData
     */
    async init(slotData) {
        this.#slotData = slotData;
        this.#graph = createGraph(
            BigInt(this.#slotData.graph_seed),
            this.#slotData.element_amount,
            this.#slotData.compound_amount,
            this.#slotData.intermediate_amount,
            4,
            this.#slotData.compounds_are_ingredients,
        );

        const scoutedLocations = this.#client.scout(
            this.#client.room.allLocations.filter((loc) => loc <= LOCATION_AMOUNT),
            0,
        );

        // This might fit better in a different place, but should happen between
        // login and the stores being filled
        const sd = this.#slotData;
        init_naming(sd.graph_seed);

        this.#extendReceivedElements(this.#client.items.received);
        this.#extendSentElements(this.#client.room.checkedLocations);

        // Add the missing intermediates to `neededToGoal` for the goal condition
        for (const location of this.#client.room.missingLocations) {
            if (location <= LOCATION_AMOUNT || location > LOCATION_AMOUNT + INTERMEDIATE_AMOUNT) {
                // the location is not an "intermediate" so we skip adding it
                continue;
            }
            this.#neededToGoal.add(
                this.#client.package.lookupLocationName("Elementipelago", location),
            );
        }

        for (const item of await scoutedLocations) {
            this.#addElementData(item, true);
        }

        this.#client.items.hints.forEach((hint) => this.#extendReceivedHints(hint));
        this.#updateElementSets();
        this.#checkForGoal([]);
    }

    /**
     * @param {APStoreEvent} event
     * @param {EventCallback} callback
     */
    on(event, callback) {
        this.#eventEmitter.addEventListener(event, callback);
        return this;
    }
    /**
     * @param {APStoreEvent} event
     * @param {EventCallback} callback
     */
    off(event, callback) {
        this.#eventEmitter.removeEventListener(event, callback);
        return this;
    }

    get client() {
        return this.#client;
    }

    get graph() {
        return this.#graph;
    }

    get slotData() {
        return this.#slotData;
    }

    get elementData() {
        this.#subItems();
        return this.#elementData;
    }

    get receivedElements() {
        this.#subItems();
        return this.#receivedElements;
    }

    get sentElements() {
        this.#subHints();
        return this.#sentElements;
    }

    get drawerElements() {
        this.#subItems();
        return this.#drawerElements;
    }

    get upgrades() {
        this.#subItems();
        return this.#upgrades;
    }

    get hintedElements() {
        this.#subHints();
        return this.#hintedElements;
    }

    /**
     * @param {string} name
     */
    isExplorable(name) {
        this.#subItems();
        return this.#explorableElements.has(name);
    }
    /**
     * @param {string} name
     */
    isExhausted(name) {
        this.#subItems();
        return this.#exhaustedElements.has(name);
    }

    /**
     * @param {Item} item
     */
    #addElementData(item, isLocation = false) {
        const name = isLocation ? item.locationName : item.name;
        if (this.#elementData[name] !== undefined) {
            return;
        }
        const displayName = isLocation ? item.name : item.locationName;

        const elem_id = elementNameToId(name);

        let loc, icon_name;
        if (elem_id.kind === ElementKind.INTERMEDIATE) {
            loc = get_name();
            icon_name = iconForIntermediate(loc);
        } else {
            loc =
                elem_id.kind === ElementKind.INTERMEDIATE || item.locationGame === "Archipelago"
                    ? get_name()
                    : displayName;

            icon_name = isLocation ? iconForLocation(item.game, loc) : iconForItem(item.game, loc);
        }

        this.#elementData[name] = {
            elem_id,
            name: name,
            icon: "/sprites/elements/" + icon_name + ".png",
            alt: icon_name,
            location: loc,
            player: isLocation ? item.receiver.alias : item.sender.alias,
            game: isLocation ? item.receiver.game : item.sender.game,
        };
    }

    #updateElementSets() {
        if (this.#slotData.compounds_are_ingredients) {
            this.#drawerElements = this.#receivedElements.union(this.#sentElements);
        } else {
            this.#drawerElements = this.#receivedElements;
        }

        this.#explorableElements.clear();
        this.#exhaustedElements.clear();
        this.#drawerElements.forEach((e) => this.#exhaustedElements.add(e));

        for (const [[i1, i2], ps] of this.#graph.recipes.entries()) {
            const i1_name = elementIdToName(i1);
            const i2_name = elementIdToName(i2);
            const has_both = this.#drawerElements.has(i1_name) && this.#drawerElements.has(i2_name);

            for (const p of ps) {
                if (!this.#sentElements.has(elementIdToName(p))) {
                    if (has_both) {
                        this.#explorableElements.add(i1_name);
                        this.#explorableElements.add(i2_name);
                    }
                    this.#exhaustedElements.delete(i1_name);
                    this.#exhaustedElements.delete(i2_name);
                }
            }
        }
    }

    #extendReceivedElements(items) {
        /** @type string[] */
        const localReceived = this.#localGameData.current.receivedItems;

        const newLocalReceived = [...localReceived];
        for (const item of items) {
            // it isn't an element, but an upgrade or todo instead
            if (item.id < NON_ELEMENT_ITEMS) {
                if (item.name == "Progressive Filter") {
                    this.#upgrades.progressive_filter += 1;
                }
                if (item.name == "Progressive Item Limit") {
                    this.#upgrades.field_size += 1;
                }
                continue;
            }
            if (!newLocalReceived.includes(item.name)) {
                newLocalReceived.push(item.name);
            }
            this.#receivedElements.add(item.name);
            this.#addElementData(item);
        }
        const oldUpgrades = { ...this.#localGameData.current.upgrades };

        this.#localGameData.current = {
            upgrades: { ...this.#upgrades },
            receivedItems: newLocalReceived,
            date: new Date(),
        };

        const newElements = items.filter(
            (item) => item.id >= NON_ELEMENT_ITEMS && !localReceived.includes(item.name),
        );

        this.#eventEmitter.dispatchEvent("elementsReceived", [newElements]);
        this.#eventEmitter.dispatchEvent("upgradesReceived", [oldUpgrades, this.#upgrades]);
    }

    /**
     * @param {number[]} locations
     */
    #extendSentElements(locations) {
        for (const location of locations) {
            this.#sentElements.add(
                this.#client.package.lookupLocationName("Elementipelago", location),
            );
        }
    }

    #extendReceivedHints(hint) {
        const name = hint.item.locationName;
        if (this.#hintedElements.has(name)) {
            // we already have this compound in the recipe tree, no need to add it again
            return;
        }
        let s = $state([]);
        this.#hintedElements.set(name, s);

        for (const [[i1, i2], ps] of this.#graph.recipes.entries()) {
            for (const p of ps) {
                const prod_name = elementIdToName(p);
                if (prod_name == name) {
                    const i1_name = elementIdToName(i1);
                    const i2_name = elementIdToName(i2);

                    this.#hintedElements.get(name).push({
                        found: hint.found,
                        ingredient_1: i1_name,
                        ingredient_2: i2_name,
                        result: name,
                    });
                }
            }
        }
    }

    /**
     * @param {number[]} locations
     */
    #checkForGoal(locations) {
        for (const location of locations) {
            this.#neededToGoal.delete(
                this.#client.package.lookupLocationName("Elementipelago", location),
            );
        }
        if (this.#neededToGoal.size == 0) {
            this.#client.goal();
        }
    }
}
export const apstore = $state(new APStore());
/**
 * @param {string} name
 */
export function defaultElementData(name) {
    let elem_id = elementNameToId(name);
    return {
        icon: "/sprites/elements/void.png",
        alt: "void",
        name: name,
        elem_id: elem_id,
        location: name,
        player:
            elem_id.kind === ElementKind.INTERMEDIATE
                ? apstore.client.players.self.alias
                : "Unknown",
        game:
            elem_id.kind === ElementKind.INTERMEDIATE
                ? apstore.client.players.self.game
                : "Unknown",
    };
}
