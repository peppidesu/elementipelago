<script lang="js">
    import { mount } from "svelte";
    import PlacedElement from "./PlacedElement.svelte";
    import { pointerLoc } from "./stores/pointer";
    import { isExhausted, isExplorable } from "./stores/apclient.svelte";
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

    let is_bk = $derived(!isExplorable(elem_data.name));
    let is_exhausted = $derived(isExhausted(elem_data.name));
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
    .element {
        display: flex;
        align-items: center;
        image-rendering: pixelated;
        gap: 15px;

        list-style-type: none;
        margin-inline: 10px;
        padding-block: 10px;

        &:not(:last-child) {
            border-bottom: 2px #c0c0c0 solid;
        }
        > span.info {
            min-width: 0;
            flex-grow: 1;
            > h1 {
                font-weight: bold;
                margin: 0px;

                margin-bottom: 2px;
                padding-bottom: 3px;

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

            user-select: none;
            touch-action: none; /* IMPORTANT for mobile */
        }
        &.disabled {
            > span.info > * {
                color: #686868;
            }
            > img {
                filter: saturate(0.3) contrast(0.5) brightness(1.4);
            }
        }
        > span.icon {
            display: flex;
            height: 100%;
            flex-direction: column;
            justify-content: start;

            > img {
                height: 32px;
                margin-left: auto;
            }
        }
    }
</style>
