# Wrk Graphs

A web application for sharing and visualizing wrk loadtest results. Built with
Yew and Rust.

## Overview

This application allows you to:

- Share wrk loadtest results
- Visualize performance metrics
- Compare different test runs
- Collaborate on performance testing

## Prerequisites

If you don't already have it installed, it's time to install Rust:
<https://www.rust-lang.org/tools/install>. The rest of this guide assumes a
typical Rust installation which contains both `rustup` and Cargo.

To compile Rust to WASM, we need to have the `wasm32-unknown-unknown` target
installed and trunk. If you don't already them, install them with the following
command:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-bindgen-cli
```

## Development

To run the development server:

```bash
trunk serve
```

This will rebuild the app whenever a change is detected and run a local server
to host it.

## Building for Production

To create a production build:

```bash
trunk build --release
```

This builds the app in release mode. You can also pass the `--release` flag to
`trunk serve` if you need to get every last drop of performance.

Unless overwritten, the output will be located in the `dist` directory.

## License

This project is licensed under the Apache License 2.0 - see the
[LICENSE file](./LICENSE.md) for details.

## Author

Cian Butler <butlerx@notthe.cloud>
