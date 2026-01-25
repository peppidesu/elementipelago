const sfx_store = {
    drag_start: new Audio("/audio/drag-start.wav"),
    drag_end: new Audio("/audio/drag-end.wav"),
    trash: new Audio("/audio/trash.wav"),
    bubble: new Audio("/audio/bubble.wav"),
    toast: new Audio("/audio/received-item.wav"),
};

function get_sfx_volume() {
    let raw = parseFloat(localStorage.getItem("settings.sfx_volume") ?? "1.0");
    let log_a = 1 / 10 ** 2;
    let log_b = Math.log(1 / log_a);
    let val = log_a * Math.exp(log_b * raw);
    if (raw < 0.01) {
        val = raw * 10 * log_a * Math.exp(log_b * 0.1);
    }
    return val;
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
        sfx.volume = get_sfx_volume() * 0.99;
        sfx.play();
    },
    toast: () => {
        /** @type HTMLAudioElement */ // @ts-ignore
        const sfx = sfx_store.toast.cloneNode();
        sfx.volume = get_sfx_volume() * 0.99;
        sfx.play();
    },
};
