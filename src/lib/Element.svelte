<script lang="js">
    import { mount } from "svelte";
    import RealElement from "./RealElement.svelte";
    import { pointerLoc } from "./stores/pointer";
    import { get } from "svelte/store";

    const { elem } = $props();
    let el;
    export const recipe_elem = elem.recipe_elem;

    function onPointerDown(e) {
        let { x, y } = get(pointerLoc);

        const rect = el.getBoundingClientRect();

        mount(RealElement, {
            target: document.getElementById("playfield"),
            props: {
                x,
                y,
                elem,
                offsetx: x - rect.left,
                offsety: y - rect.top,
            },
        });
    }
</script>

<li class="element" onpointerdown={onPointerDown} bind:this={el}>
    <img src={elem.src} alt="" draggable="false" />
    <p>{elem.name}</p>
</li>

<style>
    .element {
        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
        cursor: grab;

        list-style-type: none;
        border-width: 3px;
        border-style: solid;
    }

    .element > p {
        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
    }

    .element > img {
        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
    }
</style>
