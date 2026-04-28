"""Use case for creating a project from a template."""

from importlib.resources import as_file, files
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any

from cookiecutter.main import cookiecutter

from new_project.errors import ProjectCreationError
from new_project.application.template_context import get_template_context
from new_project.templates.registry import get_template


def create_project(
    template_name: str,
    destination: Path | None = None,
    interactive: bool = True,
    extra_context: dict[str, Any] | None = None,
) -> list[Path]:
    """Create a project in the destination directory from the selected template."""
    target_dir = destination or Path.cwd()
    template = get_template(template_name)
    target_dir.mkdir(parents=True, exist_ok=True)
    template_resource = files("new_project.templates") / template.directory_name

    try:
        with TemporaryDirectory(dir=target_dir) as temp_dir:
            config_path = _write_cookiecutter_config(Path(temp_dir))

            with as_file(template_resource) as template_path:
                prompt_variables = get_template_context(template_name)
                merged_context = _collect_context(
                    prompt_variables=prompt_variables,
                    extra_context=extra_context,
                    interactive=interactive,
                )
                project_path = Path(
                    cookiecutter(
                        str(template_path),
                        no_input=True,
                        extra_context=merged_context,
                        output_dir=str(target_dir),
                        config_file=str(config_path),
                        accept_hooks="no",
                    )
                )
    except Exception as exc:
        raise ProjectCreationError(f"Failed to create project: {exc}") from exc

    return sorted(path for path in project_path.rglob("*") if path.is_file())


def _write_cookiecutter_config(work_dir: Path) -> Path:
    config_path = work_dir / "cookiecutter.yaml"
    replay_dir = work_dir / "replay"
    cookiecutters_dir = work_dir / "cookiecutters"
    replay_dir.mkdir(parents=True, exist_ok=True)
    cookiecutters_dir.mkdir(parents=True, exist_ok=True)
    config_path.write_text(
        "\n".join(
            [
                f'cookiecutters_dir: "{cookiecutters_dir}"',
                f'replay_dir: "{replay_dir}"',
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    return config_path


def _collect_context(
    prompt_variables: dict[str, Any],
    extra_context: dict[str, Any] | None,
    interactive: bool,
) -> dict[str, Any] | None:
    merged_context = dict(extra_context or {})

    for variable, default_value in prompt_variables.items():
        if variable in merged_context:
            continue
        if not interactive:
            continue
        merged_context[variable] = _prompt_for_value(variable, default_value)

    return merged_context or None


def _prompt_for_value(variable: str, default_value: Any) -> str:
    prompt = f"{variable} ({default_value}): "
    entered_value = input(prompt)
    return entered_value or str(default_value)
