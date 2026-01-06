<script>
    import Element from "./Element.svelte";
    let elements = [
        { name: "waaa", src: "" },
        { name: "baba", src: "" },
        { name: "Noa", src: "" },
    ];
</script>

<ul>
    {#each elements as elem}
        <Element {elem} />
    {/each}
</ul>
