"""Use case for listing available templates."""

from new_project.templates.base import Template
from new_project.templates.registry import list_templates


def get_templates() -> list[Template]:
    """Return all available templates in display order."""
    return list_templates()
