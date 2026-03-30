# API Reference

The Deskspace HTTP API is served at `http://127.0.0.1:3000/api/`. All endpoints are
relative to the configured workspace root.

## GET /api/files/

Returns the projected view of the workspace root directory.

**Query parameters:**
- `projection` (optional) — ID of a specific projection to use (e.g. `dir.list`)

**Response:**

```json
{
  "path": "",
  "is_dir": true,
  "projections": [
    { "id": "dir.list", "name": "Directory Listing", "confidence": 1.0 }
  ],
  "active_projection": "dir.list",
  "output": {
    "type": "DirectoryList",
    "entries": [...]
  }
}
```

## GET /api/files/{path}

Returns the projected view of the file or directory at `path`.

**Path:** URL-encoded relative path within the workspace (e.g. `src/main.rs`).

**Query parameters:**
- `projection` (optional) — ID of a specific projection to use

**Response:** Same structure as above, with `path` set to the requested path.

**Errors:**
- `400 Bad Request` — path escapes the workspace root, or unknown projection ID requested
- `404 Not Found` — path does not exist, or no projection is available for this resource
- `500 Internal Server Error` — projection or I/O failure

## PUT /api/files/{path}

Writes the request body to the file at `path`. Parent directories are created
automatically.

**Request body:** Raw file bytes (any content type).

**Response:** `204 No Content` on success.

**Errors:**
- `400 Bad Request` — path escapes the workspace root or other workspace error

**Note:** Mutating requests are subject to CSRF checks. Requests with a non-localhost
`Origin` header are rejected with `403 Forbidden`.

## GET /api/files/raw/{path}

Returns the raw file bytes with an appropriate `Content-Type` header inferred from the
file extension.

**Response:** Raw file bytes. Content-Type is determined by `mime_guess`; falls back to
`application/octet-stream`.

**Note:** This route does not go through the projection system. It is used by the
`image.preview` projection to provide a URL the client can load directly.

## Available projections

The `projections` array in every file response lists all projections that can handle the
resource, sorted by confidence descending. Use the `id` from this list with the
`?projection=` query parameter to select a specific view.

Example: viewing a `.md` file as plain text rather than rendered Markdown:

```
GET /api/files/README.md?projection=text.raw
```
