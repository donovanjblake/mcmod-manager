use crate::error::{Error, Result};
use crate::types::{self, MinecraftVersion, ModLoader};
use reqwest::blocking as rb;

const LABRINTH_URL: &str = "https://api.modrinth.com";

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
        U: reqwest::IntoUrl + Clone,
        P: serde::Serialize + ?Sized,
    {
        Ok(self
            .client
            .get(url)
            .query(&params)
            .send()?
            .error_for_status()?)
    }

    /// Get the latest version of a project for the target Minecraft version and mod loader
    pub fn get_project_version<S>(
        &self,
        project: S,
        game_version: MinecraftVersion,
        loader: types::ModLoader,
    ) -> Result<ProjectVersion>
    where
        S: std::fmt::Display,
    {
        let params = [
            ("game_versions", format!("[\"{game_version}\"]")),
            ("loaders", format!("[\"{loader}\"]")),
        ];
        let response = self.get_form(
            format!("{LABRINTH_URL}/v2/project/{project}/version"),
            &params,
        )?;
        let url = response.url().as_str().to_owned();
        let versions = serde_json::from_str::<Vec<ProjectVersion>>(response.text()?.as_str())?;
        let version = versions
            .into_iter()
            .max_by(|lhs, rhs| lhs.date_published.cmp(&rhs.date_published))
            .ok_or_else(|| Error::ResponseEmpty { url })?;
        Ok(version)
    }

    /// Get all the depencencies of the given project version
    pub fn get_version_dependencies(
        &self,
        version: &ProjectVersion,
        game_version: MinecraftVersion,
        loader: types::ModLoader,
    ) -> Result<Vec<ProjectVersion>> {
        let mut result = Vec::<ProjectVersion>::new();
        for dependency in &version.dependencies {
            if dependency.kind != DependencyKind::Required {
                continue;
            }
            if let Some(version_id) = &dependency.version_id {
                let response = self.get(format!("{LABRINTH_URL}/v2/version/{version_id}"))?;
                let version = serde_json::from_str::<ProjectVersion>(response.text()?.as_str())?;
                result.push(version);
            } else if let Some(project_id) = &dependency.project_id {
                let version = self.get_project_version(project_id, game_version, loader)?;
                result.push(version);
            } else {
                todo!("Give some kind of message for filename kinds")
            }
        }
        Ok(result)
    }

    /// Download a single file
    pub fn download_file(&self, version_file: &VersionFile) -> Result<Vec<u8>> {
        Ok(self
            .get(version_file.url.clone())?
            .bytes()
            .map(|x| x.into())?)
    }

    /// Download the files of a version into a list of tuples of the file info and the bytes
    pub fn download_version_files<'pv>(
        &self,
        version: &'pv ProjectVersion,
    ) -> Result<Vec<(&'pv VersionFile, Vec<u8>)>> {
        let mut result = Vec::<(&'pv VersionFile, Vec<u8>)>::new();
        for version_file in &version.files {
            result.push((version_file, self.download_file(version_file)?))
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

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct ProjectVersion {
    // #[serde(rename = "id")]
    // pub version_id: String,
    // pub project_id: String,
    pub name: String,
    pub files: Vec<VersionFile>,
    pub date_published: String,
    pub loaders: Vec<ModLoader>,
    pub game_versions: Vec<MinecraftVersion>,
    pub dependencies: Vec<VersionDependency>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct VersionFile {
    pub url: String,
    pub filename: String,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct VersionDependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub filename: Option<String>,
    #[serde(rename = "dependency_type")]
    pub kind: DependencyKind,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyKind {
    Required,
    Optional,
    Incompatible,
    Embedded,
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
            .get_project_version("faithful-32x", game_version, loader)
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
            .get_project_version("iris", game_version, loader)
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
