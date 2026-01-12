import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { model } from "./lib/stores/model.js";
import { loadIconModel } from "./iconml";
import { element_urls } from "./consts";

const app = mount(App, {
    target: document.getElementById("app"),
});

model.set(
    await loadIconModel(
        "/models/icon_model.meta.json",
        "/models/icon_model.weights.bin",
    ),
);

export default app;
