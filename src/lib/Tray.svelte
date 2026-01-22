<script>
    let { handler } = $props();

    const buttons = [
        { id: "settings", inner: "settings" },
        { id: "chat", inner: "chat" },
        { id: "hints", inner: "hints" },
    ];
</script>

<div>
    {#each buttons as btn}
        <button onclick={() => handler(btn.id)}>{btn.inner}</button>
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
        }
    }
</style>
