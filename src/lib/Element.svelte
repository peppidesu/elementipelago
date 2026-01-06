<script lang="js">
    import { mount } from "svelte";
    import RealElement from "./RealElement.svelte";

    const { elem } = $props();

    function onPointerDown(e) {
        mount(RealElement, {
            target: document.querySelector("#app"),
            props: {},
        });
    }
</script>

<li class="element" onpointerdown={onPointerDown}>
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
