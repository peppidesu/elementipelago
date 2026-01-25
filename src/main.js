import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { upgrades } from "./lib/stores/apclient.svelte.js";

const app = mount(App, {
    target: document.getElementById("app"),
});

export default app;
