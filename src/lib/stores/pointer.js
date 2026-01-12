import { writable } from "svelte/store";

export const pointerLoc = writable({ x: 0, y: 0 });
