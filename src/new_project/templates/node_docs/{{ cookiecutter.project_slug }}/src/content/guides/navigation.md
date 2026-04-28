---
title: Navigation
summary: Directory names become sections and files become documents.
order: 2
---

# Navigation

The app creates its sidebar from document paths.

## Rules

- `src/content/overview/getting-started.md` appears under the `Overview` section
- `src/content/guides/navigation.md` appears under the `Guides` section
- The current document is stored in the URL hash

## Why this shape

It keeps the authoring model simple:

- edit markdown
- commit files
- let the UI rebuild navigation automatically
