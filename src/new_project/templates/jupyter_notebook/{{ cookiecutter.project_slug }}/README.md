# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## Requirements

- Python 3.12+
- `uv`

## Setup

Install runtime and development dependencies:

```bash
uv sync --dev
```

This creates a local environment with the package itself, JupyterLab, and `pytest`.

## Usage

Run the package entry point:

```bash
uv run main
```

Run the module directly during development:

```bash
uv run python -m src.main
```

Start JupyterLab:

```bash
uv run notebook
```

The repository ships a wrapper around JupyterLab that disables password and token prompts for local development.

## Project Layout

- `src/main.py` contains the current CLI entry point.
- `src/notebook.py` starts JupyterLab with local-friendly auth settings.
- `notebook/` is the place for notebooks and related assets.
- `pyproject.toml` defines package metadata and script entry points.

## Development Notes

Refresh the lockfile after dependency changes:

```bash
uv lock
```

Run the test suite with:

```bash
uv run pytest
```

Tests live under `tests/`.
