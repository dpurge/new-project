"""Template models."""

from dataclasses import dataclass


@dataclass(frozen=True)
class Template:
    """Internal template definition."""

    name: str
    description: str
    directory_name: str
