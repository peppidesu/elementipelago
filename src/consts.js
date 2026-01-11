export const LOCATION_AMOUNT = 2000;

const modules = import.meta.glob("./assets/Elements/*.png", {
    eager: true,
    import: "default",
});

export const element_urls = Object.fromEntries(
    Object.entries(modules).map(([path, url]) => {
        const name = path.split("/").pop().replace(".png", "");
        return [name, url];
    }),
);
