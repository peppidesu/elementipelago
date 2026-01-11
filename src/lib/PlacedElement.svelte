<script lang="js">
    import { onDestroy, onMount } from "svelte";
    import { dragging_elem } from "./stores/dragging";
    import { get } from "svelte/store";
    import { pointerLoc } from "./stores/pointer";

    const { x, y, elem, offsetx: localx, offsety: localy, attach } = $props();

    let self, icon;

    let ox = localx;
    let oy = localy;
    let sx = $state(x - ox);
    let sy = $state(y - oy);

    onMount(() => {
        const srect = self.getBoundingClientRect();
        const irect = icon.getBoundingClientRect();
        ox += irect.left - srect.left;
        sx -= irect.left - srect.left;
        self.recipe_elem = elem.recipe_elem;
        if (attach) {
            dragging_elem.set({
                self: self,
                mfunc: (lx, ly) => {
                    sx = lx - ox;
                    sy = ly - oy;
                },
            });
        }
    });

    function onpointerdown(e) {
        ox = e.layerX;
        oy = e.layerY;
        dragging_elem.set({
            self: self,
            mfunc: (lx, ly) => {
                sx = lx - ox;
                sy = ly - oy;
            },
        });
    }
</script>

<div {onpointerdown} style="left: {sx}px; top: {sy}px;" bind:this={self}>
    <img src={elem.src} alt="" draggable="false" bind:this={icon} />
    <p>{elem.name}</p>
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
        }

        p {
            margin: 0px;

            user-select: none;
            touch-action: none; /* IMPORTANT for mobile */
        }
    }
</style>
