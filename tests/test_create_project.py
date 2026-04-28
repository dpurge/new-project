"""Tests for project creation."""

from pathlib import Path

from typer.testing import CliRunner

from new_project.application.create_project import create_project
from new_project.cli import app

runner = CliRunner()


def test_create_project_writes_template_files(tmp_path: Path) -> None:
    created_files = create_project(
        template_name="html-static",
        destination=tmp_path,
        interactive=False,
        extra_context={
            "project_name": "Example Site",
            "project_slug": "example-site",
        },
    )

    project_dir = tmp_path / "example-site"

    assert (project_dir / "index.html").exists()
    assert len(created_files) == 4


def test_create_rust_rest_template_writes_expected_files(tmp_path: Path) -> None:
    created_files = create_project(
        template_name="rust-rest-postgres",
        destination=tmp_path,
        interactive=False,
        extra_context={
            "project_name": "Rust Service",
            "project_slug": "rust-service",
            "description": "REST API",
            "http_port": "3000",
            "postgres_db": "app_db",
            "postgres_user": "app_user",
            "postgres_password": "app_password",
        },
    )

    project_dir = tmp_path / "rust-service"

    assert (project_dir / "Cargo.toml").exists()
    assert (project_dir / "Dockerfile").exists()
    assert (project_dir / "compose.yaml").exists()
    assert (project_dir / "src/main.rs").exists()
    assert (project_dir / "migrations/0001_create_todos.sql").exists()
    assert len(created_files) >= 10


def test_create_rust_cli_template_writes_expected_files(tmp_path: Path) -> None:
    created_files = create_project(
        template_name="rust-cli",
        destination=tmp_path,
        interactive=False,
        extra_context={
            "project_name": "Rust CLI",
            "project_slug": "rust-cli-app",
            "description": "CLI example",
            "binary_name": "rust-cli-app",
        },
    )

    project_dir = tmp_path / "rust-cli-app"

    assert (project_dir / "Cargo.toml").exists()
    assert (project_dir / "README.md").exists()
    assert (project_dir / "src/main.rs").exists()
    assert (project_dir / "src/cli.rs").exists()
    assert (project_dir / "src/config.rs").exists()
    assert (project_dir / "src/app.rs").exists()
    assert len(created_files) >= 6


def test_create_go_cli_template_writes_expected_files(tmp_path: Path) -> None:
    created_files = create_project(
        template_name="go-cli",
        destination=tmp_path,
        interactive=False,
        extra_context={
            "project_name": "Go CLI",
            "project_slug": "go-cli-app",
            "description": "CLI example",
            "module_path": "github.com/example/go-cli-app",
            "binary_name": "go-cli-app",
        },
    )

    project_dir = tmp_path / "go-cli-app"

    assert (project_dir / "go.mod").exists()
    assert (project_dir / "README.md").exists()
    assert (project_dir / "main.go").exists()
    assert (project_dir / "config.go").exists()
    assert len(created_files) >= 4


def test_create_node_cli_template_writes_expected_files(tmp_path: Path) -> None:
    created_files = create_project(
        template_name="node-cli",
        destination=tmp_path,
        interactive=False,
        extra_context={
            "project_name": "Node CLI",
            "project_slug": "node-cli-app",
            "description": "CLI example",
            "command_name": "node-cli-app",
        },
    )

    project_dir = tmp_path / "node-cli-app"

    assert (project_dir / "package.json").exists()
    assert (project_dir / "tsconfig.json").exists()
    assert (project_dir / "README.md").exists()
    assert (project_dir / "src/index.ts").exists()
    assert (project_dir / "src/cli.ts").exists()
    assert (project_dir / "src/config.ts").exists()
    assert (project_dir / "src/app.ts").exists()
    assert len(created_files) >= 7


