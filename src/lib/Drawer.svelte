<script>
    import { get } from "svelte/store";
    import Element from "./Element.svelte";
    import { drawerElements, elementData } from "./stores/apclient";
    import { dragging_elem } from "./stores/dragging";
    import Fuse from "fuse.js";

    let { mount_func } = $props();
    let search_term = $state("");


    let show_discard = $state(false);

    let filtered_elements = $derived.by(() => {
        let table = Array.from(get(drawerElements)
          .values()
          .map((e) => get(elementData).get(e))
          .filter((e) => e.elem_id != null)
        ).sort((a, b) => a.name.localeCompare(b.name));

        if (search_term === "") return table;

        let fuse = new Fuse(table, {
            keys: [
                { name: "display.location", weight: 0.5 },
                { name: "display.name", weight: 1 },
                { name: "display.player", weight: 0.5 },
            ],
            threshold: 0.3,
        });
        let res = fuse
            .search(search_term)
            .sort((a, b) => b.score - a.score)
            .map((r) => r.item);
        return res;
    });

    dragging_elem.subscribe((el) => {
        show_discard = el !== null;
    });


</script>

<div id="drawer-parent">
    <input bind:value={search_term} />
    <ul id="drawer">
        {#each filtered_elements as elem_data}
            <Element
                {elem_data}
                {mount_func}
            />
        {/each}
    </ul>
    <span class={show_discard ? "show-discard" : ""}> </span>
</div>

<style>
    #drawer-parent {
        display: grid;
        grid-template-columns: 1fr;
        grid-template-rows: 0fr auto;
    }
    @media (min-width: 800px) {
        #drawer-parent {
            min-width: 400px;
            width: 35%;
        }
    }
    @media (max-width: 800px) {
        #drawer-parent {
            height: 50%;
        }
    }
    input {
        margin: 10px;
        border-radius: 5px;
        border-width: 3px;
        padding: 10px;
    }
    span {
        transition: all 0.1s;
        pointer-events: none;
        grid-area: 2 / 1 / 2 / 1;
        background-color: color-mix(in oklab, white 80%, #ff4b6a 20%);
        border: 3px solid #ff4b6a;
        margin: 10px;
        border-radius: 5px;
        opacity: 0;
    }
    span.show-discard {
        display: inline;
        opacity: 0.9;
    }
    ul {
        grid-area: 2 / 1 / 2 / 1;
        display: flex;
        border: 3px solid black;
        border-radius: 5px;
        padding: 10px;
        margin: 10px;

        gap: 10px;
        flex-direction: column;
        flex-wrap: nowrap;
        overflow-y: scroll;
    }
</style>
