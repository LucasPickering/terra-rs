# terra-rs

Hexagon tile-based terrain generation system. I originally wrote this in Java ([here](https://github.com/LucasPickering/terra)), but decided to re-write it in Rust here. This version is very primitive. Some basic terrain algorithms are implemented, but I've put this on hold for now until the Rust 3D graphics ecosystem matures more. I'll pick this back up at some point.

In the mean time, here's a screenshot of what this version can do so far:

![Terra screenshot](/screenshots/rust.png?raw=true "Terra")

And here's what the Java version looks like, for reference:

![Terra screenshot](/screenshots/java.jpg?raw=true "Terra Java")

## Development

### Prerequisites

- [rustup](https://rustup.rs/)
- [cargo-make](https://github.com/sagiegurari/cargo-make)

As you run commands, it will prompt you to install the other tools as necessary.

### Starting the App

```sh
cargo make start
```

This will build the project and start an HTTP server on http://localhost:3000. If you are missing any other dependencies, they'll be installed at that time.