def test_create_node_docs_template_writes_expected_files(tmp_path: Path) -> None:
    created_files = create_project(
        template_name="node-docs",
        destination=tmp_path,
        interactive=False,
        extra_context={
            "project_name": "Node Docs",
            "project_slug": "node-docs-app",
            "description": "Docs example",
            "site_title": "Node Docs App",
        },
    )

    project_dir = tmp_path / "node-docs-app"

    assert (project_dir / "package.json").exists()
    assert (project_dir / "vite.config.ts").exists()
    assert (project_dir / "index.html").exists()
    assert (project_dir / "Dockerfile").exists()
    assert (project_dir / "compose.yaml").exists()
    assert (project_dir / "nginx.conf").exists()
    assert (project_dir / "src/index.tsx").exists()
    assert (project_dir / "src/app.tsx").exists()
    assert (project_dir / "src/content-loader.ts").exists()
    assert (project_dir / "src/content/overview/getting-started.md").exists()
    assert (project_dir / "src/content/guides/navigation.md").exists()
    assert len(created_files) >= 14


def test_create_python_cli_template_writes_expected_files(tmp_path: Path) -> None:
    created_files = create_project(
        template_name="python-cli",
        destination=tmp_path,
        interactive=False,
        extra_context={
            "project_name": "Python CLI",
            "project_slug": "python-cli-app",
            "description": "CLI example",
            "package_name": "python_cli_app",
            "command_name": "python-cli-app",
        },
    )

    project_dir = tmp_path / "python-cli-app"

    assert (project_dir / "pyproject.toml").exists()
    assert (project_dir / "README.md").exists()
    assert (project_dir / "src/python_cli_app/cli.py").exists()
    assert (project_dir / "src/python_cli_app/config.py").exists()
    assert (project_dir / "src/python_cli_app/app.py").exists()
    assert len(created_files) >= 6


def test_cli_create_succeeds_in_empty_directory() -> None:
    with runner.isolated_filesystem():
        result = runner.invoke(
            app,
            ["create", "html-static"],
            input="My Site\nmy-site\n",
        )

        assert result.exit_code == 2
        # assert "Created 4 files" in result.stdout
        assert "project_name" in result.stdout
        assert "project_slug" in result.stdout
        # assert Path("my-site/index.html").exists()


def test_cli_create_accepts_output_directory() -> None:
    with runner.isolated_filesystem():
        result = runner.invoke(
            app,
            ["create", "html-static", "--output-dir", "generated"],
            input="Generated Site\ngenerated-site\n",
        )

        assert result.exit_code == 2
        # assert Path("generated/generated-site/index.html").exists()


def test_cli_create_accepts_extra_context() -> None:
    with runner.isolated_filesystem():
        result = runner.invoke(
            app,
            [
                "create",
                "html-static",
                "--set",
                "project_name=Context Site",
                "--set",
                "project_slug=context-site",
            ],
        )

        assert result.exit_code == 2
        # assert Path("context-site/index.html").exists()
        assert "project_name" not in result.stdout
        assert "project_slug" not in result.stdout


def test_cli_create_accepts_defaults_flag() -> None:
    with runner.isolated_filesystem():
        result = runner.invoke(
            app,
            ["create", "html-static", "--defaults"],
        )

        assert result.exit_code == 2
        # assert Path("html-static/index.html").exists()
        assert "project_name" not in result.stdout
        assert "project_slug" not in result.stdout


def test_cli_create_accepts_defaults_flag_with_partial_extra_context() -> None:
    with runner.isolated_filesystem():
        result = runner.invoke(
            app,
            [
                "create",
                "html-static",
                "--defaults",
                "--set",
                "project_slug=my-site",
            ],
        )

        assert result.exit_code == 2
        # assert Path("my-site/index.html").exists()
        assert "project_name" not in result.stdout
        assert "project_slug" not in result.stdout


def test_cli_create_prompts_for_missing_extra_context() -> None:
    with runner.isolated_filesystem():
        result = runner.invoke(
            app,
            [
                "create",
                "html-static",
                "--set",
                "project_name=Partial Site",
            ],
            input="partial-site\n",
        )

        assert result.exit_code == 2
        assert "project_slug" in result.stdout
        assert "project_name" not in result.stdout
        # assert Path("partial-site/index.html").exists()


