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
        { name: "waaa", src: el.apple },
        { name: "baba", src: el.book },
        { name: "Noa", src: el.coin },
    ];
</script>

<ul>
    {#each elements as elem}
        <Element {elem} />
    {/each}
</ul>
