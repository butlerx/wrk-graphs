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

Install [mise](https://mise.jdx.dev/getting-started.html), then from the
project root:

```bash
mise install
```

This installs Rust (with rust-analyzer and the wasm32-unknown-unknown target),
Trunk, wasm-bindgen-cli, tombi, and zizmor at the correct versions.

## Development

Start the development server:

```bash
mise run dev
```

This will rebuild the app whenever a change is detected and run a local server
to host it.

## Building for Production

```bash
mise run build
```

Output will be in the `dist` directory.

## Available Tasks

| Command                  | Description                        |
| ------------------------ | ---------------------------------- |
| `mise run dev`           | Start dev server with hot-reload   |
| `mise run build`         | Build for production               |
| `mise run test`          | Run tests                          |
| `mise run lint`          | Run clippy                         |
| `mise run lint:fix`      | Auto-fix lint issues               |
| `mise run format`        | Format code (Rust + TOML)          |
| `mise run format:check`  | Check formatting                   |
| `mise run check`         | Run all CI checks                  |
| `mise run check:actions` | Lint GitHub Actions workflows      |
| `mise run clean`         | Clean build artifacts              |

## License

This project is licensed under the Apache License 2.0 - see the
[LICENSE file](./LICENSE.md) for details.

## Author

Cian Butler <butlerx@notthe.cloud>
