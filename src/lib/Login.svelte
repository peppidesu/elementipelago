<script>
    import { get } from "svelte/store";
    import { apclient } from "./stores/apclient";

    export let onSubmit;

    let host = localStorage.getItem("ap.host") ?? "archipelago.gg:38281";
    let slot = localStorage.getItem("ap.slot") ?? "";
    let password = localStorage.getItem("ap.password") ?? "";

    let loading = false;
    let error = "";

    async function submit() {
        error = "";
        loading = true;
        try {
            localStorage.setItem("ap.host", host);
            localStorage.setItem("ap.slot", slot);
            localStorage.setItem("ap.password", password);

            await get(apclient).login(
                host,
                slot,
                "Elementipelago",
                password != "" ? { password: password } : {},
            );
            onSubmit();
        } catch (e) {
            error = e?.message ?? String(e);
        } finally {
            loading = false;
        }
    }
</script>

<div class="login">
    <h1>Connect to Archipelago</h1>

    <label>
        Host
        <input bind:value={host} placeholder="archipelago.gg:38281" />
    </label>

    <label>
        Slot / Player name
        <input bind:value={slot} placeholder="MySlotName" />
    </label>

    <label>
        Password (optional)
        <input type="password" bind:value={password} />
    </label>

    {#if error}
        <div class="error">{error}</div>
    {/if}

    <button on:click={submit} disabled={loading || !host || !slot}>
        {loading ? "Connecting..." : "Connect"}
    </button>
</div>

<style>
    .login {
        max-width: 420px;
        margin: 48px auto;
        display: grid;
        gap: 12px;
    }
    label {
        display: grid;
        gap: 6px;
    }
    input {
        padding: 10px;
        border: 1px solid #ccc;
        border-radius: 8px;
    }
    .error {
        color: #b00020;
    }
    button {
        padding: 10px 14px;
        border-radius: 10px;
        border: 0;
        cursor: pointer;
    }
    button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>
