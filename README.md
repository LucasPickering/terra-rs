# terra-rs

**Try it out! [terra.lucaspickering.me](https://terra.lucaspickering.me)**

Hexagon tile-based terrain generation system. A series of configurable algorithsm can generated varied and realistic terrain.

## Usage

Want to use Terra? It has a native Rust interface, as well as a WebAssembly interface. View the code in `demo/` for an example of how to create Terra configs and generate worlds from Wasm.

## Development

### Prerequisites

- [rustup](https://rustup.rs/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [npm](https://www.npmjs.com/get-npm)

### Running the Demo

There is a minimal in-browser demo that's very useful for development.

```sh
cd demo
npm install
npm run start
```

This will build the project and start an HTTP server on http://localhost:3000.

## Deployment

Deployed through GitHub pages. Any push to master will trigger the CI to rebuild, which will updated the deployment.

## Rust Nightly

Currently we use Rust nightly because of a handful of features. Once this list gets down to zero, we can switch to stable.

- rustc
  - [const_fn_floating_point_arithmetic](https://github.com/rust-lang/rust/issues/57241)
- rustfmt
  - [imports_granularity](https://github.com/rust-lang/rustfmt/blob/master/Configurations.md#imports_granularity)
  - [wrap_comments](https://github.com/rust-lang/rustfmt/issues/3347)
- cargo
  - [per-package-target](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#per-package-target)

## Troubleshooting

### ARM + wasm-pack

If you're running on a Mac M1 or any other ARM system and get this error when trying to run the demo:

```
Error: no prebuilt wasm-opt binaries are available for this platform: Unrecognized target!
To disable `wasm-opt`, add `wasm-opt = false` to your package metadata in your `Cargo.toml`
```

Then make sure your installation of `wasm-pack` is at least version `0.10.2`. You can check with:

```sh
wasm-pack --version
```

You can upgrade by reinstalling with the link above.
