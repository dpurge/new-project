"""Use cases for inspecting template context."""

import json
from importlib.resources import as_file, files
from typing import Any

from new_project.templates.registry import get_template


def get_template_context(template_name: str) -> dict[str, Any]:
    """Return the user-facing context variables for a template."""
    template = get_template(template_name)
    template_resource = files("new_project.templates") / template.directory_name

    with as_file(template_resource) as template_path:
        return _get_prompt_variables(template_path / "cookiecutter.json")


def _get_prompt_variables(cookiecutter_json_path) -> dict[str, Any]:
    template_context = json.loads(cookiecutter_json_path.read_text(encoding="utf-8"))
    return {
        key: value
        for key, value in template_context.items()
        if not key.startswith("_")
    }
