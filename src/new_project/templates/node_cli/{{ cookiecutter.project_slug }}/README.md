# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## Features

- Node.js CLI implemented with `commander`
- TypeScript project compiled with `tsc`
- optional config file loaded from the user's home directory

## Layout

- `package.json`: package metadata, scripts, and executable mapping
- `tsconfig.json`: TypeScript compiler configuration
- `src/index.ts`: CLI bootstrap
- `src/cli.ts`: command definitions
- `src/config.ts`: optional config loading and config path resolution
- `src/app.ts`: command handlers

## Commands

- `{{ cookiecutter.command_name }} greet`
- `{{ cookiecutter.command_name }} config-path`

## Config file

The app looks for an optional config file at:

```text
~/.config/{{ cookiecutter.project_slug }}/config.env
```

Example contents:

```text
name=CLI User
count=2
```

## Run

Install dependencies:

```bash
npm install
```

Build the CLI:

```bash
npm run build
```

Run the built CLI:

```bash
node dist/index.js greet
```

Override values from the command line:

```bash
node dist/index.js greet --name Alice --count 3
```

Print the resolved config path:

```bash
node dist/index.js config-path
```
