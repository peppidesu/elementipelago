<script>
    import { derived } from "svelte/store";
    import Element from "./Element.svelte";
    import { getElementData, hintedElements } from "./stores/apclient.svelte";

    let edata = getElementData();
    let hints = $derived(
        hintedElements.entries().map(([_, hint]) => {
            return {
                ingredient_1: edata.get(hint.ingredient_1),
                ingredient_2: edata.get(hint.ingredient_2),
                product: edata.get(hint.result),
            };
        }),
    );

    function mountFunc() {}
</script>

<div>
    {#each hints as hint_data}
        <Element elem_data={hint_data.ingredient_1} mount_func={mountFunc} />
        <Element elem_data={hint_data.ingredient_2} mount_func={mountFunc} />
        <Element elem_data={hint_data.product} mount_func={mountFunc} />
    {/each}
</div>
