<script lang="js">
    import { onDestroy, onMount } from "svelte";
    import { dragging_elem } from "./stores/dragging";
    import { get } from "svelte/store";

    const { x, y, elem } = $props();

    let sx = $state(x);
    let sy = $state(y);

    onMount(() => {
        dragging_elem.set((lx, ly) => {
            sx = lx;
            sy = ly;
        });
    });
</script>

<div style="left: {sx}px; top: {sy}px;">TEST</div>

<style>
    div {
        position: absolute;
        background-color: green;
    }
</style>
