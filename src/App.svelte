<script lang="javascript">
    import { get } from "svelte/store";
    import Drawer from "./lib/Drawer.svelte";
    import { dragging_elem as dragging_move_function } from "./lib/stores/dragging";
    import { pointerLoc } from "./lib/stores/pointer";
    import { mount, unmount } from "svelte";
    import { apclient, graph, slotdata } from "./lib/stores/apclient";
    import RealElement from "./lib/RealElement.svelte";
    import { elem_to_location_id, elem_to_name, name_to_kind } from "./utils";

    function intersect(rect1, rect2) {
        return (
            rect1.left < rect2.right &&
            rect1.right > rect2.left &&
            rect1.top < rect2.bottom &&
            rect1.bottom > rect2.top
        );
    }

    function onpointermove(event) {
        pointerLoc.set({ x: event.clientX, y: event.clientY });

        let dmf = get(dragging_move_function);
        if (dmf != null) {
            dmf.mfunc(event.clientX, event.clientY);
        }
    }

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
        let dropped_relem = { ...dropped_el.recipe_elem };

        for (const element of placed_elements) {
            // don't check collision with itself
            if (element == dropped_el) {
                console.log("element was dropped_el", element, dropped_el);
                continue;
            }
            let el_rect = element.getBoundingClientRect();
            if (intersect(dropped_el_rect, el_rect)) {
                // Get recipe_elem for both dropped_el and element
                let other_relem = element.recipe_elem;
                // Find the combination in the graph
                let products =
                    gr.recipes.get([dropped_relem, other_relem]) ||
                    gr.recipes.get([other_relem, dropped_relem]);

                if (products == undefined) {
                    continue;
                }

                let locations = products.map((val) => elem_to_location_id(val));
                get(apclient).check(...locations);

                for (const prod of products) {
                    // spawn element with type product
                    const elem = {
                        name: elem_to_name(prod),
                        src: "",
                        recipe_elem: prod,
                    };
                    mount(RealElement, {
                        target: document.getElementById("playfield"),
                        props: {
                            x: (dropped_el_rect.x + el_rect.x) / 2,
                            y: (dropped_el_rect.y + el_rect.y) / 2,
                            elem,
                            offsetx: 0,
                            offsety: 0,
                            attach: false,
                        },
                    });
                }

                // remove dropped, and other
                dropped_el.remove();
                element.remove();

                // no need to continue checking
                break;
            }
        }
    }
</script>

<svelte:window {onpointermove} {onpointerup} />

<main style="width: 100vw; height: 100vh;">
    <Drawer />
    <div id="playfield"></div>
</main>
