import { get, writable } from "svelte/store";
import { apclient, getElementData } from "./apclient.svelte";
import { iconForItem, iconForLocation } from "../machine-learning/iconml";
import { NON_ELEMENT_ITEMS } from "../../consts";

/**
    @typedef {{
        title: string,
        description: string,
        image: string,
    }} ToastMessage
    @import { Item } from 'archipelago.js'
*/

export const toast_queue = writable([]);

const initialized = writable(false);

/**
 * @param {Item[]} items
 */
export function sendReceivedToasts(items) {
    console.log(items);
    let elements = items.filter((item) => {
        return item.id >= NON_ELEMENT_ITEMS;
    });

    let upgrades = items.filter((item) => {
        return item.id < NON_ELEMENT_ITEMS;
    });

    toast_queue.update((queue) => {
        queue.push(elementsReceivedMessage(elements));
        return queue;
    });

    toast_queue.update((queue) => {
        for (const upgrade of upgrades) {
            queue.push(upgradeReceivedMessage(upgrade));
        }

        return queue;
    });
}

/**
    @param {Item[]} elements
    @returns {{title: string, description: string, image: string}}
*/
function elementsReceivedMessage(elements) {
    if (elements.length == 0) return;
    const first_item_data = getElementData().get(elements[0].name)
    let image = first_item_data.icon;
    let first_item = first_item_data.location;
    let others_suffix = elements.length > 1 ? ` + ${elements.length - 1} more` : "";
    let description = `${first_item}${others_suffix}`;

    return {
        title: "New elements available!",
        description,
        image: image,
    };
}

/**
    @param {Item} upgrade
    @returns {{title: string, description: string, image: string}}
*/
function upgradeReceivedMessage(upgrade) {
    let image = iconForItem(upgrade.game, upgrade.name);

    return {
        title: "Upgrade received!",
        description: upgrade.name,
        image: "/sprites/elements/" + image + ".png",
    };
}
