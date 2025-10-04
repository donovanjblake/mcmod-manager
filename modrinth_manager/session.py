"""A wrapper class for a session with the Labrinth API."""

import json

import modrinth_classes as mc
from requests import PreparedRequest, Request, Response, Session


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

    def _request(self, method: str, path: str, params: dict) -> Request:
        url = f"{self.url}{path}"
        params = {k: json.dumps(v) for k, v in params.items()}
        return Request(method, url=url, params=params)

    def _request_project_version(self, project: str, game_version: str, loader: str) -> Request:
        return self._request(
            "GET",
            f"v2/project/{project}/version",
            {
                "loaders": [loader],
                "game_versions": [game_version],
            },
        )

    def _send(self, request: PreparedRequest) -> Response:
        return self.session.send(request)

    def test_connection(self) -> bool:
        """Test the connection to the Labrinth API."""
        return self._send(self._request("GET", "", {}).prepare()).ok

    def get_project_version(
        self,
        project: str,
        game_version: str,
        loader: str,
    ) -> None | mc.ProjectVersion:
        """Get the latest version of a project that supports given game version and loader."""
        request = self._request_project_version(project, game_version, loader)
        response = self._send(request.prepare())
        if not response.ok:
            return None
        versions = [_to_project_version(entry) for entry in response.json()]
        if not versions:
            return None
        return sorted(versions, key=lambda x: x.published)[-1]


def _to_project_version(data: dict) -> mc.ProjectVersion:
    return mc.ProjectVersion(
        name=data["name"],
        id_=data["id"],
        project_id=data["project_id"],
        version=data["version_number"],
        files=[each["url"] for each in data["files"]],
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
