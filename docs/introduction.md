# Introduction

Deskspace is a unified file workspace server. It exposes a workspace directory over HTTP,
rendering each file through the most appropriate available view — directory listing,
syntax-highlighted source, rendered Markdown, image preview — selected automatically
based on the file's type.

## The idea

File browsers and editors have historically been separate tools. Deskspace treats the
separation as artificial: browsing, viewing, and editing are the same operation on the
same surface, mediated by projections.

A projection is a typed view of a resource. Different projections of the same file
coexist — you can view a Markdown file as rendered HTML or as raw text. The server
selects the best match by default; the client can request a specific one.

## Current state (MVP)

The MVP is a working projection-based file browser:

- **Workspace** — sandboxed file access with path-traversal rejection, async read/write/metadata via Tokio
- **Projection system** — `Projection` trait with confidence-based dispatch, `ProjectionRegistry` for registration and selection
- **Built-in projections** — directory listing, plain text with syntax hints, Markdown with TOC extraction, image preview
- **HTTP API** — Axum server on `localhost:3000`, routes for projected views, raw file access, and file writes
- **UI** — minimal browser-based frontend (vanilla JS + highlight.js, served from `ui/`) that renders projection output

The server is intentionally local-first. It binds to `127.0.0.1` and applies CSRF
checks to reject mutating requests from non-localhost origins.

## What's not yet built

- Editor integration (inline editing through the UI)
- Authentication / access control for remote use
- Plugin loading at runtime (projections are registered at startup in code)
- Search across the workspace
