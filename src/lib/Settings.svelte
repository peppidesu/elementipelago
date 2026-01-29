<script>
    import { sfx } from "../audio";
    import Window from "./Window.svelte";

    let { show, onClose } = $props();
    let music_vol = $state(parseFloat(localStorage.getItem("settings.music_volume") ?? "1.0"));
    let sfx_vol = $state(parseFloat(localStorage.getItem("settings.sfx_volume") ?? "1.0"));

    let users = await (await fetch("/discord-users.json")).json();

    const credits = [
        {
            label: "Development & artwork",
            users: ["382561799742160896", "235482863250702336"],
        },
        {
            label: "Playtesting",
            users: ["1429874676590575907", "317342307705683968"],
        },
        {
            label: "Feedback & support",
            users: [
                "329656222213079053",
                "110878826136907776",
                "1296648831370268756",
                "86612976529838080",
            ],
            columns: 2,
        },
    ];

    function onchange() {
        localStorage.setItem("settings.music_volume", music_vol.toString());
    }
    function onpointerup() {
        localStorage.setItem("settings.sfx_volume", sfx_vol.toString());
        sfx.bubble();
    }
</script>

<Window {show} {onClose}>
    <div>
        <h1>Settings</h1>
        <ul>
            <h2>Audio</h2>
            <div>
                Music
                <input type="range" min="0" max="1" step="0.01" bind:value={music_vol} {onchange} />
            </div>
            <div>
                SFX
                <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={sfx_vol}
                    {onpointerup}
                />
            </div>
            <h2>Credits</h2>
            <p>Elementipelago</p>
            <p>a game made for Archipelago</p>
            {#each credits as section}
                <h3>{section.label}</h3>
                <ul
                    style="display: grid; grid-template-columns: repeat({section.columns ??
                        1}, 1fr); justify-self: stretch; align-items: center;"
                >
                    {#each section.users as id, i}
                        <li
                            style={section.users.length % 2 === 1 && i === section.users.length - 1
                                ? "grid-column: 1 / -1"
                                : ""}
                        >
                            {users[id].display ?? users[id].username} (@{users[id].username})
                        </li>
                    {/each}
                </ul>
            {/each}
            <div style="border-bottom: 3px solid #c0c0c0; padding-block: 10px"></div>
            <h2>License</h2>
            <p>
                Elementipelago is a free and open-source project licensed under the
                <a
                    href="https://github.com/peppidesu/elementipelago/blob/main/LICENSE"
                    target="_blank"
                >
                    AGPL-3.0 license
                </a>.
            </p>
            <p>
                All sprites are &copy; 2026 by Pepijn Bakker &amp; Noa Aarts and licensed under CC
                BY-NC 4.0. To view a copy of this license, visit
                <a href="https://creativecommons.org/licenses/by-nc/4.0/" target="_blank">
                    https://creativecommons.org/licenses/by-nc/4.0/
                </a>.
            </p>
            <p>
                SFX include samples sourced from freesound.com, some of which are licensed under CC
                BY 4.0. Attribution for these authors can be found in the README.md of this project.
            </p>
            <h2>Links</h2>
            <p style="display: flex; flex-wrap: wrap; justify-content: center; gap: 10px">
                <a
                    class="button"
                    target="_blank"
                    href="https://github.com/peppidesu/elementipelago"
                >
                    Github
                </a>
                <a class="button" target="_blank" href="https://archipelago.gg">Archipelago</a>
                <a
                    class="button"
                    target="_blank"
                    href="https://discord.com/channels/731205301247803413/1397584087493115934"
                    >Discord channel</a
                >
            </p>
        </ul>
    </div>
</Window>

<style>
    div {
        display: flex;
        flex-direction: column;
        align-items: stretch;
        width: 100%;
        gap: 10px;

        h1 {
            align-self: start;
            margin: 0;
            font-size: 1.5em;
            line-height: 1.2;
            text-align: left;
        }
        ul {
            padding-inline: 50px;
            overflow-y: scroll;
            flex-grow: 1;
            margin: 0;
            > * {
                max-width: 1000px;
                margin-inline: auto;
            }
            div {
                margin: 0 auto;
                display: flex;
                flex-direction: row;
                align-items: center;
                justify-content: space-between;
                text-anchor: middle;
                gap: 10px;
                input[type="range"] {
                    -webkit-appearance: none; /* Override default CSS styles */
                    appearance: none;
                    padding: 0px;
                    width: 50%;

                    border-width: 0px;
                    border-color: transparent;
                    height: 3px;

                    background: black;

                    &::-webkit-slider-thumb {
                        -webkit-appearance: none; /* Override default CSS styles */

                        width: 25px; /* Set a specific slider handle width */
                        height: 25px; /* Slider handle height */
                        border: 3px solid black;
                        border-radius: 999px;
                        background: white;
                        cursor: pointer; /* Cursor on hover */
                    }
                    &::-moz-range-thumb {
                        appearance: none;
                        width: 25px; /* Set a specific slider handle width */
                        height: 25px; /* Slider handle height */
                        border: 3px solid black;
                        border-radius: 999px;
                        background: white;
                        cursor: pointer; /* Cursor on hover */
                    }
                }
            }
            ul {
                list-style: none;
                justify-self: center;
                overflow: auto;
            }
        }
    }
</style>
