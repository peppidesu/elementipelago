import { writable } from "svelte/store";


// item/location name (with game and kind) -> {src: path to icon, name: name to display}
export const icon_cache = writable(new Map())

