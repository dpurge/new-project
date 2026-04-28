# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## Features

- SolidJS single-page app built with Vite
- markdown documents loaded from `src/content/`
- YAML front matter for document metadata
- sidebar navigation generated from the directory structure
- multi-stage container build for a small runtime image

## Layout

- `src/content/`: markdown documents organized by section
- `src/content-loader.ts`: document loading, front matter parsing, and navigation tree building
- `src/app.tsx`: shell, sidebar, and document viewer
- `src/styles.css`: site styling
- `Dockerfile`: multi-stage container build
- `compose.yaml`: local container orchestration for quick startup
- `nginx.conf`: lightweight static file serving configuration

## Front matter

Each markdown file can start with YAML front matter:

```yaml
---
title: Getting Started
summary: Overview of the docs app
order: 1
---
```

Supported fields:

- `title`
- `summary`
- `order`

## Run

Install dependencies:

```bash
npm install
```

Start the dev server:

```bash
npm run dev
```

Build the site:

```bash
npm run build
```

Preview the production build:

```bash
npm run preview
```

## Container image

Build the container image:

```bash
docker build -t {{ cookiecutter.project_slug }} .
```

Run it locally:

```bash
docker run --rm -p 8080:8080 {{ cookiecutter.project_slug }}
```

The site will be available at `http://localhost:8080`.

## Docker Compose

Build and run the app with Compose:

```bash
docker compose up --build
```

Stop the app:

```bash
docker compose down
```
