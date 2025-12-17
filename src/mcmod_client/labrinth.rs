use crate::error::{Error, Result};
use crate::types::{self, MinecraftVersion, ModLoader, ProjectId, ProjectSlug, VersionId};
use reqwest::blocking as rb;

const API_MODRINTH: &str = "https://api.modrinth.com";

#[derive(Default)]
pub struct Client {
    client: rb::Client,
}

impl Client {
    fn get<U>(&self, url: U) -> Result<rb::Response>
    where
        U: reqwest::IntoUrl,
    {
        Ok(self.client.get(url).send()?.error_for_status()?)
    }

    fn get_form<U, P>(&self, url: U, params: &P) -> Result<rb::Response>
    where
        U: reqwest::IntoUrl,
        P: serde::Serialize + ?Sized,
    {
        Ok(self
            .client
            .get(url)
            .query(&params)
            .send()?
            .error_for_status()?)
    }

    /// Get a project from the database
    pub fn get_project(&self, project: &str) -> Result<types::ModProject> {
        let response = self.get(format!("{API_MODRINTH}/v2/project/{project}"))?;
        let mut project = serde_json::from_str::<Project>(response.text()?.as_str())?;
        project
            .game_versions
            .retain(|x| !matches!(x, MinecraftVersion::Unknown { version: _ }));
        Ok(project.into())
    }

    /// Get a version from the database
    pub fn get_version(&self, version: &str) -> Result<types::ModVersion> {
        let response = self.get(format!("{API_MODRINTH}/v2/version/{version}"))?;
        let version = serde_json::from_str::<Version>(response.text()?.as_str())?;
        Ok(version.into())
    }

