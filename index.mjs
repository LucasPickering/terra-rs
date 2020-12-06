const CANVAS_ID = "canvas";

const { Terra } = await import("./pkg/terra.js");
const terra = await Terra.load(CANVAS_ID);

const canvas = document.getElementById(CANVAS_ID);

const resizeCanvas = () => {
  // TODO debounce this event listener
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;

  terra.render();
};

// Always size the canvas to fit the window
resizeCanvas();
window.addEventListener("resize", resizeCanvas);

window.setInterval(() => {
  window.requestAnimationFrame(() => terra.render());
}, 1000 / 60);
