# Wrk Graphs

A web application for sharing and visualizing
[wrk](https://github.com/wg/wrk) loadtest and
[Criterion.rs](https://github.com/bheisler/criterion.rs) benchmark results.
Built with [Yew](https://yew.rs) and Rust, compiled to WebAssembly.

## Supported Formats

### wrk

Paste the output from a `wrk` or `wrk2` loadtest run. The parser extracts
latency distributions, request rates, transfer rates, and error counts.

### Criterion.rs

Three input formats are supported:

- **CLI output** — paste the terminal output from `cargo bench`
- **JSON messages** — output from `cargo-criterion --message-format=json`
- **sample.json** — raw sample data from
  `target/criterion/<benchmark>/new/sample.json`

## Sharing

Results are serialized, compressed, and encoded into a shareable URL — no
server-side storage required. Anyone with the link can view the dashboard.

## Prerequisites

Install [mise](https://mise.jdx.dev/getting-started.html), then from the
project root:

```bash
mise install
```

This installs Rust (with the `wasm32-unknown-unknown` target), Trunk,
wasm-bindgen-cli, yew-fmt, tombi, and zizmor.

## Development

Start the development server:

```bash
mise run dev
```

This rebuilds the app on file changes and serves it locally.

## Building for Production

```bash
mise run build <public_url>
```

Output will be in the `dist` directory.

## Available Tasks

| Command                  | Description                              |
| ------------------------ | ---------------------------------------- |
| `mise run dev`           | Start dev server with hot-reload         |
| `mise run build`         | Build for production                     |
| `mise run test`          | Run tests                                |
| `mise run lint`          | Lint with clippy                         |
| `mise run lint:fix`      | Auto-fix lint issues                     |
| `mise run format`        | Format code (Rust, Yew HTML, TOML)       |
| `mise run format:check`  | Check formatting                         |
| `mise run audit`         | Audit dependencies for vulnerabilities   |
| `mise run check`         | Run all CI checks (format, lint, test, audit) |
| `mise run check:actions` | Lint GitHub Actions workflows            |
| `mise run clean`         | Clean build artifacts                    |

## Project Structure

```
src/
├── components/       # Yew components
│   ├── charts/       # Canvas-based chart renderers (wrk)
│   ├── criterion/    # Criterion-specific charts and tables
│   └── wrk/          # wrk-specific display components
├── drawing.rs        # Shared canvas drawing utilities
├── hooks.rs          # Custom Yew hooks (canvas, resize)
├── pages/            # Route-level page components
├── parser/           # Input format parsers (wrk, criterion)
└── serializer.rs     # URL-safe compression and encoding
styles/
├── base/             # Reset, variables, typography
├── components/       # Component-level styles (charts, modals)
└── layout/           # Page layout styles
```

## License

Licensed under the Apache License 2.0 — see [LICENSE.md](./LICENSE.md).

## Author

Cian Butler <butlerx@notthe.cloud>
