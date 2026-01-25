<script lang="js">
    import { onMount } from "svelte";
    import { dragging_elem } from "./stores/dragging";
    import { scale } from "svelte/transition";
    import { sfx } from "../audio";
    import { ElementKind } from "./graph.js";
    import { getElementData } from "./stores/apclient.svelte";
    import { element_to_name } from "../utils";

    let { x, y, elem_id, offsetx: ox, offsety: oy, attach, index } = $props();

    export function get_elem_id() {
        return elem_data.elem_id;
    }

    let icon;
    let selfWidth = $state(0);
    let iconWidth = $state(0);

    let lox = $derived((selfWidth - iconWidth) / 2);
    let sx = $derived(x - ox - lox);
    let sy = $derived(y - oy);
    let z = $state(10000);
    let being_dragged = $state(false);

    export function get_z_index() {
        return z;
    }

    export function set_z_index(index) {
        z = index;
    }

    export function get_rect() {
        return icon.getBoundingClientRect?.();
    }

    onMount(() => {
        if (attach) {
            dragging_elem.set({
                index: index,
                mfunc: (/** @type {number} */ lx, /** @type {number} */ ly) => {
                    x = lx;
                    y = ly;
                },
            });
        }
    });
    dragging_elem.subscribe((el) => {
        being_dragged = el != null && el.index === index;
    });

    /**
     * @param {{ layerX: any; layerY: any; x: any; y: any}} e
     */
    function onpointerdown(e) {
        sfx.drag_start();

        lox = 0;
        z = 10000;
        ox = e.layerX;
        oy = e.layerY;
        x = e.x;
        y = e.y;
        dragging_elem.set({
            index: index,
            mfunc: (/** @type {number} */ lx, /** @type {number} */ ly) => {
                x = lx;
                y = ly;
            },
        });
    }

    const elem_data_map = getElementData();
    const elem_name = $derived(element_to_name(elem_id));
    const elem_data = $derived(
        elem_data_map.get(elem_name) ?? {
            icon: "/sprites/elements/void.png",
            alt: "void",
            location: "Loading...",
            player: "Loading...",
            elem_id: elem_id,
            name: elem_name,
        },
    );
</script>

<div
    class="{being_dragged ? 'dragged' : ''} wrapper"
    style="left: {sx}px; top: {sy}px; z-index: {z};"
    transition:scale={{ duration: 100 }}
    bind:clientWidth={selfWidth}
>
    <img
        {onpointerdown}
        src={elem_data.icon}
        alt={elem_data.alt}
        draggable="false"
        class="
            {being_dragged ? 'dragged' : ''}
            {elem_data.elem_id.kind === ElementKind.OUTPUT ? 'compound' : ''}
        "
        bind:this={icon}
        bind:clientWidth={iconWidth}
    />
    <div>
        <h1>{elem_data.location}</h1>
        <p>
            {elem_data.elem_id.kind === ElementKind.OUTPUT ? "to" : "from"}
            {elem_data.player}
        </p>
        <p>{elem_data.name}</p>
    </div>
</div>

<style>
    .wrapper {
        position: absolute;
        justify-content: space-between;
        align-items: center;

        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
        pointer-events: none;

        padding: 0px;

        list-style-type: none;
        transition: transform 0.1s;
        &.dragged {
            transform: translateY(-5px);
        }
        > img {
            cursor: grab;
            width: 96px;
            height: 96px;
            image-rendering: pixelated;

            user-select: none;
            touch-action: none; /* IMPORTANT for mobile */
            pointer-events: all;

            background-color: white;
            border-width: 3px;
            border-radius: 10px;
            margin-bottom: 3px;
            border-style: solid;
            padding: 5px;
            box-shadow: 0 0px 0px 0px rgba(0, 0, 0, 0.4);
            transition: all 0.1s;

            &.dragged {
                box-shadow: 0 8px 12px 4px rgba(0, 0, 0, 0.35);
            }
            &.compound {
                border-color: #747bff;
                background-color: color-mix(in oklab, #747bff 20%, white 80%);
            }
        }
        > div {
            display: flex;
            flex-direction: column;
            align-items: center;
            > h1 {
                border-radius: 5px;
                margin: 5px;
                padding: 2px;
                line-height: 1.3;

                font-size: 1em;
                max-width: 500px;
                clip-path: rect(0 100% 1rlh 0);

                transition: all 0.1s;

                user-select: none;
                touch-action: none; /* IMPORTANT for mobile */
            }
            > p {
                display: inline;

                border-radius: 5px;
                margin: 0px;

                font-size: 0.75em;
                color: #484848;

                opacity: 0;
                transition: opacity 0.1s;

                user-select: none;
                touch-action: none; /* IMPORTANT for mobile */
            }
        }
        &:hover > div {
            > h1 {
                clip-path: rect(0 100% 100% 0);
            }
            > p {
                opacity: 1;
            }
        }
    }
</style>
