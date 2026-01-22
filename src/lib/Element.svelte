<script lang="js">
    import { mount } from "svelte";
    import PlacedElement from "./PlacedElement.svelte";
    import { pointerLoc } from "./stores/pointer";
    import { isExhausted, isExplorable, upgrades } from "./stores/apclient.svelte";
    import { get } from "svelte/store";
    import { ElementKind } from "./graph.js";
    import { sfx } from "../audio.js";

    /**
     * @import { ElementData } from "./stores/apclient.svelte";
     * @type {{ elem_data: ElementData, mount_func: any}}
     */
    const { elem_data, mount_func } = $props();
    let el;
    /**
     * @param {any} event
     */
    function onPointerDown(event) {
        sfx.drag_start();

        pointerLoc.set({ x: event.clientX, y: event.clientY });
        let { x, y } = get(pointerLoc);
        const rect = el.getBoundingClientRect();
        mount_func(x, y, elem_data.elem_id, x - rect.left, y - rect.top, true);
    }

    let is_bk = $derived(!isExplorable(elem_data.name) && upgrades.progressive_filter > 1);
    let is_exhausted = $derived(isExhausted(elem_data.name) && upgrades.progressive_filter > 0);
</script>

<li class="element {is_bk || is_exhausted ? 'disabled' : ''}" bind:this={el}>
    <img src={elem_data.icon} alt={elem_data.alt} draggable="false" onpointerdown={onPointerDown} />
    <span class="info">
        <h1>{elem_data.location}</h1>
        <p>from {elem_data.player}</p>
        <p>{elem_data.name}</p>
    </span>

    <span class="icon">
        {#if is_exhausted}
            <img src="/sprites/ui/check.png" alt="exhausted" />
        {:else if is_bk}
            <img src="/sprites/ui/burger.png" alt="BK" />
        {/if}
    </span>
</li>

<style>
</style>
