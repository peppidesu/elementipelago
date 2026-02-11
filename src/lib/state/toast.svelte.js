import { get } from "svelte/store";
import { apstore } from "./apclient.svelte";

/**
    @typedef {{
        title: string,
        description: string,
        image: string,
    }} ToastMessage
    @import { Item } from 'archipelago.js'
*/

export const toast_queue = $state([]);

/**
 * @param {Item[]} items
 */
apstore.on("elementsReceived", (elements) => {
    if (elements.length > 0) {
        toast_queue.push(elementsReceivedMessage(elements));
    }
});

/**
 * @param {*} oldUpgrades
 * @param {*} newUpgrades
 */
apstore.on("upgradesReceived", (oldUpgrades, newUpgrades) => {
    for (const key in oldUpgrades) {
        let count = newUpgrades[key] - oldUpgrades[key];
        console.log(key, count);
        if (count === 0) continue;
        toast_queue.push(upgradeReceivedMessage(key, count));
    }
});

/**
    @param {Item[]} elements
    @returns {{title: string, description: string, image: string}}
*/
function elementsReceivedMessage(elements) {
    const first_item_data = apstore.elementData[elements[0].name];
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
