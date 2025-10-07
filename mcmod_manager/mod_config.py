"""Modrinth configuration."""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from typing import TextIO

from mcmod_manager.mod_classes import LoaderKind
from mcmod_manager.result import Err, Ok, Result


@dataclass
class MCModsConfig:
    """Modrinth configuration."""

    game_version: str
    loader: LoaderKind
    api_url: str
    projects: list[ProjectVersion]
    optional_projects: list[ProjectVersion]

    @staticmethod
    def loads(text: str) -> MCModsConfig:
        """Load config from a string."""
        return loads(text)


@dataclass
class ProjectVersion:
    """Version information for a project in a config."""

    name: str
    loader: LoaderKind | None
    game_version: str | None


def load(fp: TextIO) -> MCModsConfig:
    """Load config from a file."""
    return loads(fp.read())


def _load_project(name: str, data: dict[str, str]) -> Result[ProjectVersion, str]:
    game_version = data.get("game_version")
    loader = data.get("loader")
    if not data.get("defaults"):
        if game_version is None:
            return Err(f"{name}: Expected game_version")
        if loader is None:
            return Err(f"{name}: Expected loader")
    if loader is not None:
        loader = LoaderKind(loader)
    return Ok(
        ProjectVersion(
            name=name,
            game_version=game_version,
            loader=loader,
        )
    )


def loads(text: str) -> MCModsConfig:
    """Load config from a string."""
    data = tomllib.loads(text)

    defaults = data.get("defaults", {})
    default_game_version = defaults.get("game_version")
    default_loader = defaults.get("loader")
    if default_loader is not None:
        default_loader = LoaderKind(default_loader)
    default_url = defaults.get("url", "https://api.modrinth.com/")

    def load_projects(projects: dict) -> list[ProjectVersion]:
        result = []
        for name, project in projects.items():
            result.append(_load_project(name, project).expect("Bad project item"))
        return result

    projects = load_projects(data.get("projects", {}))
    optional_projects = load_projects(data.get("optional-projects", {}))

    return MCModsConfig(
        game_version=default_game_version,
        loader=default_loader,
        api_url=default_url,
        projects=projects,
        optional_projects=optional_projects,
    )
