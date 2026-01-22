<script>
    import { fly } from "svelte/transition";
    import { apclient } from "./stores/apclient.svelte";
    import { get } from "svelte/store";
    import { linear } from "svelte/easing";

    let { show, onClose } = $props();

    /**
     * @type Element
     */
    let chat = $state(undefined);
    let msgs = $state([]);
    let sendContent = $state("");

    get(apclient).messages.on("message", (msg, _) => {
        msgs.push(msg);
    });

    const attachMsg = (el) => {
        el.scrollIntoView({
            behavior: "smooth",
            block: "end",
        });
    };

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
        <div class="root" transition:fly={{ duration: 300, y: "-30%" }}>
            <ul>
                {#each msgs as msg}
                    <li {@attach attachMsg}>
                        <span>{">>"}</span>
                        <pre>{msg}</pre>
                    </li>
                {/each}
            </ul>
            <input
                bind:value={sendContent}
                onkeydown={(event) => {
                    if (event.key === "Enter" || event.keyCode === 13) {
                        event.preventDefault();

                        get(apclient).messages.say(sendContent);
                        sendContent = "";
                    }
                }}
            />
            <button onclick={onClose}>X</button>
            <h1>Chat</h1>
        </div>
    </div>
{/if}

<style>
    div.bg {
        backdrop-filter: blur(5px);
        display: grid;
        z-index: 10000;
        grid-template: 100% / 100%;
        div.root {
            margin: 20px;
            padding: 10px;
            border: 3px solid black;
            border-radius: 10px;
            background-color: white;
            display: grid;
            align-items: center;
            grid-template:
                "a b" 0fr
                "c c" 1fr
                "d d" 0fr / 1fr 0fr;

            ul {
                grid-area: c;
                padding-inline: 10px;
                margin-block: 10px;
                list-style: none;
                overflow-y: scroll;
                align-self: stretch;
                display: flex;
                flex-direction: column;
                gap: 0.1lh;

                li {
                    text-align: left;
                    display: flex;
                    gap: 0.6em;
                    padding: 0;

                    pre {
                        white-space: pre-wrap;
                        margin: 0;
                        font-family: inherit;
                    }
                }
            }
            input {
                grid-area: d;
            }
            button {
                grid-area: b;
                aspect-ratio: 1 / 1;
                height: 100%;
                padding: 0;
                text-anchor: middle;
                line-height: 1;
            }
            h1 {
                grid-area: a;
                margin: 0;
                font-size: 1.5em;
                line-height: 1.2;
                text-align: left;
            }
        }
    }
</style>
