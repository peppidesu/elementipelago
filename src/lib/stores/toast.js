import { get, writable } from "svelte/store";
import { apclient, getElementData } from "./apclient.svelte";
import { iconForItem, iconForLocation } from "../machine-learning/iconml";

/**
 * @typedef {{
    title: string,
    description: string,
    image: string,
   }} ToastMessage
 */

export const toast_queue = writable([]);

const initialized = writable(false);
apclient.subscribe((client) => {
  if (get(initialized)) return;
  initialized.set(true);

  client.items.on("itemsReceived", (items) => {
    let image = iconForItem(items[0]);
    let first_item = items[0].locationName;
    let others_suffix = items.length > 1 ? ` + ${items.length - 1} more` : "";
    let description = `${first_item}${others_suffix}`;

    toast_queue.update((queue) => {
      queue.push({
        title: "New elements available!",
        description,
        image: "/sprites/elements/" + image + ".png",
      });
      return queue;
    });
  });
});
