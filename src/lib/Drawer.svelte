<script>
    import { create_graph } from "../graph";
    import Element from "./Element.svelte";
    import { apclient, slotdata } from "./stores/apclient";

    const modules = import.meta.glob("../assets/Elements/*.png", {
        eager: true,
        import: "default",
    });

    let graph;
    let is_setup = false;
    let elements = [];

    slotdata.subscribe((sd) => {
        if (sd == null || is_setup) {
            return;
        }

        console.log("got new slotdata: ", sd);

        graph = create_graph(
            BigInt(sd.graph_seed),
            sd.element_amount,
            sd.compound_amount,
            sd.intermediate_amount,
            4,
            sd.compounds_are_ingredients,
        );
        console.log(graph);
    });

    apclient.subscribe((client) => {
        console.log("client changed");
        if (!client.authenticated || is_setup) {
            return;
        }

        const cim = client.items;
        elements = cim.received;
        cim.on("itemsReceived", (items, _startingIndex) => {
            elements.push(...items);
        });
        console.log(elements);
    });

    const el = Object.fromEntries(
        Object.entries(modules).map(([path, url]) => {
            const name = path.split("/").pop().replace(".png", "");
            return [name, url];
        }),
    );
</script>

<ul>
    <!-- {#each elements as elem} -->
    <!--     <Element {elem} /> -->
    <!-- {/each} -->
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
