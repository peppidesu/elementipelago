import { DeepMap } from "deep-equality-data-structures";
import { writable } from "svelte/store";

export const element_to_icon = writable(new DeepMap());
