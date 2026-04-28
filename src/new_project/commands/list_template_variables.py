"""CLI wrapper for listing template context variables."""

from typing import Annotated

import typer

from new_project.application.template_context import get_template_context
from new_project.errors import ProjectCreationError
from new_project.presentation.console import console
from new_project.presentation.template_views import render_template_context_table


def list_template_variables_command(
    template_name: Annotated[str, typer.Argument(help="Template name to inspect")],
) -> None:
    """List the context variables for a template."""
    try:
        context = get_template_context(template_name)
    except ProjectCreationError as exc:
        raise typer.BadParameter(str(exc)) from exc

    console.print(render_template_context_table(template_name, context))
