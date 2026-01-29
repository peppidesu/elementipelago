<script>
    import { get } from "svelte/store";
    import { apclient, slotdata } from "./stores/apclient.svelte";
    import { LoginError } from "archipelago.js";
    import { APWORLD_VERSION_REGEX, APWORLD_VERSIONS } from "../consts";

    /** @import { SlotData } from "./stores/apclient.svelte"; */
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

            /** @type SlotData */
            const response = await get(apclient).login(
                host,
                slot,
                "Elementipelago",
                password != "" ? { password: password } : {},
            );
            if (!APWORLD_VERSION_REGEX.test(response.version)) {
                throw new Error(
                    `AP world version '${response.version}' is not supported.\n Supported versions include: ${APWORLD_VERSIONS}`,
                );
            }

            slotdata.set(response);
            onSubmit();
        } catch (e) {
            if (e.name === "SecurityError" && !host.startsWith("ws://")) {
                error = "Failed to connect to Archipelago server.";
            } else {
                error = e.message;
            }
        } finally {
            loading = false;
        }
    }
</script>

<div class="login">
    <h1>elementipelago</h1>

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
        {loading ? "Connecting..." : "Connect to multiworld"}
    </button>

    <div class="error">{error}</div>
</div>
<div class="download">
    Download AP world here <a href="https://github.com/peppidesu/elementipelago/releases"
        ><img src="/sprites/ui/download.png" /></a
    >
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
    .download {
        position: absolute;
        top: 0;
        right: 0;
        margin: 10px;
        z-index: 10000;
        display: flex;
        height: fit-content;
        width: fit-content;
        align-items: center;
        gap: 5px;
        > a {
            padding: 0.25em;
            > img {
                display: block;
                width: 32px;
                height: 32px;
                image-rendering: pixelated;
            }
            margin-inline: auto;
            border-radius: 10px;
            border: 3px solid black;
            padding: 0.4em 0.4em;
            font-size: 1em;
            font-weight: 500;
            font-family: inherit;
            background-color: white;
            cursor: pointer;
            transition: border-color 0.25s;
            margin-left: 0.5em;

            &:hover {
                border-color: #646cff;
            }
        }
    }
    h1 {
        font-size: 3.2em;

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
