import { writable } from "svelte/store";
import { getElementData } from "./apclient.svelte";
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

/**
 * @param {Item[]} items
 */
export function sendElementToasts(items) {
    if (items.length > 0) {
        toast_queue.update((queue) => {
            queue.push(elementsReceivedMessage(items));
            return queue;
        });
    }
}

/**
 * @param {*} oldUpgrades
 * @param {*} newUpgrades
 */
export function sendUpgradeToasts(oldUpgrades, newUpgrades) {
    toast_queue.update((queue) => {
        for (const key in oldUpgrades) {
            let count = newUpgrades[key] - oldUpgrades[key];
            if (count == 0) continue;
            queue.push(upgradeReceivedMessage(key, count));
        }
        return queue;
    });
}

/**
    @param {Item[]} elements
    @returns {{title: string, description: string, image: string}}
*/
function elementsReceivedMessage(elements) {
    const first_item_data = getElementData().get(elements[0].name);
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

const upgradeKeyToItem = {
    field_size: "Progressive Item Limit",
    progressive_filter: "Progressive Filter",
};
/**
    @param {string} upgrade
    @param {number} count
    @returns {{title: string, description: string, image: string}}
*/
function upgradeReceivedMessage(upgrade, count) {
    return {
        title: "Upgrade received!",
        description: `${upgradeKeyToItem[upgrade]} (x${count})`,
        image: "/sprites/elements/upgrade.png",
    };
}
