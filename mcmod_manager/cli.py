"""Command line interface for modrinth-based mod manager."""

import argparse
import re
from pathlib import Path

import modrinth_manager as mc


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


def type_game_version(arg: str) -> str:
    """Ensure the passed argument is a valid game version."""
    matched = re.matchall(r"\d+\.\d+\.\d+", arg)
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
    return parser.parse_args()


def load_config(path: None | Path) -> mc.ModrinthConfig:
    """Load the config from the given path of a default path."""
    if path is not None:
        return mc.ModrinthConfig.loads(path.read_text())
    path = Path("./modrinth.toml")
    return mc.ModrinthConfig.loads(path.read_text())


if __name__ == "__main__":
    import argparse

    ARGS = parse_args()
    print(f"{ARGS=}")  # noqa: T201

    with mc.LabrinthSession() as session:
        print("Labrinth session initialized.")  # noqa: T201
        response = session.test_connection()
        print(f"connection test: {response}")  # noqa: T201
        response = session.get_project_version("iris", "1.21.5", "fabric")
        print(f"check iris: {response}")  # noqa: T201
