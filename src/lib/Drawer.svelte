<script>
    import { get } from "svelte/store";
    import { create_graph } from "../graph";
    import { name_to_kind } from "../utils";
    import Element from "./Element.svelte";
    import { apclient, graph, slotdata } from "./stores/apclient";
    import { iconForItem, iconForLocation } from "../iconml";
    import { element_to_elem } from "./stores/item_cache";

    import { element_urls } from "../consts";

    let received_elements = $state([]);
    let elements = $derived(
        received_elements
            .toSorted((a, b) => a.name.localeCompare(b.name))
            .reduce((acc, value) => {
                // de-dupe (array is already sorted)
                if (acc.length && acc[acc.length - 1].name === value.name) {
                    return acc;
                }

                // parse "Element 129" | "Intermediate 29"

                let kind = name_to_kind(value.name);
                if (kind == null) return acc;

                const ete = get(element_to_elem);
                if (ete.has(value.name)) {
                    acc.push(ete.get(value.name));
                } else {
                    const val = value.name;
                    const iname = iconForItem(value);
                    const ico = element_urls[iname] ?? element_urls["void"];
                    let elem = {
                        name: val,
                        src: ico,
                        recipe_elem: kind,
                        init: true,
                    };
                    ete.set(val, elem);
                    acc.push(elem);
                }

                return acc;
            }, []),
    );

    slotdata.subscribe((sd) => {
        console.log(sd);
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
</script>

<div>
    <ul id="drawer">
        {#each elements as elem}
            <Element {elem} />
        {/each}
    </ul>
</div>

<style>
    div {
        height: 100%;
        display: flex;
    }
    ul {
        display: flex;
        border: 3px solid black;
        border-radius: 5px;
        padding: 10px;
        margin: 10px;
        width: 300px;

        gap: 10px;
        flex-direction: column;
        flex-wrap: nowrap;
        overflow-y: scroll;
    }
</style>
