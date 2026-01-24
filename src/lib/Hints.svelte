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
    <div class="holder">
        <h1>Hinted Items</h1>
        <div class="hint-list">
            {#each hints.values() as hint_data}
                {@const i1_data = hint_data.ingredient_1}
                {@const i2_data = hint_data.ingredient_2}
                {@const prod_data = hint_data.product}
                <div>
                    <img src={i1_data.icon} alt={i1_data.alt} draggable="false" />
                    <span class="info">
                        <h1>{i1_data.location}</h1>
                        <p>from {i1_data.player}</p>
                        <p>{i1_data.name}</p>
                    </span>
                </div>
                <span>+</span>
                <div>
                    <img src={i2_data.icon} alt={i2_data.alt} draggable="false" />
                    <span class="info">
                        <h1>{i2_data.location}</h1>
                        <p>from {i2_data.player}</p>
                        <p>{i2_data.name}</p>
                    </span>
                </div>
                <span>=</span>
                <div>
                    <img src={prod_data.icon} alt={prod_data.alt} draggable="false" />
                    <span class="info">
                        <h1>{prod_data.location}</h1>
                        <p>from {prod_data.player}</p>
                        <p>{prod_data.name}</p>
                    </span>
                </div>
            {/each}
        </div>
    </div>
</Window>

<style>
    .holder {
        display: grid;
        grid-template:
            "a" auto
            "b" 1fr / 1fr;

        justify-content: start;

        > h1 {
            height: fit-content;
            margin: 0;
            padding: 10px;
            grid-area: a;
            z-index: 10001;
            background: linear-gradient(white, 90%, transparent);
        }

        > div {
            grid-area: a / b;
        }
    }

    .hint-list {
        display: grid;
        gap: 1rem;
        width: 100%;

        overflow-y: scroll;
        align-content: center;
        align-items: center;
        grid-template-columns: 1fr auto 1fr auto 1fr;

        > span {
            font-size: 2rem;
        }

        > div {
            display: grid;
            grid-template-columns: auto 1fr;
            grid-auto-flow: column;
            align-items: center;
            image-rendering: pixelated;
            gap: 15px;

            list-style-type: none;
            margin-inline: 10px;
            padding-block: 10px;

            > span.info {
                min-width: 0;
                flex-shrink: 1;

                > h1 {
                    font-weight: bold;
                    margin: 0px;

                    margin-bottom: 2px;
                    padding-bottom: 3px;

                    text-align: left;
                    font-size: 1em;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    overflow: hidden;
                }

                > p {
                    color: #484848;
                    margin: 0px;
                    text-align: left;
                    font-size: 0.75em;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    overflow: hidden;
                }
            }

            > img {
                width: 96px;
                height: 96px;

                user-select: none;
                touch-action: none;
                /* IMPORTANT for mobile */
            }
        }
    }
</style>
