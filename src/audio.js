export const sfx = {
  drag_start: () => {
    const sfx = new Audio("/audio/drag-start.wav");
    sfx.volume = 0.3;
    sfx.play();
  },
  drag_end: () => {
    const sfx = new Audio("/audio/drag-end.wav");
    sfx.volume = 0.4;
    sfx.play();
  },
  trash: () => {
    const sfx = new Audio("/audio/trash.wav");
    sfx.volume = 0.6;
    sfx.play();
  },
  bubble: () => {
    const sfx = new Audio("/audio/bubble.wav");
    sfx.volume = 0.6;
    sfx.play();
  },
};
