"""CLI wrapper for project creation."""

from pathlib import Path
from typing import Annotated

import typer

from new_project.application.create_project import create_project
from new_project.errors import ProjectCreationError


def create_command(
    template_name: Annotated[str, typer.Argument(help="Internal template name to scaffold")],
    output_dir: Annotated[
        Path | None,
        typer.Option("--output-dir", "-o", help="Directory where the generated project should be created"),
    ] = None,
    use_defaults: Annotated[
        bool,
        typer.Option(
            "--defaults",
            help="Run non-interactively and use template defaults for missing values.",
        ),
    ] = False,
    extra_context_items: Annotated[
        list[str] | None,
        typer.Option(
            "--set",
            help="Cookiecutter extra context in key=value form. Repeat to pass multiple values.",
        ),
    ] = None,
) -> None:
    """Create a project from an internal template."""
    try:
        created_files = create_project(
            template_name=template_name,
            destination=output_dir,
            interactive=not use_defaults,
            extra_context=_parse_extra_context(extra_context_items or []),
        )
    except ProjectCreationError as exc:
        raise typer.BadParameter(str(exc)) from exc

    typer.echo(f"Created {len(created_files)} files from template '{template_name}'.")


def _parse_extra_context(items: list[str]) -> dict[str, str] | None:
    if not items:
        return None

    extra_context: dict[str, str] = {}
    for item in items:
        key, separator, value = item.partition("=")
        if not separator or not key:
            raise ProjectCreationError(
                f"Invalid extra context '{item}'. Expected key=value."
            )
        extra_context[key] = value

    return extra_context
