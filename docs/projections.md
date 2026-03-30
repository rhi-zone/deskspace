# Projections

A projection is a typed view of a file or directory. The server selects the
highest-confidence projection for each resource by default; clients can request a
specific one by ID.

## Built-in projections

### `dir.list` — Directory Listing

Handles directories only (confidence 1.0 for directories, 0.0 for files).

Returns a `DirectoryList` with entries sorted directories-first, then alphabetically:

```json
{
  "type": "DirectoryList",
  "entries": [
    { "name": "src", "is_dir": true, "size": 0, "extension": null },
    { "name": "Cargo.toml", "is_dir": false, "size": 512, "extension": "toml" }
  ]
}
```

### `text.raw` — Plain Text

Handles text files (confidence 0.8 for known text extensions, 0.3 for extensionless
files, 0.0 for directories and unknown binary extensions).

Known extensions include: `rs`, `toml`, `json`, `yaml`, `md`, `ts`, `js`, `py`, `lua`,
`nix`, `sh`, `sql`, and dozens more common source and config formats.

Returns a `Text` output with the file content, an optional language hint for syntax
highlighting, and a line count:

```json
{
  "type": "Text",
  "content": "fn main() { ... }",
  "language": "rust",
  "line_count": 42
}
```

### `text.markdown` — Markdown

Handles `.md` and `.markdown` files (confidence 1.0). Takes priority over `text.raw`
for Markdown files.

Returns `Markdown` output with the raw source and an extracted table of contents:

```json
{
  "type": "Markdown",
  "raw": "# Hello\n\nSome text\n\n## World\n",
  "toc": [
    { "level": 1, "text": "Hello", "slug": "hello" },
    { "level": 2, "text": "World", "slug": "world" }
  ]
}
```

TOC slugs are lowercased, spaces replaced with `-`, non-alphanumeric characters removed.

### `image.preview` — Image Preview

Handles `png`, `jpg`, `jpeg`, `gif`, `webp`, and `svg` files (confidence 1.0).

Rather than embedding the raw bytes in the JSON response, returns a URL pointing to the
raw file endpoint:

```json
{
  "type": "Image",
  "mime_type": "image/png",
  "url": "/api/files/raw/path/to/image.png"
}
```

The client fetches the image directly from the raw endpoint.

## Writing a custom projection

Implement the `Projection` trait:

```rust
use async_trait::async_trait;
use deskspace::projection::{Projection, ProjectionOutput, Resource, Result};
use deskspace::workspace::Workspace;

pub struct MyProjection;

#[async_trait]
impl Projection for MyProjection {
    fn id(&self) -> &str {
        "my.projection"
    }

    fn name(&self) -> &str {
        "My Projection"
    }

    fn confidence(&self, resource: &Resource) -> f32 {
        match resource.extension.as_deref() {
            Some("myext") => 1.0,
            _ => 0.0,
        }
    }

    async fn project(
        &self,
        resource: &Resource,
        workspace: &Workspace,
    ) -> Result<ProjectionOutput> {
        let content = workspace
            .read_to_string(std::path::Path::new(&resource.path))
            .await?;
        // Transform content as needed...
        Ok(ProjectionOutput::Text {
            content,
            language: Some("myext".to_string()),
            line_count: 0,
        })
    }
}
```

Register it in `main.rs`:

```rust
registry.register(Arc::new(MyProjection));
```

If multiple projections match a resource, the one with the highest confidence wins. If
two projections return the same confidence, the result is implementation-defined (HashMap
iteration order).
