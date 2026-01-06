<script lang="js">
    import { onMount } from "svelte";
    import { dragging_elem } from "./stores/dragging";
    import { get } from "svelte/store";

    const { x, y, elem } = $props();

    let sx = $state(x);
    let sy = $state(y);

    onMount(() => {
        dragging_elem.set((x, y) => {
            (sx = x), (sy = y);
        });
        console.log(get(dragging_elem));
    });
</script>

<div style="left: {x}px; top: {y}px;">TEST</div>

<style>
    div {
        position: absolute;
        background-color: green;
    }
</style>
