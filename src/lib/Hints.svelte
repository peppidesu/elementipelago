<script>
    import { derived, get } from "svelte/store";
    import Element from "./Element.svelte";
    import { apclient, getElementData, hintedElements, hints } from "./stores/apclient.svelte";
    import { parse_element } from "../utils";
    import { ElementKind } from "./graph";
    import Window from "./Window.svelte";
    import { onMount } from "svelte";

    const { show, onClose } = $props();
</script>

<Window {show} {onClose}>
    <div class="hint-list">
        {#each hints.values() as hint_data}
            {@const i1_data = hint_data.ingredient_1}
            {@const i2_data = hint_data.ingredient_2}
            {@const prod_data = hint_data.product}
            <div>
                <div class="element">
                    <img src={i1_data.icon} alt={i1_data.alt} draggable="false" />
                    <span class="info">
                        <h1>{i1_data.location}</h1>
                        <p>from {i1_data.player}</p>
                        <p>{i1_data.name}</p>
                    </span>
                </div>
                <span>+</span>
                <div class="element">
                    <img src={i2_data.icon} alt={i2_data.alt} draggable="false" />
                    <span class="info">
                        <h1>{i2_data.location}</h1>
                        <p>from {i2_data.player}</p>
                        <p>{i2_data.name}</p>
                    </span>
                </div>
                <span>=</span>
                <div class="element">
                    <img src={prod_data.icon} alt={prod_data.alt} draggable="false" />
                    <span class="info">
                        <h1>{prod_data.location}</h1>
                        <p>from {prod_data.player}</p>
                        <p>{prod_data.name}</p>
                    </span>
                </div>
            </div>
        {/each}
    </div>
</Window>

<style>
    .hint-list {
        display: grid;
        gap: 1rem;
        width: 100%;
        align-content: center;

        > div {
            > span {
                font-size: 2rem;
            }
            grid-column: 2;
            display: flex;
            align-content: center;
            align-items: center;

            > div.element {
                max-width: 30%;
            }
        }
    }
</style>
