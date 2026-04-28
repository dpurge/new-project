# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## Features

- `clap` derive API for commands and options
- optional config file in the user's home directory
- a small command layout that is easy to extend

## Layout

- `src/main.rs`: bootstrap
- `src/cli.rs`: clap parser types
- `src/config.rs`: optional config loading from the user's home directory
- `src/app.rs`: command handlers

## Commands

- `{{ cookiecutter.binary_name }} greet`
- `{{ cookiecutter.binary_name }} config-path`

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

```bash
cargo run -- greet
```

Override values from the command line:

```bash
cargo run -- greet --name Alice --count 3
```

Print the resolved config path:

```bash
cargo run -- config-path
```
