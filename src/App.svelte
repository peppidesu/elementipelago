<script>
    import { get } from "svelte/store";
    import Drawer from "./lib/Drawer.svelte";
    import { dragging_elem as dragging_move_function } from "./lib/stores/dragging";
    import { pointerLoc } from "./lib/stores/pointer";

    function onpointermove(event) {
        pointerLoc.set({ x: event.clientX, y: event.clientY });

        let dmf = get(dragging_move_function);
        if (dmf != null) {
            console.log("dmf, ", dmf);
            dmf(event.clientX, event.clientY);
        }
    }

    function onpointerup(event) {
        dragging_move_function.set(null);
    }
</script>

<main {onpointermove}>
    <Drawer />
</main>