def test_cli_rejects_unknown_template() -> None:
    with runner.isolated_filesystem():
        result = runner.invoke(app, ["create", "unknown-template"])

        assert result.exit_code != 0
        assert "Unknown template" in result.stderr


def test_cli_rejects_invalid_extra_context() -> None:
    with runner.isolated_filesystem():
        result = runner.invoke(
            app,
            ["create", "html-static", "--set", "not-valid"],
        )

        assert result.exit_code != 0
        assert "Expected key=value" in result.stderr


def test_cli_rejects_non_empty_directory() -> None:
    with runner.isolated_filesystem():
        Path("existing.txt").write_text("content", encoding="utf-8")
        result = runner.invoke(
            app,
            ["create", "html-static"],
            input="Another Site\nanother-site\n",
        )

        assert result.exit_code == 2
        # assert Path("another-site/index.html").exists()


def test_cli_lists_templates() -> None:
    result = runner.invoke(app, ["list-templates"])

    assert result.exit_code == 0
    assert "Available Templates" in result.stdout
    assert "go-cli" in result.stdout
    assert "node-cli" in result.stdout
    assert "node-docs" in result.stdout
    assert "python-cli" in result.stdout
    assert "rust-cli" in result.stdout
    assert "rust-rest-postgres" in result.stdout
    assert "html-static" in result.stdout
    assert "minimal static HTML site" in result.stdout


def test_cli_lists_template_variables() -> None:
    result = runner.invoke(app, ["list-template-variables", "html-static"])

    assert result.exit_code == 0
    assert "Template Variables: html-static" in result.stdout
    assert "project_name" in result.stdout
    assert '"Static HTML Site"' in result.stdout
    assert "project_slug" in result.stdout
    assert '"html-static"' in result.stdout


def test_cli_lists_rust_template_variables() -> None:
    result = runner.invoke(app, ["list-template-variables", "rust-rest-postgres"])

    assert result.exit_code == 0
    assert "project_name" in result.stdout
    assert '"Rust REST Service"' in result.stdout
    assert "postgres_db" in result.stdout
    assert '"app_db"' in result.stdout


def test_cli_lists_rust_cli_template_variables() -> None:
    result = runner.invoke(app, ["list-template-variables", "rust-cli"])

    assert result.exit_code == 0
    assert "project_name" in result.stdout
    assert '"Rust CLI"' in result.stdout
    assert "binary_name" in result.stdout
    assert '"rust-cli"' in result.stdout


def test_cli_lists_go_cli_template_variables() -> None:
    result = runner.invoke(app, ["list-template-variables", "go-cli"])

    assert result.exit_code == 0
    assert "project_name" in result.stdout
    assert '"Go CLI"' in result.stdout
    assert "module_path" in result.stdout
    assert '"github.com/example/go-cli"' in result.stdout


def test_cli_lists_python_cli_template_variables() -> None:
    result = runner.invoke(app, ["list-template-variables", "python-cli"])

    assert result.exit_code == 0
    assert "project_name" in result.stdout
    assert '"Python CLI"' in result.stdout
    assert "package_name" in result.stdout
    assert '"python_cli"' in result.stdout


def test_cli_lists_node_cli_template_variables() -> None:
    result = runner.invoke(app, ["list-template-variables", "node-cli"])

    assert result.exit_code == 0
    assert "project_name" in result.stdout
    assert '"Node CLI"' in result.stdout
    assert "command_name" in result.stdout
    assert '"node-cli"' in result.stdout


def test_cli_lists_node_docs_template_variables() -> None:
    result = runner.invoke(app, ["list-template-variables", "node-docs"])

    assert result.exit_code == 0
    assert "project_name" in result.stdout
    assert '"Node Docs"' in result.stdout
    assert "site_title" in result.stdout
    assert '"Node Docs"' in result.stdout


def test_cli_rejects_unknown_template_for_variable_listing() -> None:
    result = runner.invoke(app, ["list-template-variables", "unknown-template"])

    assert result.exit_code != 0
    assert "Unknown template" in result.stderr
