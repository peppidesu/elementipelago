import { writable } from "svelte/store";
import { Client } from "archipelago.js";

export const apclient = writable(new Client());

export const slotdata = writable(null);
