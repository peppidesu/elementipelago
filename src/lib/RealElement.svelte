<script lang="js">
    import { onDestroy, onMount } from "svelte";
    import { dragging_elem } from "./stores/dragging";
    import { get } from "svelte/store";
    import { pointerLoc } from "./stores/pointer";

    const { x, y, elem, offsetx: localx, offsety: localy } = $props();

    let ox = localx;
    let oy = localy;
    let sx = $state(x - ox);
    let sy = $state(y - oy);

    onMount(() => {
        dragging_elem.set((lx, ly) => {
            sx = lx - ox;
            sy = ly - oy;
        });
    });

    function onpointerdown(e) {
        ox = e.layerX;
        oy = e.layerY;
        dragging_elem.set((lx, ly) => {
            sx = lx - ox;
            sy = ly - oy;
        });
    }
</script>

<div {onpointerdown} style="left: {sx}px; top: {sy}px;">
    <img src={elem.src} alt="" draggable="false" />
    <p>{elem.name}</p>
</div>

<style>
    div {
        position: absolute;
        background-color: green;
        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
        cursor: grab;

        list-style-type: none;
        border-width: 3px;
        border-style: solid;

        p {
            user-select: none;
            touch-action: none; /* IMPORTANT for mobile */
        }

        img {
            user-select: none;
            touch-action: none; /* IMPORTANT for mobile */
        }
    }
</style>
