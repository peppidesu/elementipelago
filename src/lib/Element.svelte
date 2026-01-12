<script lang="js">
    import { mount } from "svelte";
    import PlacedElement from "./PlacedElement.svelte";
    import { pointerLoc } from "./stores/pointer";
    import { get } from "svelte/store";

    const { elem_data, display_data, mount_func } = $props();
    let el;

    export const elem_id = elem_data.elem_id;

    /**
     * @param {any} event
     */
    function onPointerDown(event) {
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
    <p>{display_data.name}</p>
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
    }

    .element > p {
        margin: 0px;
        text-align: left;
    }

    .element > img {
        width: 96px;
        height: 96px;
        cursor: grab;
        image-rendering: pixelated;
        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
    }
</style>
