# Getting Started

## Running the server

```bash
git clone https://github.com/rhi-zone/deskspace
cd deskspace
nix develop    # Enter dev shell
cargo build --release
```

Start the server, pointing it at a directory to serve:

```bash
cargo run -- /path/to/your/files
# or after build:
./target/release/deskspace /path/to/your/files
```

The server listens on `http://127.0.0.1:3000`.

- **`http://127.0.0.1:3000/`** — serves the UI
- **`http://127.0.0.1:3000/api/files/`** — projected view of the workspace root
- **`http://127.0.0.1:3000/api/files/path/to/file`** — projected view of a specific file

## UI

The UI is a minimal vanilla JS frontend in `ui/`. It reads projection output from the
API and renders it in the browser. It uses highlight.js for syntax highlighting and
respects the system color scheme preference.

To build the UI:

```bash
cd ui
bun install
bun run build   # outputs to ui/dist/
```

The server serves `ui/` as a static directory; `ui/dist/app.js` is loaded by
`ui/index.html`.

## Development

```bash
nix develop
cargo test       # Run tests (workspace + unit tests)
cargo clippy     # Lint
cd docs && bun dev  # Local docs preview
```

Run the server against a test directory:

```bash
cargo run -- ~/some-dir
```

Use `RUST_LOG=debug` for verbose output:

```bash
RUST_LOG=debug cargo run -- ~/some-dir
```
