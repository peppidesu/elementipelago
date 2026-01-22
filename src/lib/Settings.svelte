<script>
    import { sfx } from "../audio";
    import Window from "./Window.svelte";

    let { show, onClose } = $props();
    let music_vol = $state(parseFloat(localStorage.getItem("settings.music_volume") ?? "1.0"));
    let sfx_vol = $state(parseFloat(localStorage.getItem("settings.sfx_volume") ?? "1.0"));

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
            <p>a game by Pepijn & Noa made for Archipelago</p>
            <!-- <h3>Programming</h3>
            <h3>Artwork</h3>
            <h3>Audio</h3>
            <h3>Support & Playtesting</h3> -->
            <h3>License</h3>
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
                SFX include samples sourced from freesound.com, some of which licensed under CC-BY
                4.0. Authors requiring attribution are listed below, detailed attribution can be
                found in the README.md of this project:
            </p>
            <ul>
                <li>maisonsonique</li>
                <li>InspectorJ</li>
            </ul>
            <h3>Special thanks</h3>
            <p>you &lt;3</p>
        </ul>
    </div>
</Window>

<style>
    div {
        display: flex;
        flex-direction: column;
        align-items: center;
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
                list-style: disc;
                justify-self: center;
                overflow: auto;
            }
        }
    }
</style>
