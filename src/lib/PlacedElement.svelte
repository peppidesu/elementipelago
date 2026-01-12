<script lang="js">
    import { onMount } from "svelte";
    import { dragging_elem } from "./stores/dragging";
    import { scale } from "svelte/transition";

    const {
        x,
        y,
        elem_data,
        offsetx: localx,
        offsety: localy,
        attach,
    } = $props();

    let self, icon;

    let ox = localx;
    let oy = localy;
    let sx = $state(x - ox);
    let sy = $state(y - oy);
    let z = $state(10000);
    let being_dragged = $state(false);

    onMount(() => {
        const srect = self.getBoundingClientRect();
        const irect = icon.getBoundingClientRect();
        ox += irect.left - srect.left;
        sx -= irect.left - srect.left;
        self.elem_id = elem_data.elem_id;
        self.set_z_idx = (/** @type {number} */ val) => {
            z = val;
        };
        if (attach) {
            dragging_elem.set({
                self: self,
                mfunc: (/** @type {number} */ lx, /** @type {number} */ ly) => {
                    sx = lx - ox;
                    sy = ly - oy;
                },
            });
        }
    });
    dragging_elem.subscribe((el) => {
        being_dragged = el != null && el.self === self;
    });

    /**
     * @param {{ layerX: any; layerY: any; }} e
     */
    function onpointerdown(e) {
        z = 10000;
        ox = e.layerX;
        oy = e.layerY;
        dragging_elem.set({
            self: self,
            mfunc: (/** @type {number} */ lx, /** @type {number} */ ly) => {
                sx = lx - ox;
                sy = ly - oy;
            },
        });
    }
</script>

<div
    {onpointerdown}
    style="left: {sx}px; top: {sy}px; z-index: {z};"
    transition:scale
    bind:this={self}
>
    <img
        src="/sprites/elements/apple.png"
        alt=""
        draggable="false"
        class={being_dragged ? "dragged" : ""}
        bind:this={icon}
    />
    <p>{elem_data.name}</p>
</div>

<style>
    div {
        position: absolute;
        justify-content: space-between;
        align-items: center;

        user-select: none;
        touch-action: none; /* IMPORTANT for mobile */
        cursor: grab;
        padding: 0px;

        list-style-type: none;

        img {
            width: 96px;
            height: 96px;
            image-rendering: pixelated;
            user-select: none;
            touch-action: none; /* IMPORTANT for mobile */
            background-color: white;
            border-width: 3px;
            border-radius: 10px;
            border-style: solid;
            padding: 5px;
            box-shadow: 0 0px 0px 0px rgba(0, 0, 0, 0.4);
            transition: all 0.1s;

            &.dragged {
                transform: translateY(-5px);
                box-shadow: 0 8px 12px 4px rgba(0, 0, 0, 0.35);
            }
        }

        p {
            margin: 0px;

            user-select: none;
            touch-action: none; /* IMPORTANT for mobile */
        }
    }
</style>
