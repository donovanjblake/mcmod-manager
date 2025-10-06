"""Modrinth configuration."""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from typing import TextIO

from mcmod_manager.mod_classes import LoaderKind


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
    loader: LoaderKind
    game_version: str


def load(fp: TextIO) -> MCModsConfig:
    """Load config from a file."""
    return loads(fp.read())


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
            game_version = None
            loader = None
            if project.get("defaults"):
                game_version = default_game_version
                loader = default_loader
            game_version = project.get("game_version") or game_version
            loader = project.get("loader") or loader
            result.append(
                ProjectVersion(
                    name=name,
                    game_version=game_version,
                    loader=LoaderKind(loader),
                )
            )
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
