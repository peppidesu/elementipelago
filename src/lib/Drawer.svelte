<script>
    import Element from "./Element.svelte";

    const modules = import.meta.glob("../assets/Elements/*.png", {
        eager: true,
        import: "default",
    });

    const el = Object.fromEntries(
        Object.entries(modules).map(([path, url]) => {
            const name = path.split("/").pop().replace(".png", "");
            return [name, url];
        }),
    );

    console.log(el);

    let elements = [
        { name: "apple", src: el.apple },
        { name: "book", src: el.book },
        { name: "coin", src: el.coin },
    ];
</script>

<ul>
    {#each elements as elem}
        <Element {elem} />
    {/each}
</ul>

<style>
    ul {
        display: flex;
        width: 200px;
        gap: 10px;
        flex-direction: row;
        flex-wrap: wrap;
    }
</style>
