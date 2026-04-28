"""CLI entry point."""

import typer

from {{ cookiecutter.package_name }}.app import config_path_command, greet_command

app = typer.Typer(no_args_is_help=True, add_completion=False)
app.command("greet")(greet_command)
app.command("config-path")(config_path_command)


if __name__ == "__main__":
    app()
