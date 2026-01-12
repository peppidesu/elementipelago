<script lang="javascript">
    import { onMount } from "svelte";

    /**
     * @param {Map<number, import("./PlacedElement.svelte").default>} mounted
     */
    export function handle_dropped(mounted) {
        Array.from(mounted)
            .toSorted(
                ([_a, a], [_b, b]) =>
                    Number(a.get_z_index()) - Number(b.get_z_index()),
            )
            .forEach(([_num, ele], idx) => ele.set_z_index(String(idx + 1)));
    }
</script>

<div id="playfield"></div>
