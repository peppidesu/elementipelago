<script>
    import { get } from "svelte/store";
    import Drawer from "./lib/Drawer.svelte";
    import { dragging_elem as dragging_move_function } from "./lib/stores/dragging";
    import { pointerLoc } from "./lib/stores/pointer";

    function onpointermove(event) {
        pointerLoc.set({ x: event.clientX, y: event.clientY });

        let dmf = get(dragging_move_function);
        if (dmf != null) {
            dmf(event.clientX, event.clientY);
        }
    }

    function onpointerup(event) {
        dragging_move_function.set(null);
    }
</script>

<svelte:window {onpointermove} {onpointerup} />

<main style="width: 100vw; height: 100vh;">
    <Drawer />
</main>
