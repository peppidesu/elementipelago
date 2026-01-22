<script>
    // @ts-ignore
    import { get } from "svelte/store";
    import Element from "./Element.svelte";
    import {
        getDrawerElements,
        getElementData,
        isExhausted,
        isExplorable,
        upgrades,
    } from "./stores/apclient.svelte";
    import { dragging_elem } from "./stores/dragging";
    import Fuse from "fuse.js";

    let { mount_func, mounted_elements } = $props();
    let search_term = $state("");

    let show_discard = $state(false);

    let filtered_elements = $derived.by(() => {
        let el_data = getElementData();
        let table = Array.from(
            getDrawerElements()
                .values()
                .map((e) => el_data.get(e))
                .filter((e) => e.elem_id != null),
        ).sort((a, b) => {
            let res = 0;

            if (upgrades.progressive_filter > 1) {
                // @ts-ignore
                res = res || isExplorable(b.name) - isExplorable(a.name);
            }
            if (upgrades.progressive_filter > 0) {
                // @ts-ignore
                res = res || isExhausted(a.name) - isExhausted(b.name);
            }

            return (
                res ||
                a.elem_id.kind - b.elem_id.kind ||
                a.elem_id.id - b.elem_id.id
            );
        });

        if (search_term === "") return table;

        let fuse = new Fuse(table, {
            keys: [
                { name: "location", weight: 1 },
                { name: "player", weight: 0.5 },
                { name: "name", weight: 0.5 },
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
            <Element {elem_data} {mount_func} />
        {/each}
    </ul>
    <span
        class={show_discard
            ? "show-discard"
            : mounted_elements.size >= upgrades.field_size
              ? "show-blocking"
              : ""}
    >
    </span>
</div>

<style>
    #drawer-parent {
        display: grid;
        grid-template-columns: 1fr;
        grid-template-rows: 0fr auto;
    }
    @media (min-width: 1000px) {
        #drawer-parent {
            min-width: 500px;
            width: 40%;
        }
    }
    @media (max-width: 1000px) {
        #drawer-parent {
            height: 55%;
        }
    }
    input {
        margin: 10px;
        border-radius: 10px;
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
        border-radius: 10px;
        opacity: 0;
    }
    span.show-blocking {
        background-color: color-mix(in oklab, white 60%, #000000 40%);
        pointer-events: all;
        border: 3px solid #111111;
        display: inline;
        opacity: 0.9;
    }
    span.show-discard {
        display: inline;
        opacity: 0.9;
    }
    ul {
        grid-area: 2 / 1 / 2 / 1;
        display: flex;
        border: 3px solid black;
        border-radius: 10px;
        padding: 10px;
        margin: 10px;

        gap: 0px;
        flex-direction: column;
        flex-wrap: nowrap;
        overflow-y: scroll;
        scrollbar-color: #000 rgba(0, 0, 0, 0);
    }
</style>
