"""Template registry."""

from new_project.errors import ProjectCreationError
from new_project.templates.base import Template

TEMPLATES: dict[str, Template] = {
    "go-cli": Template(
        name="go-cli",
        description="A Go CLI application with optional configuration loaded from the user's home directory.",
        directory_name="go_cli",
    ),
    "node-cli": Template(
        name="node-cli",
        description="A Node.js CLI application using TypeScript and commander with optional configuration loaded from the user's home directory.",
        directory_name="node_cli",
    ),
    "node-docs": Template(
        name="node-docs",
        description="A SolidJS documentation site that serves markdown documents with YAML front matter from a navigable directory tree.",
        directory_name="node_docs",
    ),
    "python-cli": Template(
        name="python-cli",
        description="A Python CLI application using uv and Typer with optional configuration loaded from the user's home directory.",
        directory_name="python_cli",
    ),
    "rust-cli": Template(
        name="rust-cli",
        description="A Rust CLI application using clap with optional configuration loaded from the user's home directory.",
        directory_name="rust_cli",
    ),
    "rust-rest-postgres": Template(
        name="rust-rest-postgres",
        description="A Rust REST service using axum, tokio, serde, sqlx, and tower-http with a PostgreSQL backend.",
        directory_name="rust_rest_postgres",
    ),
    "static-html-site": Template(
        name="static-html-site",
        description="A minimal static HTML site with semantic structure and a stylesheet.",
        directory_name="static_html_site",
    ),
}


def get_template(template_name: str) -> Template:
    """Return the named template or raise a clear error."""
    try:
        return TEMPLATES[template_name]
    except KeyError as exc:
        available = ", ".join(sorted(TEMPLATES))
        raise ProjectCreationError(
            f"Unknown template '{template_name}'. Available templates: {available}."
        ) from exc


def list_templates() -> list[Template]:
    """Return all registered templates."""
    return [TEMPLATES[name] for name in sorted(TEMPLATES)]
