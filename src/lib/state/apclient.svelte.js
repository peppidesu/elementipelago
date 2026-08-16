import { Client } from "archipelago.js";
import { createSubscriber } from "svelte/reactivity";
import { elementIdToName, elementNameToId } from "../utils";
import { iconForIntermediate, iconForItem, iconForLocation } from "../icons";
import {
    ALL_ICONS,
    ELEMENT_ICONS,
    ELEMENT_SYNONYMS,
    ICON_PALETTES,
    INTERMEDIATE_AMOUNT,
    LOCATION_AMOUNT,
    NON_ELEMENT_ITEMS,
    SUBSTITUTE_ICONS,
} from "../consts";
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
    elements: Record<string, { location: string; alt: string; }>
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

    /** @type {Map<string, {item: Item, isLocation: boolean}>} */
    #elementCache = new Map();
    /** @type {Set<string>} */
    #elementQueue = new Set();
    #elementQueueProcessing = -1; // Not processing
    /** @type {Record<string, ElementData>} */
    #elementData = $state({}); // Refresh Drawer on update

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
        /** @type {SlotData["elements"] | undefined} */
        const savedElements = await this.#client.storage.fetch(
            `elementipelago_${this.#client.name}_elements`,
            true,
        );
        if (savedElements) {
            this.#slotData.elements = savedElements;
        }

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
            this.queueElement(item, true);
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
    queueElement(item, isLocation = false) {
        let name = isLocation ? item.locationName : item.name;
        if (this.#elementData[name] !== undefined) {
            if (this.#elementData[name].location !== "Loading...") {
                return;
            }
        } else {
            const elem_id = elementNameToId(name);

            this.#elementCache.set(name, { item, isLocation });
            this.#elementData[name] = {
                elem_id,
                name: name,
                icon: "/sprites/elements/void.png",
                alt: "void",
                location: "Loading...",
                player: isLocation ? item.receiver.alias : item.sender.alias,
                game: isLocation ? item.receiver.game : item.sender.game,
            };
        }

        const newElement = this.#receivedElements.has(name) || this.#sentElements.has(name);
        if (newElement && !this.#elementQueue.has(name)) {
            this.#elementQueue.add(name);
            this.#processQueue();
        }
    }

    async #processQueue() {
        if (this.#elementQueueProcessing >= 0) {
            return;
        }
        this.#elementQueueProcessing = 0;

        do {
            const elementArray = Array.from(this.#elementQueue);
            elementArray.sort((a, b) => {
                /** @param {string} name */
                const getKind = (name) => {
                    const id = elementNameToId(name);
                    return id.kind;
                };

                const kindOrder = [ElementKind.INPUT, ElementKind.INTERMEDIATE, ElementKind.OUTPUT];
                const kindA = kindOrder.indexOf(getKind(a));
                const kindB = kindOrder.indexOf(getKind(b));
                const orderA = kindA === -1 ? 99 : kindA;
                const orderB = kindB === -1 ? 99 : kindB;
                return orderA - orderB;
            });

            const name = elementArray[this.#elementQueueProcessing];
            if (name !== undefined) {
                const entry = this.#elementCache.get(name);
                if (entry !== undefined) {
                    try {
                        await this.#addElementData(entry.item, entry.isLocation);
                        this.#elementQueue.delete(name);
                        this.#elementQueueProcessing = -1; // Reset to 0
                    } catch (e) {
                        console.error(
                            `Failed to process element "${name}", temporarily skipping`,
                            e,
                        );
                        if (this.#elementQueueProcessing < elementArray.length) {
                            await new Promise((r) => setTimeout(r, 10));
                        } else {
                            console.error("All elements in queue failing, stopping");
                            this.#elementQueueProcessing = -2;
                        }
                    }
                }
            } else {
                this.#elementQueueProcessing = -2;
            }
            this.#elementQueueProcessing++; // Increase error count or go to -1 on stop
        } while (this.#elementQueueProcessing >= 0);
    }

    /**
     * @param {string} userPrompt
     */
    async #queryOllama(userPrompt, newName = true, iconAndColor = true) {
        // Show only some icons and colors to the AI to see more random pictures
        let randomIcons = [...ALL_ICONS];
        randomIcons.sort((_a, _b) => Math.floor(Math.random() * 2) * 2 - 1);
        randomIcons = randomIcons.slice(0, 15);
        let randomColors = [...ICON_PALETTES];
        randomColors.sort((_a, _b) => Math.floor(Math.random() * 2) * 2 - 1);
        randomColors = randomColors.slice(0, 5);
        // System prompt must only contain general guidance, no specific direction or restrictions
        let systemPrompt = `You are an experienced alchemist and natural philosopher cataloguing newly discovered alchemical components in modern times. Your expertise is in${newName ? " naming them based on their unusual origins and recipes (NOT icons/colors)" : ""}${newName && iconAndColor ? ", and" : ""}${iconAndColor ? " categorizing their appearances with a single icon and color each." : ""}\nFor every request,${newName ? " produce a name that is a creative and archaic re-invention of the provided info" : ""}${newName && iconAndColor ? ", and" : ""}${iconAndColor ? " choose one each of the available icons and colors" : ""}.${iconAndColor ? `\nIcons (choose appearance-accurate): ${randomIcons.join(",")}\nColors: ${randomColors.join(",")}` : ""}`;

        /** @type {{ type: string; properties: { name?: object; icon?: object; color?: object; }; required: string[]; }} */
        const schema = {
            type: "object",
            properties: {
                name: { type: "string" },
                icon: { type: "string", enum: [...SUBSTITUTE_ICONS, ...ELEMENT_ICONS] },
                color: { type: "string", enum: ICON_PALETTES },
            },
            required: ["name", "icon", "color"],
        };
        if (!newName) {
            delete schema.properties.name;
            schema.required.shift();
        }
        if (!iconAndColor) {
            delete schema.properties.icon;
            delete schema.properties.color;
            schema.required.pop();
            schema.required.pop();
        }

        try {
            console.log(`$ ${userPrompt}`);
            const response = await fetch("http://localhost:11434/api/chat", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    model: localStorage.getItem("settings.ollama_model") ?? "gemma4:e2b",
                    messages: [
                        { role: "system", content: systemPrompt },
                        { role: "user", content: userPrompt },
                    ],
                    format: schema,
                    think: false,
                    stream: false,
                }),
            });
            const json = JSON.parse((await response.json()).message.content);
            console.log(`> ${JSON.stringify(json)}`);
            return json;
        } catch (e) {
            console.error("Ollama query failed", e);
            throw e;
        }
    }

    /**
     * @param {number} num
     */
    toRoman(num) {
        const numerals = {
            M: 1000,
            CM: 900,
            D: 500,
            CD: 400,
            C: 100,
            XC: 90,
            L: 50,
            XL: 40,
            X: 10,
            IX: 9,
            V: 5,
            IV: 4,
            I: 1,
        };
        let roman = "";
        for (let i in numerals) {
            // @ts-ignore
            while (num >= numerals[i]) {
                roman += i;
                // @ts-ignore
                num -= numerals[i];
            }
        }
        return roman;
    }

    /**
     * @param {string} loc
     * @param {string} name
     */
    nameLocationUniquely(loc, name) {
        const existingLocs = Object.values(this.#elementData)
            .filter((elem) => elem && elem.location !== "Loading..." && elem.name !== name)
            .map((elem) => elem.location);

        const romanRegex = /\s*([IVXLCDM]+)$/;
        let baseLoc = loc.replace(romanRegex, "");

        let i = 2;
        while (existingLocs.includes(loc)) {
            loc = `${baseLoc} ${this.toRoman(i)}`.trim();
            i++;
        }
        return loc;
    }

    /**
     * @param {Item} item
     * @param {boolean} isLocation
     */
    async #addElementData(item, isLocation) {
        let name = isLocation ? item.locationName : item.name;
        const displayName = isLocation ? item.name : item.locationName;

        const elem_id = elementNameToId(name);

        const ollamaEnabled = localStorage.getItem("settings.ollama_enabled") === "true";

        let loc, icon_name;
        const elementSlotData = this.slotData.elements?.[name];
        if (elementSlotData) {
            loc = elementSlotData.location;
            icon_name = elementSlotData.alt;
        } else if (!ollamaEnabled) {
            if (elem_id.kind === ElementKind.INTERMEDIATE) {
                loc = get_name();
                icon_name = iconForIntermediate(loc);
            } else {
                loc =
                    elem_id.kind === ElementKind.INTERMEDIATE || item.locationGame === "Archipelago"
                        ? get_name()
                        : displayName;

                icon_name = isLocation
                    ? iconForLocation(item.game, loc)
                    : iconForItem(item.game, loc);
            }
        } else {
            const startingElement = item.locationGame === "Archipelago";
            const fromOtherPlayer =
                item.sender && item.sender.slot !== this.#client.players.self.slot;
            const elementSynonym =
                ELEMENT_SYNONYMS[Math.floor(Math.random() * ELEMENT_SYNONYMS.length)];

            let response;
            if (startingElement) {
                // Elements available from the start get fixed seed names
                loc = get_name();
                response = await this.#queryOllama(
                    `Select an icon and color for the element "${loc}"`,
                    false,
                );
            } else if (fromOtherPlayer) {
                // Elements sent by other players are named based on the sender location name (player and world information is not used as it leads to repetitive names, Progressive items do not trigger this)
                response = await this.#queryOllama(
                    `A new element was discovered within the location/artifact "${item.locationName}".\nTransform the location's wording into an alchemical name, without omitting spaces nor adding common English terms. The final name MUST be 1-3 words long, and may use mixed case (e.g., "X of Y").\nAlso choose a relevant icon and an accurate color (or a fantasical color if the location is significantly unusual within its icon class).`,
                );
            } else {
                // All other elements are combinations with parents
                /** @type {ElementData | undefined} */
                let parent1;
                /** @type {ElementData | undefined} */
                let parent2;
                let fromRecipe = false;
                for (const [[i1, i2], ps] of this.#graph.recipes.entries()) {
                    for (const p of ps) {
                        if (elementIdToName(p) === name) {
                            const parent1Name = elementIdToName(i1);
                            const parent2Name = elementIdToName(i2);
                            if (parent1Name === undefined || parent2Name === undefined) {
                                throw new Error(
                                    `Element "${name}" parent names unexpectedly undefined`,
                                );
                            }
                            parent1 = this.#elementData[parent1Name];
                            parent2 = this.#elementData[parent2Name];
                            if (
                                parent1 &&
                                parent2 &&
                                parent1.location !== "Loading..." &&
                                parent2.location !== "Loading..."
                            ) {
                                fromRecipe = true;
                                break;
                            }
                        }
                    }
                    if (fromRecipe) break;
                }

                if (!fromRecipe) {
                    // Element is a compound, treat single parent element (once it has been named) as both parents
                    const compoundParent = this.#elementData[item.locationName];
                    if (compoundParent && compoundParent.location !== "Loading...") {
                        parent1 = compoundParent;
                        parent2 = parent1;
                    }
                }

                if (
                    !parent1 ||
                    parent1.location === "Loading..." ||
                    !parent2 ||
                    parent2.location === "Loading..."
                ) {
                    throw new Error(`Element "${name}" parents not yet processed'`);
                }
                const parent1Appearance = parent1.alt.replace("-", " ");
                const parent2Appearance = parent2.alt.replace("-", " ");
                const sameParents = parent1.elem_id === parent2.elem_id;

                const forOtherPlayer =
                    item.receiver && item.receiver.slot !== this.#client.players.self.slot;

                if (!forOtherPlayer) {
                    // Combinations that create new elements or items are named based on their parents' names
                    if (item.name === "Progressive Item Limit") {
                        // Progressive Item Limit allows more elements on screen at once
                        response = await this.#queryOllama(
                            `A by-product is created in the reaction from ${!sameParents ? "combining" : "the transmutation of"} "${parent1.location}" (icon/color: ${parent1Appearance})${!sameParents ? ` and "${parent2.location}" (icon/color: ${parent2Appearance})` : ""}, and it has an unlimiting effect.\nTransform and expand the provided names and effect (NOT the icons/colors) to create the name of the by-product. The final name MUST be 1-3 words long and use a synonym of an important word. It may also use adjectives, mixed case (e.g., "X of Y") and gibberish Latin (for shortening).\nAlso choose a fitting icon that is not used by the ingredients, and one of the ingredients' colors.`,
                        );
                    } else if (item.name === "Progressive Filter") {
                        // Progressive Filter adds indications of progress to the UI
                        response = await this.#queryOllama(
                            `A by-product is created in the reaction from ${!sameParents ? "combining" : "the transmutation of"} "${parent1.location}" (icon/color: ${parent1Appearance})${!sameParents ? ` and "${parent2.location}" (icon/color: ${parent2Appearance})` : ""}, and it has a filtering effect.\nTransform and expand the provided names and effect (NOT the icons/colors) to create the name of the by-product. The final name MUST be 1-3 words long and use a synonym of an important word. It may also use adjectives, mixed case (e.g., "X of Y") and gibberish Latin (for shortening).\nAlso choose a fitting icon that is not used by the ingredients, and one of the ingredients' colors.`,
                        );
                    } else if (!fromRecipe) {
                        // Compounds are by-products of combinations, they get a random icon and color
                        icon_name = ALL_ICONS[Math.floor(Math.random() * ALL_ICONS.length)];
                        if (SUBSTITUTE_ICONS.includes(icon_name)) {
                            icon_name = `${icon_name}-${ICON_PALETTES[Math.floor(Math.random() * ICON_PALETTES.length)]}`;
                        }
                        const compoundLook = icon_name.split("-");
                        compoundLook.reverse();
                        const compoundLookStr = compoundLook.join(" ");
                        const compoundLookVowel = /^[aeiou]/i.test(compoundLookStr);
                        response = await this.#queryOllama(
                            `A by-product is created in the chemical reaction that produces "${parent1.location}" (icon/color: ${parent1Appearance}), and it is different in composition, having the appearance of ${compoundLookVowel ? "an" : "a"} ${compoundLookStr}.\nCreate a believable alchemical name for the by-product that contains a tangential reference to the ingredient's name. The final name MUST be 1-3 words long and use a synonym of an important word. It may also use adjectives, mixed case (e.g., "X of Y") and gibberish Latin (for shortening).`,
                            true,
                            false,
                        );
                    } else {
                        // All other items are elements
                        response = await this.#queryOllama(
                            `A new ${elementSynonym} was created in the reaction from ${!sameParents ? "combining" : "the transmutation of"} "${parent1.location}" (icon/color: ${parent1Appearance})${!sameParents ? ` and "${parent2.location}" (icon/color: ${parent2Appearance})` : ""}.\nTransform and expand the ingredient name${!sameParents ? "s" : ""} (NOT the icons/colors) into a new alchemical name without omitting spaces. The final name MUST be 1-3 words long and use a synonym of an important word. It may also use adjectives, mixed case (e.g., "X of Y") and gibberish Latin (for shortening).\n${parent1Appearance !== parent2Appearance ? "Also choose the icon of one ingredient and the color of the other." : `Also choose an icon and color, one or both of which MUST be different from yet still be relevant to the ingredient${!sameParents ? "s" : ""}.`}`,
                        );
                    }
                } else {
                    // Combinations that send items to other players are named differently depending on what game they are sending to and if they are ingredients or not
                    let ingredient = false;
                    for (const [i1, i2] of this.#graph.recipes.keys()) {
                        if (elementIdToName(i1) === name || elementIdToName(i2) === name) {
                            ingredient = true;
                            break;
                        }
                    }

                    if (item.receiver.game !== "Elementipelago") {
                        if (!ingredient) {
                            // Those that send items to NON-Elementipelago players and are NOT ingredients are not given new names
                            loc = displayName;
                            response = await this.#queryOllama(
                                `Select an accurate icon and color for the item "${loc}"`,
                                false,
                            );
                        } else {
                            // Those that send items to NON-Elementipelago players but ARE ingredients are named based on their metadata
                            response = await this.#queryOllama(
                                `A by-product was created in the reaction from ${!sameParents ? "combining" : "the transmutation of"} "${parent1.location}" (icon/color: ${parent1Appearance})${!sameParents ? ` and "${parent2.location}" (icon/color: ${parent2Appearance})` : ""}, which is a component of a greater whole.\nTransform and expand the ingredient names (NOT the icons/colors) into a new alchemical name without omitting spaces. The name MUST allude to its greater whole (named "${item.name}") and be 1-3 words long. It may also use mixed case (e.g., "X of Y") and gibberish Latin (for shortening).\nAlso choose a fitting icon and color that are not used by the ingredients.`,
                            );
                        }
                    } else if (!ingredient) {
                        // Those that send items to Elementipelago players but are NOT ingredients are fully dedicated to the receiver
                        response = await this.#queryOllama(
                            `A new ${elementSynonym} is assumed to be created in the reaction from ${!sameParents ? "combining" : "the transmutation of"} "${parent1.location}" (icon/color: ${parent1Appearance})${!sameParents ? ` and "${parent2.location}" (icon/color: ${parent2Appearance})` : ""}. This undiscovered ${elementSynonym} has been dedicated to the alchemist "${item.receiver.alias}" (omit all forms of "Doodle God" and "Elementipelago").\nCreate its name based on the ingredient and dedication names (NOT the icons/colors), it should be vague/illusory enough so it can take any form once discovered. The final name MUST be 1-3 words long, and may also use adjectives and mixed case (e.g., "X of Y").\nAlso choose an icon and color, one or both of which MUST be different from the ingredient${!sameParents ? "s" : ""}.`,
                        );
                    } else {
                        // Those that send items to Elementipelago players and ARE ingredients pay homage to the receiver
                        response = await this.#queryOllama(
                            `A new ${elementSynonym} was created in the reaction from ${!sameParents ? "combining" : "the transmutation of"} "${parent1.location}" (icon/color: ${parent1Appearance})${!sameParents ? ` and "${parent2.location}" (icon/color: ${parent2Appearance})` : ""}. Its discovery has led to various advancements by the alchemist "${item.receiver.alias}" (omit all forms of "Doodle God" and "Elementipelago").\nTransform and expand the ingredient name${!sameParents ? "s" : ""} (NOT the icons/colors) and homage into a new alchemical name without omitting spaces. The final name MUST be 1-3 words long and use a synonym of an important word. It may also use adjectives, mixed case (e.g., "X of Y") and gibberish Latin (for shortening).\n${parent1Appearance !== parent2Appearance ? "Also choose the icon of one ingredient and the color of the other." : `Also choose an icon and color, one or both of which MUST be different from yet still be relevant to the ingredient${!sameParents ? "s" : ""}.`}`,
                        );
                    }
                }
            }

            if ("name" in response) {
                loc = response.name;
            }
            if ("icon" in response && "color" in response) {
                icon_name = response.icon;
                if (SUBSTITUTE_ICONS.includes(response.icon)) {
                    icon_name = `${response.icon}-${response.color}`;
                }
            }
        }

        if (loc === "Loading...") {
            throw new Error(`Element "${name}" unexpectedly not named'`);
        }
        if (!elementSlotData) {
            loc = this.nameLocationUniquely(loc, name);
        }
        this.#elementData[name] = {
            ...this.#elementData[name],
            icon: "/sprites/elements/" + icon_name + ".png",
            alt: icon_name,
            location: loc,
        };

        if (!elementSlotData && ollamaEnabled) {
            this.slotData.elements ??= {};
            this.slotData.elements[name] = {
                location: loc,
                alt: icon_name,
            };
            this.#client.storage
                .prepare(`elementipelago_${this.#client.name}_elements`, {})
                .update({ [name]: this.slotData.elements[name] })
                .commit();
        }
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
            this.queueElement(item);
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
            const name = this.#client.package.lookupLocationName("Elementipelago", location);
            this.#sentElements.add(name);

            const entry = this.#elementCache.get(name);
            if (entry !== undefined) {
                this.queueElement(entry.item, entry.isLocation);
            }
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
