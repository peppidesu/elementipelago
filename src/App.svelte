<script>
    import Drawer from "./lib/Drawer.svelte";
    import { pointerLoc } from "./lib/stores/pointer";

    function onpointermove(event) {
        pointerLoc.set({ x: event.clientX, y: event.clientY });
    }
</script>

<main {onpointermove}>
    <Drawer />
</main>
