"""CLI entry point for the new-project command."""

import typer

from new_project.commands.create import create_command
from new_project.commands.list_templates import list_templates_command
from new_project.commands.list_template_variables import list_template_variables_command

app = typer.Typer(no_args_is_help=True, add_completion=False)


@app.callback()
def main() -> None:
    """Root CLI group."""


app.command("create")(create_command)
app.command("list-templates")(list_templates_command)
app.command("list-template-variables")(list_template_variables_command)


if __name__ == "__main__":
    app()
