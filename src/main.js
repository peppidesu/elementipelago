import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { apclient, slotdata } from "./lib/stores/apclient";
import { Client } from "archipelago.js";
import { get } from "svelte/store";
import { model } from "./lib/stores/model";
import { loadIconModel } from "./lib/machine-learning/iconml";



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
