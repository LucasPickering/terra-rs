const CANVAS_ID = "canvas";
const TARGET_FRAME_RATE = 60;

const { Terra } = await import("terra-wasm");
const terra = await Terra.load(CANVAS_ID);

// type safety!
const canvas: HTMLCanvasElement = document.getElementById(
  CANVAS_ID
)! as HTMLCanvasElement;

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
}, 1000 / TARGET_FRAME_RATE);

export {}; // #computerscience
