import { derived, get, readable, writable } from "svelte/store";
import { Client } from "archipelago.js";
import { createSubscriber, SvelteMap, SvelteSet } from "svelte/reactivity";
import { element_to_name, parse_element } from "../../utils";
import { iconForItem, iconForLocation } from "../machine-learning/iconml";
/**
 * @import { Graph } from "../graph"
 * @import { Writable, Readable } from "svelte/store";
 * @import { ElementID } from "./graph";
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
 * @type {Readable<SvelteMap<string, ElementData>>}
 */
export const elementData = readable(new SvelteMap());
export const receivedElements = readable(new SvelteSet());
export const sentElements = readable(new SvelteSet());

export const drawerElements = derived(
  [receivedElements, sentElements, slotdata],
  ([receivedElements, sentElements, slotdata]) => {
    if (slotdata.compounds_are_ingredients) {
      return receivedElements.union(sentElements);
    } else {
      return receivedElements;
    }
  },
);

export const reachableElements = derived(
  [graph, drawerElements],
  ([gr, drawerElements], _, update) => {
    for (const [[i1, i2], ps] of gr.recipes.entries()) {
      const i1_name = element_to_name(i1);
      const i2_name = element_to_name(i2);

      if (drawerElements.has(i1_name) && drawerElements.has(i2_name)) {
        update((self) => {
          for (const p of ps) {
            self.add(element_to_name(p));
          }
          return self;
        });
      }
    }
  },
  new SvelteSet(),
);

export const discoverableElements = derived(
  [reachableElements, sentElements],
  ([reachableElements, sentElements]) => {
    return reachableElements.difference(sentElements);
  },
);

export async function initElementStores() {
  let client = get(apclient);
  let scoutedLocations = client.scout(client.room.allLocations, 0);
  await extendReceivedElements(client.items.received);
  extendSentElements(client.room.checkedLocations);

  let inner = get(elementData);
  for (const item of await scoutedLocations) {
    if (!inner.has(item.locationName)) {
      let icon_name = iconForLocation(item);
      let elem_id = parse_element(item.locationName);
      inner.set(item.locationName, {
        elem_id,
        name: item.locationName,
        icon: "/sprites/elements/" + icon_name + ".png",
        alt: icon_name,
        location: item.name,
        player: item.receiver.alias,
        game: item.receiver.game,
      });
    }
  }

  client.items.on("itemsReceived", extendReceivedElements);
  client.room.on("locationsChecked", extendSentElements);
}

/**
 * @param {Item[]} items
 */
async function extendReceivedElements(items) {
  let el_data = get(elementData);
  let el_set = get(receivedElements);
  for (const item of items) {
    let icon_name = iconForItem(item);
    let elem_id = parse_element(item.name);

    el_data.set(item.name, {
      elem_id,
      name: item.name,
      icon: "/sprites/elements/" + icon_name + ".png",
      alt: icon_name,
      location: item.locationName,
      player: item.sender.alias,
      game: item.sender.game,
    });

    el_set.add(item.name);
  }
}

/**
 * @param {number[]} locations
 */
function extendSentElements(locations) {
  let el_set = get(sentElements);
  let client = get(apclient);
  for (const location of locations) {
    el_set.add(client.package.lookupLocationName("Elementipelago", location));
  }
}
