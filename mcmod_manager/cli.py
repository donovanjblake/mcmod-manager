"""Command line interface for modrinth-based mod manager."""

import argparse
import re
from pathlib import Path

import mcmod_manager as mc
from mcmod_manager.mod_config import MCModsConfig, ProjectVersion
from mcmod_manager.result import Err, Ok, Result


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
    return arg


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
        "--download",
        type=type_parent_exists,
        nargs="?",
        help="Whether and where to download the files. Defaults to loader-version.",
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="Development option. Check internal data for inconsistencies with server.",
    )
    return parser.parse_args()


def load_config(path: None | Path) -> MCModsConfig:
    """Load the config from the given path of a default path."""
    if path is not None:
        return MCModsConfig.loads(path.read_text())
    path = Path("./mcmods.toml")
    return MCModsConfig.loads(path.read_text())


def _overprint(x: str) -> None:
    print(f"\x1b[A\x1b[K{x}")  # noqa: T201


def _get_version(
    session: mc.LabrinthSession, project: ProjectVersion, width: int
) -> Result[mc.ModrinthProjectVersion, str]:
    """Try to get the version."""
    prefix = f"  {project.name}: ".ljust(width + len("  : "), ".") + " "
    print(prefix)  # noqa: T201
    result = session.get_project_version(project.name, project.game_version, project.loader)
    return result.inspect_err(lambda x: _overprint(f"{prefix}\x1b[31m{x}\x1b[m")).inspect(
        lambda x: _overprint(f"{prefix}Found {x.name!r}")
    )


def _get_version_dependencies(
    session: mc.LabrinthSession, version: mc.ModrinthProjectVersion
) -> Result[list[mc.ModrinthProjectVersion], str]:
    """Try to get the versions for the dependencies of a project."""
    depends = list(filter(lambda x: x.kind == mc.DependencyKind.REQUIRED, version.dependencies))
    if not depends:
        return Ok([])
    result = []
    for each in depends:
        dep_ver = session.get_project_version(each.project_id, version.game_version, version.loader)
        match dep_ver:
            case Err(x):
                return Err(x)
            case Ok(x):
                result.append(x)
    return Ok(result)


def _get_versions(
    session: mc.LabrinthSession, projects: list[ProjectVersion]
) -> Result[list[mc.ModrinthProjectVersion], str]:
    """Try to get all requested project versions."""
    if not projects:
        return Ok([])
    print("Collecting projects...")  # noqa: T201
    result = []
    maxlen = max([len(x.name) for x in projects])
    for project in projects:
        version = _get_version(session, project, maxlen)
        match version:
            case Err(x):
                return Err(x)
            case Ok(x):
                result.append(x)
    return Ok(result)


def _get_optional_versions(
    session: mc.LabrinthSession, projects: list[ProjectVersion]
) -> Result[list[mc.ModrinthProjectVersion], str]:
    """Try to get all requested project versions."""
    if not projects:
        return Ok([])
    print("Collecting optional projects...")  # noqa: T201
    result = []
    maxlen = max([len(x.name) for x in projects])
    for project in projects:
        version = _get_version(session, project, maxlen).ok()
        if version:
            result.append(version)
    return Ok(result)


def _download(
    session: mc.LabrinthSession, version: mc.ModrinthProjectVersion, folder: Path, width: int
) -> Result[None, str]:
    """Try to download the files for a version."""
    prefix = f"  {version.name}: ".ljust(width + len("  : "), ".") + " "

    def write_all(buffers: list[bytes]) -> None:
        for filelink, buffer in zip(version.files, buffers, strict=True):
            (folder / filelink.filename).write_bytes(buffer)
        _overprint(f"{prefix}downloaded {len(version.files)} file(s)")

    print(prefix)  # noqa: T201
    result = session.download_project_version(version)
    return result.inspect_err(lambda x: _overprint(f"{prefix}download error: {x}")).inspect(
        write_all
    )


def _download_all(
    session: mc.LabrinthSession, versions: mc.ModrinthProjectVersion, folder: Path
) -> Result[None, str]:
    """Try to download all project versions."""
    if not versions:
        return Ok(None)
    print("Downloading files...")  # noqa: T201
    maxlen = max([len(x.name) for x in versions])
    for version in versions:
        result = _download(session, version, folder, maxlen)
        if result.is_err():
            return result
    return Ok(None)


def main() -> None:
    """Entry point for mcmods cli script."""
    args = parse_args()
    print(f"{args=}")  # noqa: T201

    config = load_config(args.config)
    if args.download:
        args.download.mkdir(exist_ok=True, parents=False)
    print("Starting Labrinth session, this may take a bit...")  # noqa: T201
    with mc.LabrinthSession() as session:
        print("Labrinth session started.")  # noqa: T201
        if args.validate:
            session.check_enums().inspect_err(lambda x: print(f"\x1b[33m{x}\x1b[m"))  # noqa: T201
        versions = _get_versions(session, config.projects).expect(
            "Some error finding a required project"
        )
        optional = _get_optional_versions(session, config.optional_projects).ok()
        if optional:
            versions.extend(optional)
        if args.download:
            _download_all(session, versions, args.download).expect(
                "Some error downloading a project file"
            )
