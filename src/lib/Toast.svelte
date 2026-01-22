<script>
    import { get } from "svelte/store";
    import { toast_queue } from "./stores/toast";
    import { fly, scale } from "svelte/transition";
    import { sfx } from "../audio";

    const transition_duration = 200;

    let current = $state(null);
    let cooldown = $state(false);

    let close_timeout = $state(0);

    toast_queue.subscribe(open);
    $effect(open);

    function onpointerup() {
        clearTimeout(close_timeout);
        close();
    }

    function open() {
        if (!cooldown && get(toast_queue).length > 0 && current == null) {
            toast_queue.update((q) => {
                current = q.splice(0, 1)[0];
                return q;
            });
            cooldown = true;
            sfx.toast();
            close_timeout = setTimeout(close, 5000);
            console.log(current);
        }
    }

    function close() {
        current = null;
        setTimeout(() => (cooldown = false), transition_duration);
    }
</script>

{#if current != null}
    <div {onpointerup} transition:fly={{ duration: transition_duration, x: "100%", y: 0 }}>
        <h1>{current.title}</h1>
        <p>{current.description}</p>
        <img src={current.image} />
    </div>
{/if}

<style>
    div {
        margin: 10px;
        padding: 10px;
        border: 3px solid black;
        border-radius: 10px;
        position: absolute;
        top: 0;
        right: 0;

        display: grid;
        grid-template-columns: auto 1fr;
        grid-template-rows: 0fr 0fr;
        align-items: center;
        justify-items: left;
        column-gap: 10px;
        width: 400px;
        row-gap: 0px;
        background-color: white;
        z-index: 10000;

        h1 {
            font-size: 1em;
            margin: 0px;
            margin-bottom: 3px;
            grid-area: 1 / 2 / 2 / 3;
            align-self: flex-end;
        }
        p {
            margin: 0px;
            font-size: 0.75em;
            align-self: flex-start;
            color: #484848;
            grid-area: 2 / 2 / 3 / 3;
        }
        img {
            grid-area: 1 / 1 / 3 / 2;
            image-rendering: pixelated;

            width: 96px;
            height: 96px;
        }
    }
</style>
