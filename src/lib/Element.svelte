<script lang="js">
    import { mount } from "svelte";
    import PlacedElement from "./PlacedElement.svelte";
    import { pointerLoc } from "./stores/pointer";
    import { get } from "svelte/store";
    import { ElementKind } from "../utils";
    import { sfx } from "../audio.js";

    const { elem_data, display_data, mount_func } = $props();
    let el;

    export const elem_id = elem_data.elem_id;
    /**
     * @param {any} event
     */
    function onPointerDown(event) {
        sfx.drag_start();

        pointerLoc.set({ x: event.clientX, y: event.clientY });
        let { x, y } = get(pointerLoc);
        const rect = el.getBoundingClientRect();
        mount_func(x, y, elem_data, x - rect.left, y - rect.top, true);
    }
</script>

<li class="element" bind:this={el}>
    <img
        src={display_data.icon}
        alt={display_data.alt}
        draggable="false"
        onpointerdown={onPointerDown}
    />
    <span>
        {#if elem_data.elem_id.kind !== 2}
            <h1>{display_data.name}</h1>
            <p>{elem_data.name}</p>
            <p>{display_data.player}</p>
        {:else}
            <h1>{elem_data.name}</h1>
        {/if}
    </span>
</li>

<style>
    .element {
        display: flex;
        align-items: center;

        gap: 15px;

        list-style-type: none;
        margin-inline: 5px;
        padding-block: 5px;

        &:not(:last-child) {
            border-bottom: 2px #c0c0c0 solid;
        }
        > span {
            min-width: 0;
            > h1 {
                font-weight: normal;
                margin: 0px;
                margin-bottom: 5px;
                text-align: left;
                font-size: 1em;
                text-overflow: ellipsis;
                white-space: nowrap;
                overflow: hidden;
            }
            > p {
                color: #484848;
                margin: 0px;
                text-align: left;
                font-size: 0.75em;
                text-overflow: ellipsis;
                white-space: nowrap;
                overflow: hidden;
            }
        }
        > img {
            width: 96px;
            height: 96px;
            cursor: grab;
            image-rendering: pixelated;
            user-select: none;
            touch-action: none; /* IMPORTANT for mobile */
        }
    }
</style>
