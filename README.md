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
  - [const_fn_trait_bound](https://github.com/rust-lang/rust/issues/57563)
- rustfmt
  - [importa_granularity](https://github.com/rust-lang/rustfmt/blob/master/Configurations.md#imports_granularity)
  - [wrap_comments](https://github.com/rust-lang/rustfmt/issues/3347)
- cargo
  - [namespaced-features](https://doc.rust-lang.org/cargo/reference/unstable.html#namespaced-features)
  - [per-package-target](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#per-package-target)
