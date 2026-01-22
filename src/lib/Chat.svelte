<script>
    import { apclient } from "./stores/apclient.svelte";
    import { get } from "svelte/store";
    import Window from "./Window.svelte";

    let { show, onClose } = $props();

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
</script>

<Window {show} {onClose}>
    <div>
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
                if (event.key === "Enter") {
                    event.preventDefault();

                    get(apclient).messages.say(sendContent);
                    sendContent = "";
                }
            }}
        />
        <h1>Chat</h1>
    </div>
</Window>

<style>
    div {
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
        h1 {
            grid-area: a;
            margin: 0;
            font-size: 1.5em;
            line-height: 1.2;
            text-align: left;
        }
    }
</style>
