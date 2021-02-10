# terra-rs

**Try it out! [terra.lucaspickering.me](https://terra.lucaspickering.me)**

Hexagon tile-based terrain generation system. I originally wrote this in Java ([here](https://github.com/LucasPickering/terra)), but decided to re-write it in Rust here. This version isn't as advanced as the Java one, but I'm working on it.

For reference, here's what the Java version looks like:

![Terra screenshot](/screenshots/java.jpg?raw=true "Terra Java")

## Development

### Prerequisites

- [rustup](https://rustup.rs/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [npm](https://www.npmjs.com/get-npm)

### Starting the App

```sh
rustup target add wasm32-unknown-unknown
cd typescript
npm install
npm run start
```

This will build the project and start an HTTP server on http://localhost:3000.

## Deployment

Deployed through GitHub pages. Any push to master will trigger the CI to rebuild, which will updated the deployment.
