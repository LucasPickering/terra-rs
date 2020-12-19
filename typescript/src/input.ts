/**
 * This type NEEDS to match the InputEvent in `input.rs`!
 */
type InputEvent =
  | {
      KeyDown: {
        key: string;
        repeat: boolean;
      };
    }
  | { KeyUp: { key: string } }
  | { MouseDown: { x: number; y: number } }
  | { MouseUp: { x: number; y: number } }
  | { MouseMove: { x: number; y: number } }
  | { Scroll: { up: boolean } }
  | { Blur: undefined };

/**
 * Map JS key strings to a string that our Rust enum can parse
 * @param key KeyboardEvent key string
 * @return Rust key value
 */
function convertKey(key: string): string {
  if (key.match(/^[a-zA-Z]$/)) {
    return key.toUpperCase();
  }
  if (key.match(/^[0-9]$/)) {
    return `Num${key}`;
  }

  switch (key.toLowerCase()) {
    case "arrowup":
      return "UpArrow";
    case "arrowdown":
      return "DownArrow";
    case "arrowleft":
      return "LeftArrow";
    case "arrowright":
      return "RightArrow";
    case "shift":
      return "LeftShift";
    case " ":
      return "Space";
    default:
      return "Unknown";
  }
}

class InputHandler {
  private canvas: HTMLCanvasElement;
  private handleEvent: (event: InputEvent) => void;

  constructor(
    canvas: HTMLCanvasElement,
    handleEvent: (event: InputEvent) => void
  ) {
    this.canvas = canvas;
    this.handleEvent = handleEvent;

    canvas.addEventListener("keydown", (e) =>
      this.handleEvent({
        KeyDown: { key: convertKey(e.key), repeat: e.repeat },
      })
    );
    canvas.addEventListener("keyup", (e) =>
      this.handleEvent({ KeyUp: { key: convertKey(e.key) } })
    );
    canvas.addEventListener("mousedown", (e) => {
      this.handleEvent({ MouseDown: { x: e.clientX, y: e.clientY } });
    });
    canvas.addEventListener("mouseup", (e) => {
      this.handleEvent({ MouseUp: { x: e.clientX, y: e.clientY } });
    });
    canvas.addEventListener("mousemove", (e) => {
      this.handleEvent({ MouseMove: { x: e.clientX, y: e.clientY } });
    });
    canvas.addEventListener("wheel", (e) =>
      this.handleEvent({ Scroll: { up: e.deltaY < 0 } })
    );
    canvas.addEventListener("blur", () =>
      this.handleEvent({ Blur: undefined })
    );
  }
}

export default InputHandler;
