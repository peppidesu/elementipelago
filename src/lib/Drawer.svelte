<script>
    import { get } from "svelte/store";
    import { create_graph } from "../graph";
    import Element from "./Element.svelte";
    import { apclient, slotdata } from "./stores/apclient";
    import { DeepSet } from "deep-equality-data-structures";

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

    let graph;
    let received_elements = $state([]);
    let elements = $derived(
        received_elements
            // sort on names, might want to add something about the slot once we use location names
            .toSorted((a, b) => {
                return a.name.localeCompare(b.name);
            })
            // remove duplicates, they shouldn't show up multiple times
            .filter((val, idx, arr) => {
                return idx == 0 || val.name != arr[idx - 1].name;
            })
            // map from the Item to the format we need in game
            .map((value) => {
                return { name: value.name, src: el.apple };
            }),
    );

    slotdata.subscribe((sd) => {
        if (sd == null) {
            return;
        }

        graph = create_graph(
            BigInt(sd.graph_seed),
            sd.element_amount,
            sd.compound_amount,
            sd.intermediate_amount,
            4,
            sd.compounds_are_ingredients,
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

<ul>
    {#each elements as elem}
        <Element {elem} />
    {/each}
</ul>

<style>
    ul {
        display: flex;
        width: 200px;
        gap: 10px;
        flex-direction: row;
        flex-wrap: wrap;
    }
</style>
