<script>
    import { get } from "svelte/store";
    import { create_graph } from "../graph";
    import { name_to_kind } from "../utils";
    import Element from "./Element.svelte";
    import { apclient, graph, slotdata } from "./stores/apclient";

    const modules = import.meta.glob("../assets/Elements/*.png", {
        eager: true,
        import: "default",
    });

    const el = Object.fromEntries(
        Object.entries(modules).map(([path, url]) => {
            const name = path.split("/").pop().replace(".png", "");
            return [name, url];
        }),
    );

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

                acc.push({
                    name: value.name,
                    src: el.apple,
                    recipe_elem: kind,
                });

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
