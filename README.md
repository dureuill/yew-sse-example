yew-sse-example
===============

This repository is a proof of concept of:

- Serving a [yew](https://yew.rs) frontend application from a [rocket](https://rocket.rs) backend
- Using the [yew-sse](https://github.com/liquidnya/yew-sse) library to send messages and receive them from several connected clients.

This example is not meant to be *minimal*. Rather, it's meant to be *fun*, and so it uses:

- The [cargo xtask](https://github.com/matklad/cargo-xtask) pattern to distribute and serve the frontend application from the backend.
- [trunk](https://trunkrs.dev) to perform the "distribution" step of the frontend
- [clap](https://clap.rs/) for ergonomic CLI definition
- [color-eyre](https://github.com/yaahc/color-eyre) for nice, colored errors :sparkles:
- [duct](https://github.com/oconnor663/duct.rs), for easy scripting in Rust


What it does **not** demonstrate though, is the vastness of my skills at CSS :stuck_out_tongue:.

## How to use

### Prerequisites

You'll need Rust installed, with the `wasm32-unknown-unknown` target available, as well as the [trunk](https://trunkrs.dev) tool.

To install `trunk`, just run:

```
cargo install trunk
```

### Serving the example

From the repository's root directory, run:

```
cargo xtask run
```

(add the `--release` flag to build everything in `--release` mode)

After the build, once rocket is started, open [http://127.0.0.1:8000](http://127.0.0.1:8000) in your browser to use the yew application.
Open several times the same page to exchange messages! Very convenient :sweat_smile:
