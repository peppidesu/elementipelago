import { mount, unmount } from "svelte";
import { SvelteMap } from "svelte/reactivity";
import { get, writable } from "svelte/store";
import PlacedElement from "../components/PlacedElement.svelte";

export const dragging_elem = writable(null);
export const mounted = new SvelteMap();

export function moveDragging(x, y) {
    let dmf = get(dragging_elem);
    if (dmf != null) {
        dmf.mfunc(x, y);
    }
}
let nextMountIdx = $state(0);

/**
 * @import { ElementID } from "../graph";
 * @param {number} x
 * @param {number} y
 * @param {ElementID} elem_id
 */
export function mountElem(x, y, elem_id, offsetx = 0, offsety = 0, attach = false) {
    let placed = mount(PlacedElement, {
        target: document.getElementById("playfield"),
        props: {
            x: x,
            y: y,
            elem_id: elem_id,
            offsetx: offsetx,
            offsety: offsety,
            attach: attach,
            index: nextMountIdx,
        },
    });

    mounted.set(nextMountIdx, placed);
    nextMountIdx += 1;
}

export function unmountElem(el, idx) {
    unmount(el, { outro: true });
    mounted.delete(idx);
}
