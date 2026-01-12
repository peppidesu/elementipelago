<script lang="javascript">
    import { get } from "svelte/store";
    import Drawer from "./lib/Drawer.svelte";
    import { dragging_elem as dragging_move_function } from "./lib/stores/dragging";
    import { pointerLoc } from "./lib/stores/pointer";
    import { mount } from "svelte";
    import { apclient, graph } from "./lib/stores/apclient";
    import PlacedElement from "./lib/PlacedElement.svelte";
    import { element_to_location_id, element_to_name } from "./utils";
    import Login from "./lib/Login.svelte";
    import Playfield from "./lib/Playfield.svelte";

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

    /**
     * @param {any} event
     */
    function onpointerup(event) {
        let dmf = get(dragging_move_function);
        if (dmf == null) {
            return;
        }
        let dropped_el = dmf.self;
        let dropped_el_rect = dropped_el.getBoundingClientRect();

        dragging_move_function.set(null);

        // if overlaps with the drawer
        let drawer_rect = document
            .getElementById("drawer")
            .getBoundingClientRect();

        if (intersect(dropped_el_rect, drawer_rect)) {
            // element dropped inside of the drawer should be removed
            dropped_el.remove();
        }

        let placed_elements = document.getElementById("playfield").children;
        let gr = get(graph);
        let dropped_elem_id = { ...dropped_el.elem_id };

        for (const other_el of placed_elements) {
            // don't check collision with itself
            if (other_el == dropped_el) {
                continue;
            }
            let other_el_rect = other_el.getBoundingClientRect();
            if (intersect(dropped_el_rect, other_el_rect)) {
                // Get recipe_elem for both dropped_el and element
                // @ts-ignore
                let other_elem_id = other_el.elem_id;

                // Find the combination in the graph
                let products =
                    gr.recipes.get([dropped_elem_id, other_elem_id]) ||
                    gr.recipes.get([other_elem_id, dropped_elem_id]);

                if (products == undefined) {
                    continue;
                }

                let locations = products.map(
                    (/** @type {import("./utils").ElementID} */ val) =>
                        element_to_location_id(val),
                );
                get(apclient).check(...locations);

                for (const prod of products) {
                    // spawn element with type product
                    const elem_data = {
                        name: element_to_name(prod),
                        elem_id: prod,
                    };
                    mount(PlacedElement, {
                        target: document.getElementById("playfield"),
                        props: {
                            x: (dropped_el_rect.x + other_el_rect.x) / 2,
                            y: (dropped_el_rect.y + other_el_rect.y) / 2,
                            elem_data: elem_data,
                            offsetx: 0,
                            offsety: 0,
                            attach: false,
                        },
                    });
                }

                // remove dropped, and other
                dropped_el.remove();
                other_el.remove();

                // no need to continue checking
                break;
            }
        }
        document.getElementById("playfield").handle_dropped();
    }

    let connected = false;

    async function handleLogin() {
        connected = true;
    }
</script>

<svelte:window {onpointermove} {onpointerup} />

{#if !connected}
    <Login onSubmit={handleLogin} />
{:else}
    <Drawer />
    <Playfield />
{/if}
