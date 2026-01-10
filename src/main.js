import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { apclient, slotdata } from "./lib/stores/apclient";
import { Client } from "archipelago.js";
import { get } from "svelte/store";

const app = mount(App, {
  target: document.getElementById("app"),
});

export default app;