    /// Get the project versions matching the given query
    pub fn get_project_versions(
        &self,
        project: &ProjectSlug,
        game_versions: &[&MinecraftVersion],
        loaders: &[types::ModLoader],
    ) -> Result<Vec<types::ModVersion>> {
        let game_versions = game_versions
            .iter()
            .map(|x| format!("\"{x}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let loaders = loaders
            .iter()
            .map(|x| format!("\"{x}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let params = [
            ("game_versions", format!("[{game_versions}]")),
            ("loaders", format!("[{loaders}]")),
        ];
        let response = self.get_form(
            format!("{API_MODRINTH}/v2/project/{project}/version"),
            &params,
        )?;
        let versions = serde_json::from_str::<Vec<Version>>(response.text()?.as_str())?;
        Ok(versions.into_iter().map(Version::into).collect())
    }

    /// Get the latest version of a project for the target Minecraft version and mod loader
    pub fn get_project_version_latest(
        &self,
        project_slug: &ProjectSlug,
        game_version: &MinecraftVersion,
        mod_loader: types::ModLoader,
    ) -> Result<types::ModVersion> {
        self.get_project_versions(project_slug, &[game_version], &[mod_loader])?
            .into_iter()
            .max_by(|x, y| x.date_published.cmp(&y.date_published))
            .ok_or_else(|| Error::NoMatchingVersion {
                project_slug: project_slug.clone(),
                game_version: game_version.clone(),
                mod_loader,
            })
    }

    /// Download a single file
    #[cfg(test)]
    pub fn download_file(&self, file_url: &str) -> Result<Vec<u8>> {
        Ok(self.get(file_url)?.bytes().map(Into::into)?)
    }

    /// Download the files of a version into a list of tuples of the file info and the bytes
    #[cfg(test)]
    pub fn download_version_files<'a>(
        &self,
        version: &'a types::ModVersion,
    ) -> Result<Vec<(&'a types::ModFile, Vec<u8>)>> {
        let mut result = Vec::<(&'a types::ModFile, Vec<u8>)>::new();
        for version_file in &version.files {
            result.push((version_file, self.download_file(&version_file.url)?));
        }
        Ok(result)
    }

    fn validate_loaders(&self) -> Result<Vec<crate::error::Error>> {
        let mut result = Vec::<crate::error::Error>::new();
        let repsonse = self.get(format!("{API_MODRINTH}/v2/tag/loader"))?;
        let values = serde_json::from_str::<Vec<LoaderInfo>>(repsonse.text()?.as_str())?;
        for v in values {
            if let Err(e) = ModLoader::try_from(v.name.as_str()) {
                result.push(e);
            }
        }
        Ok(result)
    }

    fn validate_game_versions(&self) -> Result<Vec<crate::error::Error>> {
        let mut result = Vec::<crate::error::Error>::new();
        let repsonse = self.get(format!("{API_MODRINTH}/v2/tag/game_version"))?;
        let values = serde_json::from_str::<Vec<GameVersionInfo>>(repsonse.text()?.as_str())?;
        for v in values {
            if let Err(e) = MinecraftVersion::try_parse_from(&v.version) {
                result.push(e);
            }
        }
        Ok(result)
    }

    /// Validate all internal structs are up to date
    pub fn validate_structs(&self) -> Result<Vec<crate::error::Error>> {
        let mut result = self.validate_loaders()?;
        result.extend(self.validate_game_versions()?);
        Ok(result)
    }
}

#[derive(serde::Deserialize)]
struct Project {
    pub slug: ProjectSlug,
    pub title: String,
    #[allow(clippy::struct_field_names)]
    #[serde(rename = "id")]
    pub project_id: ProjectId,
    #[serde(rename = "versions")]
    pub version_ids: Vec<VersionId>,
    pub game_versions: Vec<MinecraftVersion>,
    pub loaders: Vec<ModLoader>,
}

impl From<Project> for types::ModProject {
    fn from(value: Project) -> Self {
        Self {
            project_id: value.project_id,
            name: value.title,
            slug: value.slug,
            version_ids: value.version_ids,
            game_versions: value.game_versions,
            loaders: value.loaders,
        }
    }
}

#[derive(serde::Deserialize)]
struct Version {
    pub name: String,
    #[allow(clippy::struct_field_names)]
    #[serde(rename = "id")]
    pub version_id: VersionId,
    pub project_id: ProjectId,
    pub dependencies: Vec<Dependency>,
    pub game_versions: Vec<MinecraftVersion>,
    pub date_published: DatePublished,
    pub loaders: Vec<ModLoader>,
    pub files: Vec<FileLink>,
}

impl From<Version> for types::ModVersion {
    fn from(value: Version) -> Self {
        Self {
            project_id: value.project_id,
            version_id: value.version_id,
            name: value.name,
            game_versions: value.game_versions,
            loaders: value.loaders,
            dependencies: value
                .dependencies
                .into_iter()
                .filter_map(Dependency::into_link)
                .collect(),
            files: value.files.into_iter().map(FileLink::into).collect(),
            date_published: value.date_published.0,
        }
    }
}

#[derive(serde::Deserialize)]
struct Dependency {
    pub version_id: Option<VersionId>,
    pub project_id: Option<ProjectId>,
    #[serde(rename = "dependency_type")]
    pub kind: DependencyKind,
}

impl Dependency {
    fn into_link(self) -> Option<types::ModLink> {
        if !matches!(self.kind, DependencyKind::Required) {
            return None;
        }
        #[allow(clippy::manual_map)]
        if let Some(version_id) = self.version_id {
            Some(version_id.into())
        } else if let Some(project_id) = self.project_id {
            Some(project_id.into())
        } else {
            None
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum DependencyKind {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(serde::Deserialize)]
struct FileLink {
    pub url: String,
    pub filename: String,
}

impl From<FileLink> for types::ModFile {
    fn from(value: FileLink) -> Self {
        Self {
            url: value.url,
            name: value.filename,
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(try_from = "String")]
struct DatePublished(chrono::NaiveDateTime);

impl TryFrom<String> for DatePublished {
    type Error = Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            chrono::DateTime::parse_from_rfc3339(value.as_str())?.naive_utc(),
        ))
    }
}

#[derive(serde::Deserialize, Debug)]
struct LoaderInfo {
    pub name: String,
}

#[derive(serde::Deserialize, Debug)]
struct GameVersionInfo {
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_project() {
        let client = Client::default();
        let game_version = MinecraftVersion::from("1.21.2");
        let loader = ModLoader::Minecraft;
        let version = client
            .get_project("faithful-32x")
            .expect("Client should get a project");
        assert!(
            version.game_versions.contains(&game_version) && version.loaders.contains(&loader),
            "Client should get the latest project version for a specific target {version:?}"
        );
    }

    #[test]
    fn test_get_project_version() {
        let client = Client::default();
        let game_version = MinecraftVersion::from("1.21.2");
        let loader = ModLoader::Minecraft;
        let version = client
            .get_project_version_latest(&ProjectSlug::from("faithful-32x"), &game_version, loader)
            .expect("Client should get a project version");
        assert!(
            version.game_versions.contains(&game_version) && version.loaders.contains(&loader),
            "Client should get the latest project version for a specific target {version:?}"
        );
    }

    #[test]
    fn test_download_files() {
        let client = Client::default();
        let game_version = MinecraftVersion::from("1.21.2");
        let loader = ModLoader::Fabric;
        let version = client
            .get_project_version_latest(&ProjectSlug::from("iris"), &game_version, loader)
            .expect("Client should get a project version");

        assert!(
            version.game_versions.contains(&game_version) && version.loaders.contains(&loader),
            "Client should get the latest project version for a specific target {version:?}"
        );
        let _files = client
            .download_version_files(&version)
            .expect("Client should be able to download files");
    }

    #[test]
    fn test_validate_data() {
        let client = Client::default();
        client
            .validate_structs()
            .expect("Client shall be able to get and compare data");
    }
}
