const { Terra } = await import("./pkg/terra.js");
const terra = await Terra.load();
const scene = terra.create_scene("canvas");

window.setInterval(() => {
  window.requestAnimationFrame(() => scene.render());
}, 100);
