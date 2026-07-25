# deskspace

Unified file workspace server — browse, view, and edit files through a single surface.

## Overview

Deskspace exposes a workspace directory over HTTP, rendering each file through the
most appropriate available view — directory listing, syntax-highlighted source,
rendered Markdown, image preview — selected automatically based on the file's type.
File browsers and editors have historically been separate tools; deskspace treats that
separation as artificial. Browsing, viewing, and editing are the same operation on the
same surface, mediated by projections.

## Key Ideas

### Projections

A projection is a typed view of a resource. Different projections of the same file
coexist — a Markdown file can be viewed as rendered HTML or as raw text. Projections
implement a simple trait (`id`, `name`, `confidence`, `project`) and are registered at
startup; the server dispatches to the highest-confidence match per resource, or the
client can request a specific one via `?projection=<id>`.

Built-in projections: directory listing, plain text with syntax hints, Markdown with
TOC extraction, and image preview.

### Workspace safety

All file access is sandboxed to the workspace root. Path traversal attempts are
rejected at the boundary, not left to the caller.

### Read and write

`GET` requests view files through projections; `PUT` requests write raw bytes. The API
is minimal and composable, served by an Axum HTTP server bound to `127.0.0.1` with CSRF
checks on mutating requests from non-localhost origins.

## Status

MVP: a working projection-based file browser with workspace sandboxing, the projection
system, the four built-in projections, and a minimal browser UI. Not yet built: inline
editor integration, authentication for remote use, runtime plugin loading, and
workspace-wide search.

## Documentation

Full documentation: https://docs.rhi.zone/deskspace/

## License

Licensed under MIT OR Apache-2.0.
