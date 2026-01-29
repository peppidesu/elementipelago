<script>
    let { handler, connected } = $props();

    const buttons = [
        { id: "settings", icon: "settings" },
        { id: "chat", icon: "chat" },
        { id: "hints", icon: "hint2" },
    ];
</script>

<div id="tray">
    {#each connected ? buttons : [buttons[0]] as btn}
        <button onclick={() => handler(btn.id)}><img src="/sprites/ui/{btn.icon}.png" /></button>
    {/each}
</div>

<style>
    div {
        position: absolute;
        top: 0;
        right: 0;
        margin: 10px;
        z-index: 10000;
        display: flex;
        height: fit-content;
        width: fit-content;
        gap: 5px;
        > button {
            padding: 0.25em;
            > img {
                display: block;
                width: 32px;
                height: 32px;
                image-rendering: pixelated;
            }
        }
    }
</style>
