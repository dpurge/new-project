"""Rich renderables for template commands."""

import json
from typing import Any

from rich.table import Table

from new_project.templates.base import Template


def render_templates_table(templates: list[Template]) -> Table:
    """Build a table for available templates."""
    table = Table(title="Available Templates", header_style="bold cyan")
    table.add_column("Template", style="bold")
    table.add_column("Description")

    for template in templates:
        table.add_row(template.name, template.description)

    return table


def render_template_context_table(
    template_name: str,
    context: dict[str, Any],
) -> Table:
    """Build a table for a template's context variables."""
    table = Table(
        title=f"Template Variables: {template_name}",
        header_style="bold cyan",
    )
    table.add_column("Variable", style="bold")
    table.add_column("Default")

    for key, value in context.items():
        table.add_row(key, json.dumps(value))

    return table
