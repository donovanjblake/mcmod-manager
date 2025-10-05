"""Command line interface for modrinth-based mod manager."""

import argparse
import re
from pathlib import Path

import mcmod_manager as mc
from mcmod_manager.mod_config import MCModsConfig, ProjectVersion


class NotAFileError(Exception):
    """Exception for an entry that is not a file."""

    def __init__(self, path: str) -> None:
        """Initialize an exception for an entry that is not a file."""
        super().__init__(path)


def type_file_exists(arg: str) -> Path:
    """Ensure the argument is an existing file and return its Path."""
    result = Path(arg)
    if not result.exists():
        raise FileNotFoundError(arg)
    if not result.is_file():
        raise NotAFileError(arg)
    return result


def type_parent_exists(arg: str) -> Path:
    """Ensure the argument given is the child of an existing path and return its Path."""
    result = Path(arg)
    if not result.parent.exists():
        raise FileNotFoundError(arg)
    if not result.parent.is_dir():
        raise NotADirectoryError(arg)
    return result


def type_game_version(arg: str) -> str:
    """Ensure the passed argument is a valid game version."""
    matched = re.fullmatch(r"\d+\.\d+\.\d+", arg)
    if matched is None:
        msg = f"Not a game version: {arg!r}"
        raise TypeError(msg)


def type_loader_kind(arg: str) -> mc.LoaderKind:
    """Ensure the passed argument is a valid loader kind."""
    return mc.LoaderKind(arg.lower())


def parse_args() -> argparse.Namespace:
    """Parse the command line arguments."""
    parser = argparse.ArgumentParser(
        description="Command line interface for modrinth-based mod manager."
    )
    parser.add_argument("--config", type=type_file_exists, help="The config file to load.")
    parser.add_argument(
        "--game-version", type=type_game_version, help="The Minecraft version to load."
    )
    parser.add_argument(
        "--loaders",
        type=type_loader_kind,
        nargs="+",
        action="extend",
        help="The Minecraft version to load.",
    )
    parser.add_argument(
        "--download",
        type=type_parent_exists,
        nargs="?",
        help="Whether and where to download the files. Defaults to loader-version."
    )
    return parser.parse_args()


def load_config(path: None | Path) -> MCModsConfig:
    """Load the config from the given path of a default path."""
    if path is not None:
        return MCModsConfig.loads(path.read_text())
    path = Path("./mcmods.toml")
    return MCModsConfig.loads(path.read_text())


def _get_version(session: mc.LabrinthSession, project: ProjectVersion, width: int) -> None | mc.ModrinthProjectVersion:
    """Try to get the version."""
    prefix = f"Find {project.name}: ".ljust(width + len("Find : "), ".") + " "
    result = session.get_project_version(project.name, project.game_version, project.loader)
    return result.inspect_err(lambda x: print(f"{prefix}\x1b[31m{x}\x1b[m")).inspect(lambda x: print(f"{prefix}{x.name}")).ok()


def _download(session: mc.LabrinthSession, version: mc.ModrinthProjectVersion, folder: Path) -> bool:
    """Try to download the files for a version."""
    def write_all(buffers: list[bytes])->None:
        for filelink, buffer in zip(version.files, buffers, strict=True):
            (folder / filelink.filename).write_bytes(buffer)
        print(f"  downloaded {len(version.files)} files")
    result = session.download_project_version(version)
    return result.inspect_err(lambda x: print(f"  download error: {x}")).inspect(write_all).is_ok()


def main() -> None:
    """Entry point for mcmods cli script."""
    args = parse_args()
    print(f"{args=}")  # noqa: T201

    config = load_config(args.config)
    if args.download:
        args.download.mkdir(exist_ok=True, parents=False)
    with mc.LabrinthSession() as session:
        print("Labrinth session initialized.")  # noqa: T201
        width = max(len(x.name) for x in config.projects)
        for project in config.projects:
            version = _get_version(session, project, width)
            if version is None:
                continue
            if args.download:
                _download(session, version, args.download)
