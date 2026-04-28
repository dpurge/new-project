"""Optional user config loading."""

from dataclasses import dataclass
from pathlib import Path

import typer


@dataclass
class Config:
    name: str | None = None
    count: int | None = None


def config_path() -> Path:
    return Path(typer.get_app_dir("{{ cookiecutter.command_name }}")) / "config.env"


def load_config() -> Config:
    path = config_path()
    if not path.exists():
        return Config()

    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()

    return Config(
        name=values.get("name"),
        count=_parse_int(values.get("count")),
    )


def _parse_int(value: str | None) -> int | None:
    if value is None:
        return None

    try:
        return int(value)
    except ValueError:
        return None
