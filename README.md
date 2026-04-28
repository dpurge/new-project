# The new-project command

`new-project` is a small CLI for scaffolding projects from internal templates.

## Call without installing

```sh
uvx --from https://github.com/dpurge/new-project.git new-project --help
```

## Layout

- `src/new_project/cli.py`: root Typer application
- `src/new_project/commands/`: thin CLI command wrappers
- `src/new_project/application/`: project-generation use cases
- `src/new_project/templates/`: internal template definitions
- `tests/`: CLI and application tests

## Usage

List the available templates:

```bash
uv run new-project list-templates
```

The output is rendered as a Rich table for readability.

Current templates include:

- `go-cli`
- `node-cli`
- `node-docs`
- `python-cli`
- `static-html-site`
- `rust-cli`
- `rust-rest-postgres`

List the context variables for a template:

```bash
uv run new-project list-template-variables static-html-site
```

The output is rendered as a Rich table showing variable names and defaults.

Create a new directory, move into it, and scaffold the first template:

```bash
mkdir my-site
cd my-site
uv run new-project create static-html-site
```

The `create` command renders an internal cookiecutter template and prompts for
all template variables. For `static-html-site`, it asks for:

- `project_name`
- `project_slug`

The generated project directory uses the entered `project_slug`.

To choose a different parent directory for the generated project, use
`--output-dir`:

```bash
uv run new-project create static-html-site --output-dir ./generated
```

To prefill or override cookiecutter variables, pass repeatable `--set key=value`
options:

```bash
uv run new-project create static-html-site \
  --set project_name="My Site" \
  --set project_slug=my-site
```

To skip prompts entirely and use template defaults for any missing values, use
`--defaults`:

```bash
uv run new-project create static-html-site --defaults
```

## Tests

Run the test suite with:

```bash
uv run pytest
```
