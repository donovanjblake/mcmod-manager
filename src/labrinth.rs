use crate::error::{Error, Result};
use crate::types::{self, MinecraftVersion, ModLoader};
use reqwest::blocking as rb;

const LABRINTH_URL: &str = "https://api.modrinth.com";

#[derive(Default)]
pub struct Client {
    client: rb::Client,
}

impl Client {
    pub fn new() -> Self {
        Self {
            client: rb::Client::new(),
        }
    }

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
        let response = self.get(format!("{LABRINTH_URL}/v2/project/{project}"))?;
        let project = serde_json::from_str::<Project>(response.text()?.as_str())?;
        Ok(project.into())
    }

    /// Get a version from the database
    pub fn get_version(&self, version: &str) -> Result<types::ModVersion> {
        let response = self.get(format!("{LABRINTH_URL}/v2/version/{version}"))?;
        let version = serde_json::from_str::<Version>(response.text()?.as_str())?;
        Ok(version.into())
    }

    /// Get the project versions matching the given query
    pub fn get_project_versions(
        &self,
        project: &str,
        game_versions: &[MinecraftVersion],
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
            format!("{LABRINTH_URL}/v2/project/{project}/version"),
            &params,
        )?;
        let versions = serde_json::from_str::<Vec<Version>>(response.text()?.as_str())?;
        Ok(versions.into_iter().map(Version::into).collect())
    }

    /// Get the latest version of a project for the target Minecraft version and mod loader
    pub fn get_project_version_latest(
        &self,
        project: &str,
        game_version: MinecraftVersion,
        loader: types::ModLoader,
    ) -> Result<types::ModVersion> {
        self.get_project_versions(project, &[game_version], &[loader])?
            .into_iter()
            .max_by(|x, y| x.date_published.cmp(&y.date_published))
            .ok_or_else(|| Error::VersionNotFound {
                project: project.to_string(),
            })
    }

    /// Download a single file
    pub fn download_file(&self, file_url: &str) -> Result<Vec<u8>> {
        Ok(self.get(file_url)?.bytes().map(|x| x.into())?)
    }

    /// Download the files of a version into a list of tuples of the file info and the bytes
    #[cfg(test)]
    pub fn download_version_files<'a>(
        &self,
        version: &'a types::ModVersion,
    ) -> Result<Vec<(&'a types::ModFile, Vec<u8>)>> {
        let mut result = Vec::<(&'a types::ModFile, Vec<u8>)>::new();
        for version_file in &version.files {
            result.push((version_file, self.download_file(&version_file.url)?))
        }
        Ok(result)
    }

    /// Validate all internal enumerations are up to date
    pub fn validate_enums(&self) -> Result<Vec<Error>> {
        let mut result = Vec::<Error>::new();
        let repsonse = self.get(format!("{LABRINTH_URL}/v2/tag/loader"))?;
        let values = serde_json::from_str::<Vec<LoaderInfo>>(repsonse.text()?.as_str())?;
        for v in values {
            if let Err(e) = ModLoader::try_from(v.name.as_str()) {
                result.push(e)
            }
        }
        Ok(result)
    }
}

#[derive(serde::Deserialize)]
struct Project {
    pub slug: String,
    pub title: String,
    #[serde(rename = "id")]
    pub project_id: String,
    // #[serde(rename = "versions")]
    // pub version_ids: Vec<String>,
    // pub game_versions: Vec<MinecraftVersion>,
    pub loaders: Vec<ModLoader>,
}

impl From<Project> for types::ModProject {
    fn from(value: Project) -> Self {
        Self {
            project_id: value.project_id.into(),
            name: value.title,
            slug: value.slug.into(),
            // version_ids: value.version_ids.into_iter().map(|x| x.into()).collect(),
            // game_versions: value.game_versions,
            loaders: value.loaders,
        }
    }
}

#[derive(serde::Deserialize)]
struct Version {
    pub name: String,
    #[serde(rename = "id")]
    pub version_id: String,
    pub project_id: String,
    pub dependencies: Vec<Dependency>,
    #[cfg(test)]
    pub game_versions: Vec<MinecraftVersion>,
    pub date_published: DatePublished,
    pub loaders: Vec<ModLoader>,
    pub files: Vec<FileLink>,
}

impl From<Version> for types::ModVersion {
    fn from(value: Version) -> Self {
        Self {
            project_id: value.project_id.into(),
            version_id: value.version_id.into(),
            name: value.name,
            #[cfg(test)]
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
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub dependency_type: DependencyKind,
}

impl Dependency {
    fn into_link(self) -> Option<types::ModLink> {
        if !matches!(self.dependency_type, DependencyKind::Required) {
            return None;
        }
        #[allow(clippy::manual_map)]
        if let Some(version_id) = self.version_id {
            Some(types::ModLink::VersionId(version_id.into()))
        } else if let Some(project_id) = self.project_id {
            Some(types::ModLink::ProjectId(project_id.into()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_project_version() {
        let client = Client::new();
        let game_version = MinecraftVersion::from("1.21.2");
        let loader = ModLoader::Minecraft;
        let version = client
            .get_project_version_latest("faithful-32x", game_version, loader)
            .expect("Client should get a project version");
        if !version.game_versions.contains(&game_version) || !version.loaders.contains(&loader) {
            panic!("Client should get the latest project version for a specific target {version:?}")
        }
    }

    #[test]
    fn test_download_files() {
        let client = Client::new();
        let game_version = MinecraftVersion::from("1.21.2");
        let loader = ModLoader::Fabric;
        let version = client
            .get_project_version_latest("iris", game_version, loader)
            .expect("Client should get a project version");
        if !version.game_versions.contains(&game_version) || !version.loaders.contains(&loader) {
            panic!("Client should get the latest project version for a specific target {version:?}")
        }
        let _files = client
            .download_version_files(&version)
            .expect("Client should be able to download files");
    }

    #[test]
    fn test_validate_data() {
        let client = Client::new();
        client
            .validate_enums()
            .expect("Client shall be able to get and compare data");
    }
}
