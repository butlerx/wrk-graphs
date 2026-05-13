# Wrk Graphs

A web application for sharing and visualizing wrk loadtest and Criterion.rs
benchmark results. Built with Yew and Rust.

## Overview

This application allows you to:

- Share wrk loadtest results
- Share Criterion.rs benchmark results
- Visualize performance metrics
- Compare different test runs
- Collaborate on performance testing

## Supported Formats

### wrk

Paste the output from a `wrk` or `wrk2` loadtest run.

### Criterion.rs

Three input formats are supported:

- **CLI output** — paste the terminal output from `cargo bench`
- **JSON messages** — output from `cargo-criterion --message-format=json`
- **sample.json** — raw sample data from
  `target/criterion/<benchmark>/new/sample.json`

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
