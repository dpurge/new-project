---
title: Getting Started
summary: Start the docs site and understand the content model.
order: 1
---

# Getting Started

This project is a small documentation site built with SolidJS and Vite.

## How content works

- Every document lives in `src/content/`
- The folder structure becomes the navigation tree
- Each markdown file may include YAML front matter

## Example workflow

1. Create a new markdown file in a section directory.
2. Add `title`, `summary`, and `order` in the front matter.
3. Start the dev server and open the page.

```bash
npm install
npm run dev
```
