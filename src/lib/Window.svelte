<script>
    import { fly } from "svelte/transition";
    import { apclient } from "./stores/apclient.svelte";
    import { get } from "svelte/store";
    import { linear } from "svelte/easing";

    let { show, onClose, children } = $props();

    function blurbg(_, params) {
        return {
            delay: params.delay || 0,
            duration: params.duration || 300,
            easing: params.easing || linear,
            css: (t, _) => `backdrop-filter: blur(${t * (params.blur || 5)}px)`,
        };
    }
</script>

{#if show}
    <div class="bg" transition:blurbg>
        <div class="window-wrapper" transition:fly={{ duration: 300, y: "-30%" }}>
            {@render children()}
            <button onclick={onClose}>X</button>
        </div>
    </div>
{/if}

<style>
    div.bg {
        backdrop-filter: blur(5px);
        display: grid;
        z-index: 10001;
        grid-template: 100% / 100%;
        div.window-wrapper {
            margin: 20px;
            padding: 10px;
            border: 3px solid black;
            border-radius: 10px;
            background-color: white;
            display: grid;
            grid-template: 100% / 100%;

            :global(> *) {
                grid-area: 1 / 1 / 1 / 1;
            }
            button {
                justify-self: end;
                z-index: 10001;
                width: 48px;
                height: 48px;
                padding: 0;
                text-anchor: middle;
                line-height: 1;
            }
        }
    }
</style>
