# Architecture

## Overview

Deskspace is a single Rust crate (`deskspace`) that bundles a workspace abstraction, a
projection system, an HTTP API, and a static UI server. There is no plugin loading at
runtime — projections are registered at startup in `main.rs`.

## Module layout

```
deskspace/
  workspace.rs      — sandboxed file I/O (Workspace, WorkspaceError)
  projection.rs     — Projection trait, Resource, ProjectionOutput
  registry.rs       — ProjectionRegistry (registration + confidence-based dispatch)
  projections/
    dir_list.rs     — directory listing
    text_raw.rs     — plain text with syntax language hints
    text_markdown.rs — Markdown with TOC extraction
    image_preview.rs — image preview (returns a URL for raw file access)
  api/
    mod.rs          — AppState, router, CSRF middleware
    files.rs        — /api/files/* route handlers
  main.rs           — server startup, projection registration, address binding
```

## Workspace

`Workspace` wraps a directory root and provides safe, async file I/O:

- `resolve(path)` — canonicalizes a relative path and rejects anything escaping the root
- `read`, `read_to_string`, `write`, `metadata`, `read_dir` — async wrappers over Tokio fs
- `read_dir` returns entries sorted: directories first, then alphabetically by name

Absolute paths passed to `resolve` have their leading `/` stripped before joining, so
`/sub/file.txt` resolves as `<root>/sub/file.txt`.

## Projection system

```rust
#[async_trait]
pub trait Projection: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn confidence(&self, resource: &Resource) -> f32;
    async fn project(&self, resource: &Resource, workspace: &Workspace) -> Result<ProjectionOutput>;
}
```

A `Resource` carries the path, whether it is a directory, and the lowercase file
extension. `confidence` returns a score in `[0, 1]`; returning 0 means "I cannot handle
this resource".

`ProjectionRegistry` holds projections keyed by ID:

- `best_for(resource)` — returns the highest-confidence projection for the resource
- `available_for(resource)` — returns all matching projections sorted by confidence, serializable as JSON

## Projection output

```rust
pub enum ProjectionOutput {
    DirectoryList { entries: Vec<DirectoryEntry> },
    Text { content: String, language: Option<String>, line_count: usize },
    Markdown { raw: String, toc: Vec<TocEntry> },
    Image { mime_type: String, url: String },
}
```

All variants serialize to JSON with a `"type"` discriminant (via `#[serde(tag = "type")]`).

## HTTP API

Routes defined in `api::router`:

| Method | Path | Handler |
|--------|------|---------|
| GET | `/api/files/` | `files::get_root` |
| GET | `/api/files/{*path}` | `files::get_file` |
| PUT | `/api/files/{*path}` | `files::put_file` |
| GET | `/api/files/raw/{*path}` | `files::raw_file` |

`GET /api/files/` and `GET /api/files/{*path}` accept an optional `?projection=<id>`
query parameter to select a specific projection. Without it, the highest-confidence
projection is used.

Response shape for projected requests:

```json
{
  "path": "src/main.rs",
  "is_dir": false,
  "projections": [
    { "id": "text.raw", "name": "Plain Text", "confidence": 0.8 }
  ],
  "active_projection": "text.raw",
  "output": { "type": "Text", "content": "...", "language": "rust", "line_count": 42 }
}
```

`GET /api/files/raw/{*path}` returns the raw file bytes with a MIME type inferred from
the extension (via `mime_guess`).

`PUT /api/files/{*path}` writes the request body to the workspace path. Parent
directories are created automatically.

## CSRF protection

All mutating requests (non-GET, non-HEAD) pass through `csrf_check` middleware. Requests
with an `Origin` header from a non-localhost origin are rejected with 403. Requests with
no `Origin` header are allowed (same-origin browser requests and non-browser clients).

## Startup

`main.rs` wires everything together:

1. Parse the workspace root from argv (defaults to `.`)
2. Create `Workspace`
3. Build `ProjectionRegistry`, register all four built-in projections
4. Wrap state in `Arc<AppState>`
5. Mount the Axum router; serve `ui/` as static files at `/`
6. Bind `127.0.0.1:3000` and serve

The UI directory is resolved relative to the binary's location (three levels up from the
binary, then `ui/`), falling back to `./ui` relative to the working directory.
