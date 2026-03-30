---
layout: home

hero:
  name: Deskspace
  text: Unified File Workspace
  tagline: Browse, view, and edit files through a single seamless surface. The separation between file browser and editor was always artificial.
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/rhi-zone/deskspace

features:
  - title: Projection-Based Viewing
    details: Every file is rendered through the best available projection — directory listing, syntax-highlighted source, rendered Markdown, image preview. The right view is selected automatically.
  - title: Pluggable Projections
    details: Projections implement a simple trait. Register new ones at startup; the server picks the highest-confidence match per resource automatically.
  - title: Workspace Safety
    details: All file access is sandboxed to the workspace root. Path traversal attempts are rejected at the boundary, not left to the caller.
  - title: Read and Write
    details: GET requests view files through projections. PUT requests write raw bytes. The API is minimal and composable.
---
