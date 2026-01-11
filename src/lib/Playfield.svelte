<script lang="javascript">
    import { onMount } from "svelte";

    let el;
    function handle_dropped() {
        if (el == undefined) return;

        Array.from(el.children)
            .toSorted((a, b) => Number(a.style.zIndex) - Number(b.style.zIndex))
            .forEach((ele, idx) => ele.set_z_idx(String(idx + 1)));
    }

    onMount(() => {
        el = document.getElementById("playfield");
        el.handle_dropped = handle_dropped;
    });
</script>

<div id="playfield"></div>
