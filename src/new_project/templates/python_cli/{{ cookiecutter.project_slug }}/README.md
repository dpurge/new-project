# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## Features

- `uv`-managed Python project
- `Typer` CLI with multiple commands
- optional config file loaded from the user's home directory

## Layout

- `pyproject.toml`: project metadata and console script entry point
- `src/{{ cookiecutter.package_name }}/cli.py`: Typer application
- `src/{{ cookiecutter.package_name }}/config.py`: optional config loading
- `src/{{ cookiecutter.package_name }}/app.py`: command handlers

## Commands

- `{{ cookiecutter.command_name }} greet`
- `{{ cookiecutter.command_name }} config-path`

## Config file

The app looks for an optional config file at:

```text
{{ "{{" }} typer.get_app_dir("{{ cookiecutter.command_name }}") {{ "}}" }}/config.env
```

Example contents:

```text
name=CLI User
count=2
```

## Run

Install dependencies:

```bash
uv sync
```

Run the CLI:

```bash
uv run {{ cookiecutter.command_name }} greet
```

Override values from the command line:

```bash
uv run {{ cookiecutter.command_name }} greet --name Alice --count 3
```

Print the resolved config path:

```bash
uv run {{ cookiecutter.command_name }} config-path
```
