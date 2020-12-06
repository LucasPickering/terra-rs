const CANVAS_ID = "canvas";

const { Terra } = await import("./pkg/terra.js");
const terra = await Terra.load();
const scene = terra.create_scene(CANVAS_ID);

// const canvas = document.getElementById(CANVAS_ID);
// canvas.addEventListener(
//   "mousedown",
//   (e) => scene.handle_keyboard_event(e),
//   true
// );
// canvas.addEventListener("keydown", (e) => scene.handle_keyboard_event(e), true);

window.setInterval(() => {
  window.requestAnimationFrame(() => scene.render());
}, 100);
