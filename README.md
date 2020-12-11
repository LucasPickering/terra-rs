# terra-rs

Hexagon tile-based terrain generation system. I originally wrote this in Java ([here](https://github.com/LucasPickering/terra)), but decided to re-write it in Rust here. This version is very primitive. Some basic terrain algorithms are implemented, but I've put this on hold for now until the Rust 3D graphics ecosystem matures more. I'll pick this back up at some point.

In the mean time, here's a screenshot of what this version can do so far:

![Terra screenshot](/screenshots/rust.png?raw=true "Terra")

And here's what the Java version looks like, for reference:

![Terra screenshot](/screenshots/java.jpg?raw=true "Terra Java")

## Development

### Prerequisites

- [rustup](https://rustup.rs/)
- [npm](https://www.npmjs.com/get-npm)

### Starting the App

```sh
rustup target add wasm32-unknown-unknown
cd typescript
npm install
npm run start
```

This will build the project and start an HTTP server on http://localhost:3000.
