<script lang="js">
    import { mount } from "svelte";
    import PlacedElement from "./PlacedElement.svelte";
    import { pointerLoc } from "./stores/pointer";
    import { get } from "svelte/store";

    const { elem_data } = $props();
    let el;

    export const elem_id = elem_data.elem_id;

    /**
     * @param {any} _
     */
    function onPointerDown(_) {
        let { x, y } = get(pointerLoc);

        const rect = el.getBoundingClientRect();

        mount(PlacedElement, {
            target: document.getElementById("playfield"),
            props: {
                x,
                y,
                elem_data,
                offsetx: x - rect.left,
                offsety: y - rect.top,
                attach: true,
            },
        });
    }
</script>

<li class="element" onpointerdown={onPointerDown} bind:this={el}>
    <img src="/sprites/elements/apple.png" alt="" draggable="false" />
    <p>{elem_data.name}</p>
</li>

<style>
    .element {
        display: flex;
        align-items: center;
        gap: 15px;

        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
        cursor: grab;

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
        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
    }

    .element > img {
        width: 96px;
        height: 96px;
        image-rendering: pixelated;
        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
    }
</style>
