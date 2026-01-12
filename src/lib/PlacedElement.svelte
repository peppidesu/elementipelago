<script lang="js">
    import { onMount } from "svelte";
    import { dragging_elem } from "./stores/dragging";
    import { get } from "svelte/store";
    import { apclient } from "./stores/apclient";
    import { elem_to_location_id } from "../utils";
    import { iconForLocation } from "../iconml";
    import { element_to_elem } from "./stores/item_cache";
    import { element_urls } from "../consts";

    let { x, y, elem, offsetx: localx, offsety: localy, attach } = $props();

    let self, icon;

    let ox = localx;
    let oy = localy;
    let sx = $state(x - ox);
    let sy = $state(y - oy);
    let z = $state(10000);

    let display_ele = $state(Promise.resolve(elem));
    if (elem.init) {
        display_ele = Promise.resolve(elem);
    } else {
        get(apclient)
            .scout([elem_to_location_id(elem)], 0)
            .then((items) => {
                const fst = items[0];
                const disp_name = fst.name;
                const iname = iconForLocation(elem.name);
                const ico = element_urls[iname] ?? element_urls["void"];
                console.log(ico);
                let ele2 = {
                    name: disp_name,
                    src: ico,
                    recipe_elem: elem.recipe_elem,
                    init: true,
                };
                get(element_to_elem).set(elem.name, ele2);
                return ele2;
            });
    }

    onMount(async () => {
        const srect = self.getBoundingClientRect();
        // const irect = icon.getBoundingClientRect();
        ox += srect.left - srect.left;
        sx -= srect.left - srect.left;
        self.recipe_elem = elem.recipe_elem;
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
    bind:this={self}
>
    {#await display_ele}
        <img src={elem.ico} alt="" draggable="false" />
        <p>{elem.name}</p>
    {:then value}
        <img src={value.ico} alt="" draggable="false" />
        <p>{value.name}</p>
    {/await}
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
