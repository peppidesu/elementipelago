<script>
    import { get } from "svelte/store";
    import { create_graph } from "../graph";
    import { element_to_name, parse_element } from "../utils";
    import Element from "./Element.svelte";
    import { apclient, graph, slotdata } from "./stores/apclient";
    import { dragging_elem } from "./stores/dragging";
    import { icon_cache } from "./stores/icon_cache";

    let { mount_func } = $props();

    let received_elements = $state([]);
    let show_discard = $state(false);
    let elements = $derived(
        received_elements
            .toSorted((a, b) => a.name.localeCompare(b.name))
            .reduce((acc, value) => {
                // de-dupe (array is already sorted)
                if (acc.length && acc[acc.length - 1].name === value.name) {
                    return acc;
                }

                // parse "Element 129" | "Intermediate 29"

                let elem_id = parse_element(value.name);
                if (elem_id == null) return acc;

                acc.push({
                    name: value.name,
                    elem_id: elem_id,
                });

                return acc;
            }, []),
    );
    dragging_elem.subscribe((el) => {
        show_discard = el !== null;
    });

    slotdata.subscribe((sd) => {
        if (sd == null) {
            return;
        }

        graph.set(
            create_graph(
                BigInt(sd.graph_seed),
                sd.element_amount,
                sd.compound_amount,
                sd.intermediate_amount,
                4,
                sd.compounds_are_ingredients,
            ),
        );
        let client = get(apclient);
        if (!client.authenticated) {
            throw "Slotdata was received without a connected client.";
        }

        const cim = client.items;
        received_elements = cim.received;
        cim.on("itemsReceived", (items, _startingIndex) => {
            received_elements.push(...items);
        });
    });

    let dd = $state(undefined);
    icon_cache.subscribe((val) => {
        dd = val;
    });

    let display_data = $state((elem_data) => {
        if (dd != undefined) {
            return dd.get(elem_data.name);
        }
        return { icon: "void", name: elem_data.name };
    });
</script>

<div>
    <ul id="drawer">
        {#each elements as elem_data}
            <Element
                {elem_data}
                {mount_func}
                display_data={display_data(elem_data)}
            />
        {/each}
    </ul>
    <span class={show_discard ? "show-discard" : ""}> </span>
</div>

<style>
    div {
        display: grid;
        grid-template-columns: 1fr;
        grid-template-rows: 1fr;
    }
    @media (min-width: 600px) {
        div {
            width: 350px;
        }
    }
    @media (max-width: 600px) {
        div {
            height: 50%;
        }
    }
    span {
        transition: all 0.1s;
        pointer-events: none;
        grid-area: 1 / 1 / 1 / 1;
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
        grid-area: 1 / 1 / 1 / 1;
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
