<script>
    import { get } from "svelte/store";
    import { apclient, slotdata } from "./stores/apclient.svelte";
    import { LoginError } from "archipelago.js";

    export let onSubmit;

    let host = localStorage.getItem("ap.host") ?? "archipelago.gg:38281";
    let slot = localStorage.getItem("ap.slot") ?? "";
    let password = localStorage.getItem("ap.password") ?? "";

    let loading = false;
    let error = "";

    async function submit() {
        error = "";
        try {
            loading = true;
            localStorage.setItem("ap.host", host);
            localStorage.setItem("ap.slot", slot);
            localStorage.setItem("ap.password", password);
            const response = await get(apclient).login(
                host,
                slot,
                "Elementipelago",
                password != "" ? { password: password } : {},
            );
            slotdata.set(response);
            onSubmit();
        } catch (e) {
            error = e.message;
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

    <button on:click={submit} disabled={loading || !host || !slot}>
        {loading ? "Connecting..." : "Connect"}
    </button>

    <div class="error">{error}</div>
</div>

<style>
    .login {
        margin: 48px auto;
        max-width: 700px;
        display: flex;
        flex-direction: column;
        justify-content: center;
        gap: 12px;
    }
    h1 {
        margin-top: 0;
    }
    label {
        display: grid;
        grid-template-columns: 10em auto;
        align-items: center;
        text-align: left;
    }
    @media (max-width: 700px) {
        .login {
            height: 100%;
        }
        label {
            display: flex;
            flex-direction: column;
        }
    }
    .error {
        color: #ff4b6a;
        height: 1lh;
    }

    button {
        width: 300px;
        margin-inline: auto;
        margin-block: 20px;
        &:disabled {
            opacity: 0.6;
            cursor: not-allowed;
        }
    }
</style>
