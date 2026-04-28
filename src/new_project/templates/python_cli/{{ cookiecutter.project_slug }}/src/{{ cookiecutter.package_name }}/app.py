"""Application commands."""

from typing import Annotated

import typer

from {{ cookiecutter.package_name }}.config import config_path, load_config


def greet_command(
    name: Annotated[str | None, typer.Option("--name", help="Name to greet")] = None,
    count: Annotated[int | None, typer.Option("--count", help="Number of greetings to print")] = None,
) -> None:
    config = load_config()
    resolved_name = name or config.name or "World"
    resolved_count = count or config.count or 1

    for _ in range(resolved_count):
        typer.echo(f"Hello, {resolved_name}!")


def config_path_command() -> None:
    typer.echo(str(config_path()))
