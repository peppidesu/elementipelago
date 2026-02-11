<script lang="javascript">
    import { unmount } from "svelte";
    import { get } from "svelte/store";
    import { mounted, mountElem, dragging_elem, unmountElem } from "../state/playfield.svelte";
    import { sfx } from "../audio";
    import { apstore } from "../state/apclient.svelte";
    import { elementIdToLocation } from "../utils";

    /**
     * @param {DOMRect} rect1
     * @param {DOMRect} rect2
     */
    function intersect(rect1, rect2) {
        return (
            rect1.left < rect2.right &&
            rect1.right > rect2.left &&
            rect1.top < rect2.bottom &&
            rect1.bottom > rect2.top
        );
    }

    /**
     * @param {any} event
     */
    export function ondrop(event) {
        let dmf = get(dragging_elem);
        if (dmf == null) {
            return;
        }
        let dropped_el_index = dmf.index;
        let dropped_el = mounted.get(dropped_el_index);
        let dropped_el_rect = dropped_el.get_rect();

        dragging_elem.set(null);

        // if overlaps with the drawer
        let drawer_rect = document.getElementById("drawer").getBoundingClientRect();

        if (intersect(dropped_el_rect, drawer_rect)) {
            // element dropped inside of the drawer should be removed
            sfx.trash();
            unmountElem(dropped_el, dropped_el_index);
            return;
        }

        sfx.drag_end();

        let dropped_elem_id = { ...dropped_el.get_elem_id() };

        for (const [idx, other_el] of mounted) {
            // don't check collision with itself
            if (other_el == dropped_el) {
                continue;
            }
            let other_el_rect = other_el.get_rect();
            if (intersect(dropped_el_rect, other_el_rect)) {
                // Get recipe_elem for both dropped_el and element
                // @ts-ignore
                let other_elem_id = other_el.get_elem_id();

                // Find the combination in the graph
                let products =
                    apstore.graph.recipes.get([dropped_elem_id, other_elem_id]) ||
                    apstore.graph.recipes.get([other_elem_id, dropped_elem_id]);

                if (products == undefined) {
                    continue;
                }

                let locations = products.map((/** @type {import("../graph").ElementID} */ val) =>
                    elementIdToLocation(val),
                );
                apstore.client.check(...locations);

                for (const prod of products) {
                    // spawn element with type product
                    mountElem(
                        (dropped_el_rect.x + other_el_rect.x) / 2,
                        (dropped_el_rect.y + other_el_rect.y) / 2,
                        prod,
                    );
                }

                setTimeout(() => sfx.bubble(), 100);

                // remove dropped, and other
                unmount(dropped_el, { outro: true });
                unmount(other_el, { outro: true });
                mounted.delete(idx);
                mounted.delete(dropped_el_index);

                // no need to continue checking
                break;
            }
        }
        Array.from(mounted)
            .toSorted(([_a, a], [_b, b]) => Number(a.get_z_index()) - Number(b.get_z_index()))
            .forEach(([_num, ele], idx) => ele.set_z_index(String(idx + 1)));
    }
</script>

<div id="playfield"></div>

<style>
    #playfield {
        flex-grow: 1;
    }
</style>
