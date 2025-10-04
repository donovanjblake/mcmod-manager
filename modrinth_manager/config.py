"""Modrinth configuration."""

from __future__ import annotations

import tomllib
from dataclasses import dataclass

import modrinth_classes as mc


@dataclass
class ModrinthConfig:
    """Modrinth configuration."""

    game_version: str
    loaders: list[mc.LoaderKind]
    api_url: str
    mods: list[str]

    @staticmethod
    def loads(text: str) -> ModrinthConfig:
        """Load config from a string."""
        data = tomllib.loads(text)
        modpack = data["modpack"]
        return ModrinthConfig(
            game_version=modpack["game_version"],
            loaders=[mc.LoaderKind(each.lower()) for each in modpack["loaders"]],
            api_url=modpack.get("url", "https://api.modrinth.com/"),
            mods=modpack["mods"],
        )
