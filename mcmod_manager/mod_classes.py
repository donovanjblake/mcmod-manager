"""Dataclasses for Modrinth objects."""

from __future__ import annotations

import enum
from dataclasses import dataclass


class LoaderKind(enum.StrEnum):
    """Enumeration of mod loaders."""

    BABRIC = enum.auto()
    BTA_BABRIC = "bta-babric"
    BUKKIT = enum.auto()
    BUNGEECORD = enum.auto()
    CANVAS = enum.auto()
    DATAPACK = enum.auto()
    FABRIC = enum.auto()
    FOLIA = enum.auto()
    FORGE = enum.auto()
    IRIS = enum.auto()
    JAVA_AGENT = "java-agent"
    LEGACY_FABRIC = "legacy-fabric"
    LITELOADER = enum.auto()
    MINECRAFT = enum.auto()
    MODLOADER = enum.auto()
    NEOFORGE = enum.auto()
    NILLOADER = enum.auto()
    OPTIFINE = enum.auto()
    ORNITHE = enum.auto()
    PAPER = enum.auto()
    PURPUR = enum.auto()
    QUILT = enum.auto()
    RIFT = enum.auto()
    SPIGOT = enum.auto()
    SPONGE = enum.auto()
    VANILLA = enum.auto()
    VELOCITY = enum.auto()
    WATERFALL = enum.auto()



@dataclass
class ModrinthProject:
    """Modrinth project information."""

    name: str
    slug: str
    id_: str
    loaders: list[LoaderKind]
    game_versions: list[str]
    versions: list[str]


@dataclass
class ModrinthProjectVersion:
    """Modrinth project version information."""

    name: str
    id_: str
    project_id: str
    loaders: list[LoaderKind]
    game_versions: list[str]
    version: str
    files: list[FileLink]
    published: str
    dependencies: list[VersionDependency]


@dataclass
class FileLink:
    """Information for downloading a file."""

    url: str
    filename: str


@dataclass
class VersionDependency:
    """Dependencies for a project version."""

    version_id: str
    project_id: str
    file_name: str
    kind: DependencyKind


class DependencyKind(enum.StrEnum):
    """Kinds of dependency links."""

    REQUIRED = enum.auto()
    OPTIONAL = enum.auto()
    INCOMPATIBLE = enum.auto()
    EMBEDDED = enum.auto()
