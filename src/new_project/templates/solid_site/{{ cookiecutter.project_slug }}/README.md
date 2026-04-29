# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

Generated with `new-project create solid-site`.

## Stack

- Deno task runner
- SolidStart application shell
- SolidBase for documentation layout and navigation
- MDX pages in `src/routes/`

## Layout

- `app.config.ts`: SolidStart and SolidBase configuration
- `deno.json`: Deno tasks for local development and builds
- `src/app.tsx`: root HTML shell
- `src/routes/`: MDX pages and the SolidBase catch-all route
- `public/`: static assets

## Run

Start the dev server:

```bash
deno task dev
```

Deno resolves the npm dependencies from `package.json` the first time you run a
task.

Build the site:

```bash
deno task build
```

Run the production server:

```bash
deno task start
```

Type-check the project:

```bash
deno task check
```
