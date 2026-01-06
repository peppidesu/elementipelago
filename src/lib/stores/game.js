import { writable } from "svelte/store";

export const elements = writable([]);
export const discovered = writable(new Set());
export const placed = writable([]);
