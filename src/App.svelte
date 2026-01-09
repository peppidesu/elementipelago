<script lang="javascript">
    import { get } from "svelte/store";
    import Drawer from "./lib/Drawer.svelte";
    import { dragging_elem as dragging_move_function } from "./lib/stores/dragging";
    import { pointerLoc } from "./lib/stores/pointer";
    import { unmount } from "svelte";

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

        for (const element of placed_elements) {
            // don't check collision with itself
            if (element == dropped_el) {
                continue;
            }
            if (intersect(dropped_el_rect, element.getBoundingClientRect())) {
                console.log("dropped element intersected with", element);
                console.log(dropped_el);
            }
        }
    }
</script>

<svelte:window {onpointermove} {onpointerup} />

<main style="width: 100vw; height: 100vh;">
    <Drawer />
    <div id="playfield"></div>
</main>
