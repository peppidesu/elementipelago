<script lang="javascript">
    import { get } from "svelte/store";
    import Drawer from "./lib/Drawer.svelte";
    import { dragging_elem as dragging_move_function } from "./lib/stores/dragging";
    import { pointerLoc } from "./lib/stores/pointer";
    import { mount, unmount } from "svelte";
    import {
        apclient,
        graph,
        getElementData,
        initElementStores,
    } from "./lib/stores/apclient.svelte";
    import PlacedElement from "./lib/PlacedElement.svelte";
    import {
        element_to_location_id,
        element_to_name,
        parse_element,
    } from "./utils";
    import Login from "./lib/Login.svelte";
    import Playfield from "./lib/Playfield.svelte";
    import { sfx } from "./audio.js";
    import { initGraph } from "./lib/graph";
    import Toast from "./lib/Toast.svelte";
    import { SvelteMap } from "svelte/reactivity";

    const mounted = new SvelteMap();

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
     * @param {{ clientX: any; clientY: any; }} event
     */
    function onpointermove(event) {
        pointerLoc.set({ x: event.clientX, y: event.clientY });

        let dmf = get(dragging_move_function);
        if (dmf != null) {
            dmf.mfunc(event.clientX, event.clientY);
        }
    }

    let on_dropped;

    /**
     * @param {any} event
     */
    function onpointerup(event) {
        let dmf = get(dragging_move_function);
        if (dmf == null) {
            return;
        }
        let dropped_el_index = dmf.index;
        let dropped_el = mounted.get(dropped_el_index);
        let dropped_el_rect = dropped_el.get_rect();

        dragging_move_function.set(null);

        // if overlaps with the drawer
        let drawer_rect = document
            .getElementById("drawer")
            .getBoundingClientRect();

        if (intersect(dropped_el_rect, drawer_rect)) {
            // element dropped inside of the drawer should be removed
            sfx.trash();
            unmount(dropped_el, { outro: true });
            mounted.delete(dropped_el_index);
            return;
        }

        sfx.drag_end();

        let gr = get(graph);
        let elem_data_map = getElementData();
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
                    gr.recipes.get([dropped_elem_id, other_elem_id]) ||
                    gr.recipes.get([other_elem_id, dropped_elem_id]);

                if (products == undefined) {
                    continue;
                }

                let locations = products.map(
                    (/** @type {import("./lib/graph").ElementID} */ val) =>
                        element_to_location_id(val),
                );
                get(apclient).check(...locations);

                for (const prod of products) {
                    // spawn element with type product
                    const elem_data = elem_data_map.get(element_to_name(prod));
                    mountElem(
                        (dropped_el_rect.x + other_el_rect.x) / 2,
                        (dropped_el_rect.y + other_el_rect.y) / 2,
                        elem_data,
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
        on_dropped(mounted);
    }

    let connected = false;
    async function handleLogin() {
        connected = true;
        initGraph();
        await initElementStores();
    }

    let next_index = 0;
    /**
     * @import { ElementData } from "./lib/stores/apclient";
     * @param {number} x
     * @param {number} y
     * @param {ElementData} elem_data
     */
    export function mountElem(
        x,
        y,
        elem_data,
        offsetx = 0,
        offsety = 0,
        attach = false,
    ) {
        let placed = mount(PlacedElement, {
            target: document.getElementById("playfield"),
            props: {
                x: x,
                y: y,
                elem_data: elem_data,
                offsetx: offsetx,
                offsety: offsety,
                attach: attach,
                index: next_index,
            },
        });

        mounted.set(next_index, placed);
        next_index += 1;
    }
</script>

<svelte:window {onpointermove} {onpointerup} />

{#if !connected}
    <Login onSubmit={handleLogin} />
{:else}
    <Drawer mount_func={mountElem} mounted_elements={mounted} />
    <Playfield bind:handle_dropped={on_dropped} />
    <Toast />
{/if}
