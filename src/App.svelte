<script lang="javascript">
    import { get } from "svelte/store";
    import Drawer from "./lib/Drawer.svelte";
    import { dragging_elem as dragging_move_function } from "./lib/stores/dragging";
    import { pointerLoc } from "./lib/stores/pointer";
    import { mount, unmount } from "svelte";
    import { apclient, graph } from "./lib/stores/apclient";
    import PlacedElement from "./lib/PlacedElement.svelte";
    import {
        element_to_location_id,
        element_to_name,
        parse_element,
    } from "./utils";
    import Login from "./lib/Login.svelte";
    import Playfield from "./lib/Playfield.svelte";
    import { icon_cache } from "./lib/stores/icon_cache";
    import {
        iconForItem,
        iconForLocation,
    } from "./lib/machine-learning/iconml";

    const mounted = new Map();

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
            unmount(dropped_el, { outro: true });
            mounted.delete(dropped_el_index);
            return;
        }

        let gr = get(graph);
        let dropped_elem_id = { ...dropped_el.elem_id };

        for (const [idx, other_el] of mounted) {
            // don't check collision with itself
            if (other_el == dropped_el) {
                continue;
            }
            let other_el_rect = other_el.get_rect();
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
                    mountElem(
                        (dropped_el_rect.x + other_el_rect.x) / 2,
                        (dropped_el_rect.y + other_el_rect.y) / 2,
                        elem_data,
                    );
                }

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

        let client = get(apclient);

        let scouted = client.scout(client.room.allLocations, 0);
        let items = client.items.received;

        let cache = get(icon_cache);
        items.forEach((item) => {
            let icon_name = iconForItem(item);
            let location_name = item.locationName;
            if (item.sender.slot === item.receiver.slot) {
                console.log(item.name, item.locationName);
                location_name = item.name;
            }
            cache.set(item.name, {
                icon: "/sprites/elements/" + icon_name + ".png",
                alt: icon_name,
                name: location_name,
                player: item.sender.alias,
                game: item.sender.game,
            });
        });
        (await scouted).forEach((item) => {
            let icon_name = iconForLocation(item);
            let item_name = item.name;
            if (item.sender.slot === item.receiver.slot) {
                console.log(item.name, item.locationName);
                item_name = item.locationName;
            }
            cache.set(item.locationName, {
                icon: "/sprites/elements/" + icon_name + ".png",
                alt: icon_name,
                name: item_name,
                player: item.receiver.alias,
                game: item.receiver.game,
            });
        });
    }

    let next_index = 0;

    export function mountElem(
        x,
        y,
        elem_data,
        offsetx = 0,
        offsety = 0,
        attach = false,
    ) {
        console.log("spawning element:", elem_data);
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
                display_data: get(icon_cache).get(elem_data.name) ??
                    get(icon_cache).get("Make " + elem_data.name) ?? {
                        icon: "void",
                        alt: "void",
                        name: elem_data.name,
                        game: "",
                        player: "",
                    },
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
    <Drawer mount_func={mountElem} />
    <Playfield bind:handle_dropped={on_dropped} />
{/if}
