<script lang="javascript">
    import Drawer from "./lib/components/Drawer.svelte";
    import { pointerLoc } from "./lib/state/pointer";
    import Login from "./lib/components/Login.svelte";
    import Playfield from "./lib/components/Playfield.svelte";
    import Toast from "./lib/components/Toast.svelte";
    import Chat from "./lib/components/Chat.svelte";
    import Tray from "./lib/components/Tray.svelte";
    import Settings from "./lib/components/Settings.svelte";
    import Hints from "./lib/components/Hints.svelte";
    import { moveDragging } from "./lib/state/playfield.svelte";

    let openWindow = $state("");
    /** @type {Playfield} */
    let playfield = $state(undefined);

    /**
     * @param {{ clientX: any; clientY: any; }} event
     */
    function onpointermove(event) {
        pointerLoc.set({ x: event.clientX, y: event.clientY });
        moveDragging(event.clientX, event.clientY);
    }
    /**
     * @param {any} event
     */
    function onpointerup(event) {
        playfield?.ondrop(event);
    }

    let connected = $state(false);
    async function handleLogin() {
        connected = true;
    }
</script>

<svelte:window {onpointermove} {onpointerup} />

{#if !connected}
    <Login onSubmit={handleLogin} />
{:else}
    <div class="game">
        <Drawer />
        <Playfield bind:this={playfield} />
    </div>
{/if}
<Tray
    handler={(/** @type {string} */ btn) => {
        openWindow = btn;
    }}
    {connected}
/>

<Toast />
<Chat show={openWindow == "chat"} onClose={() => (openWindow = "")} />
<Hints show={openWindow == "hints"} onClose={() => (openWindow = "")} />
<Settings show={openWindow == "settings"} onClose={() => (openWindow = "")} />

<style>
    .game {
        display: flex;
        @media (max-width: 1000px) {
            flex-direction: column-reverse;
        }
    }
</style>
