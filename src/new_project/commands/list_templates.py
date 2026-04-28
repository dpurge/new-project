"""CLI wrapper for listing templates."""

from new_project.application.list_templates import get_templates
from new_project.presentation.console import console
from new_project.presentation.template_views import render_templates_table


def list_templates_command() -> None:
    """List all available internal templates."""
    console.print(render_templates_table(get_templates()))
