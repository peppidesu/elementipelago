<script lang="js">
    import { mount } from "svelte";
    import PlacedElement from "./PlacedElement.svelte";
    import { pointerLoc } from "./stores/pointer";
    import { get } from "svelte/store";
    import { ElementKind } from "./graph.js";
    import { sfx } from "../audio.js";

    /**
     * @import { ElementData } from "./stores/apclient";
     * @type {{ elem_data: ElementData, mount_func: any}}
     */
    const { elem_data, mount_func } = $props();
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
        src={elem_data.icon}
        alt={elem_data.alt}
        draggable="false"
        onpointerdown={onPointerDown}
    />
    <span>
        {#if elem_data.elem_id.kind !== ElementKind.INTERMEDIATE}
            <h1>{elem_data.location}</h1>
            <p>from {elem_data.player}</p>
            <p>{elem_data.name}</p>
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
                font-weight: bold;
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
