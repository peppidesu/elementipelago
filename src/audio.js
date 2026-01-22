const sfx_store = {
    drag_start: new Audio("/audio/drag-start.wav"),
    drag_end: new Audio("/audio/drag-end.wav"),
    trash: new Audio("/audio/trash.wav"),
    bubble: new Audio("/audio/bubble.wav"),
    toast: new Audio("/audio/received-item.wav"),
};

function get_sfx_volume() {
    return parseFloat(localStorage.getItem("settings.sfx_volume") ?? "1.0");
}

export const sfx = {
    drag_start: () => {
        /** @type HTMLAudioElement */ // @ts-ignore
        const sfx = sfx_store.drag_start.cloneNode();
        sfx.volume = 0.3 * get_sfx_volume();
        sfx.play();
    },
    drag_end: () => {
        /** @type HTMLAudioElement */ // @ts-ignore
        const sfx = sfx_store.drag_end.cloneNode();
        sfx.volume = 0.4 * get_sfx_volume();
        sfx.play();
    },
    trash: () => {
        /** @type HTMLAudioElement */ // @ts-ignore
        const sfx = sfx_store.trash.cloneNode();
        sfx.volume = 0.6 * get_sfx_volume();
        sfx.play();
    },
    bubble: () => {
        /** @type HTMLAudioElement */ // @ts-ignore
        const sfx = sfx_store.bubble.cloneNode();
        sfx.volume = 1 * get_sfx_volume();
        sfx.play();
    },
    toast: () => {
        /** @type HTMLAudioElement */ // @ts-ignore
        const sfx = sfx_store.toast.cloneNode();
        sfx.volume = 1 * get_sfx_volume();
        sfx.play();
    },
};
