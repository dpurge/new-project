# Repository Guidelines

## Project Structure & Module Organization
The project is a small Python package managed with `uv`. Core application code lives in `src/`, with the current CLI entry point in `src/main.py`. Package metadata and script definitions are in `pyproject.toml`, and locked dependencies are recorded in `uv.lock`. Use `notebook/` for notebooks and related assets; keep exploratory work there rather than in `src/`. There is no `tests/` directory yet, so add one at the repository root when introducing automated tests.

## Build, Test, and Development Commands
Use `uv` for local workflows:

- `uv sync` installs project dependencies into the local environment.
- `uv run main` runs the package script defined in `pyproject.toml`.
- `uv run python -m src.main` runs the module directly during development.
- `uv run notebook` starts JupyterLab using the configured script entry point.

If you add a formatter, linter, or test runner, expose it through `uv run ...` and document it here.

## Coding Style & Naming Conventions
Target Python 3.12+ and follow standard PEP 8 conventions: 4-space indentation, snake_case for functions and modules, and PascalCase for classes. Keep modules focused and small; move reusable logic out of notebook cells and into `src/`. Prefer descriptive filenames such as `data_loader.py` or `prompt_runner.py` over vague names like `utils.py`.

## Testing Guidelines
No testing framework is configured yet. When adding tests, prefer `pytest`, place tests under `tests/`, and name files `test_<module>.py`. Match test names to behaviors, for example `test_main_prints_greeting`. Run tests with `uv run pytest` once `pytest` is added to dependencies.

## Commit & Pull Request Guidelines
This repository currently has no commit history, so there is no established commit convention to copy. Use short, imperative subjects such as `Add notebook bootstrap command` and keep each commit focused on one change. Pull requests should include a clear summary, any setup or verification steps, and screenshots only when notebook UI changes are relevant.

## Repository Notes
Do not commit `.venv/` contents or generated notebook output unless the output is intentionally part of the deliverable. Keep `README.md` updated when adding new commands or project structure.
