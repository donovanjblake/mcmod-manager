"""A wrapper class for a session with the Labrinth API."""

from requests import Response, Session

from mcmod_manager import mod_classes as mc
from mcmod_manager.result import Err, Ok, Result


class LabrinthError(Exception):
    """Base exception for a Labrinth API error."""


class LabrinthSession:
    """A wrapper class for a session with the Labrinth API."""

    def __init__(self, url: None | str = None) -> None:
        """Initialize a session with the Labrinth API."""
        self.session = Session()
        self.url = url or "https://api.modrinth.com/"
        response = self.session.get(self.url)
        if not response:
            msg = f"Bad response: {response.status_code}"
            raise LabrinthError(msg)

    def __enter__(self) -> None:
        """Enter context for this session."""
        return self

    def __exit__(self, exception_kind: object, exception: object, traceback: object) -> None:
        """Exit the context for this session."""
        self.session.close()

    def _get(self, path: None | str = None, params: None | dict[str, str] = None) -> Response:
        url = self.url + ("" if path is None else path)
        return self.session.get(url=url, params=params)

    def _get_project_version(
        self, project: str, game_version: str, loader: mc.LoaderKind
    ) -> Response:
        return self._get(
            f"v2/project/{project}/version",
            {
                "loaders": f'["{loader.value}"]',
                "game_versions": f'["{game_version}"]',
            },
        )

    def test_connection(self) -> bool:
        """Test the connection to the Labrinth API."""
        return self._get().ok

    def check_enums(self) -> Result[None, str]:
        """Check for validity of the internal enumerations."""

        def check_enum(path: str, enum_kind: type) -> Result[None, str]:
            response = self._get(path)
            if not response:
                return Err(_response_str(response))
            enums = {each.value for each in enum_kind}
            names = {each["name"] for each in response.json()}
            errs = []
            only = enums.difference(names)
            if only:
                errs.append(f"Extra enumerators: {only!r}")
            only = names.difference(enums)
            if only:
                errs.append(f"Missing enumerators: {only!r}")
            if errs:
                return Err(", ".join(errs))
            return Ok(None)

        errs = list(
            filter(
                None,
                [
                    check_enum("v2/tag/loader", mc.LoaderKind).err(),
                ],
            )
        )
        if errs:
            return Err(", ".join(errs))
        return Ok(None)

    def get_project_version(
        self,
        project: str,
        game_version: str,
        loader: mc.LoaderKind,
    ) -> Result[mc.ModrinthProjectVersion, str]:
        """Get the latest version of a project that supports given game version and loader."""
        response = self._get_project_version(project, game_version, loader)
        if not response:
            x_game_version = game_version.rsplit(".", 1) + ".x"
            response = self._get_project_version(project, x_game_version, loader)
        if not response:
            return Err(_response_str(response))
        versions = [_to_project_version(entry) for entry in response.json()]
        if not versions:
            return Err("No versions found matching the given filters.")
        return Ok(sorted(versions, key=lambda x: x.published)[-1])

    def download_project_version(
        self, version: mc.ModrinthProjectVersion
    ) -> Result[list[bytes], str]:
        """Download the files for a project version."""
        result = []
        for filelink in version.files:
            response = self.session.get(filelink.url)
            if not response:
                return Err(f"{filelink}: {_response_str(response)}")
            data = bytes(response.text, encoding="utf-8")
            if not data:
                return Err(f"{filelink}: Downloaded file is empty.")
            result.append(data)
        return Ok(result)


def _to_project_version(data: dict) -> mc.ModrinthProjectVersion:
    return mc.ModrinthProjectVersion(
        name=data["name"],
        id_=data["id"],
        project_id=data["project_id"],
        version=data["version_number"],
        files=[_to_file_link(each) for each in data["files"]],
        game_versions=data["game_versions"],
        loaders=[mc.LoaderKind(each.lower()) for each in data["loaders"]],
        published=data["date_published"],
        dependencies=[_to_version_dependency(each) for each in data["dependencies"]],
    )


def _to_version_dependency(data: dict) -> mc.VersionDependency:
    return mc.VersionDependency(
        version_id=data["version_id"],
        project_id=data["project_id"],
        file_name=data["file_name"],
        kind=mc.DependencyKind(data["dependency_type"].lower()),
    )


def _to_file_link(data: dict) -> mc.FileLink:
    return mc.FileLink(url=data["url"], filename=data["filename"])


def _response_str(response: Response) -> str:
    return f"Response(status_code={response.status_code!r}, text={response.text!r})"
