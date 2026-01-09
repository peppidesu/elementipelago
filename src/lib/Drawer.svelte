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
    let is_setup = false;
    let received_elements = $state([]);
    let elements = $derived(
        received_elements
            .toSorted((a, b) => {
                return a.name.localeCompare(b.name);
            })
            .filter((val, idx, arr) => {
                return idx == 0 || val.name != arr[idx - 1].name;
            })
            .map((value, idx, _) => {
                return { name: value.name, src: el.apple };
            }),
    );

    slotdata.subscribe((sd) => {
        if (sd == null || is_setup) {
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
        if (!client.authenticated || is_setup) {
            return;
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
