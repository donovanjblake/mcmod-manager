use crate::error::{Error, Result};
use crate::types::{self, ModLoader};
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
        self.client
            .get(url)
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)
    }

    fn get_form<U, P>(&self, url: U, params: &P) -> Result<rb::Response>
    where
        U: reqwest::IntoUrl + Clone,
        P: serde::Serialize + ?Sized,
    {
        self.client
            .get(url)
            .query(&params)
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)
    }

    /// Get the latest version of a project for the target Minecraft version and mod loader
    pub fn get_project_version<S, T>(
        &self,
        project: S,
        game_version: T,
        loader: types::ModLoader,
    ) -> Result<ProjectVersion>
    where
        S: std::fmt::Display,
        T: std::fmt::Display,
    {
        let params = [
            ("game_versions", format!("[\"{game_version}\"]")),
            ("loaders", format!("[\"{}\"]", loader.to_string())),
        ];
        let response = self.get_form(
            format!("{LABRINTH_URL}/v2/project/{project}/version"),
            &params,
        )?;
        let url = response.url().as_str().to_owned();
        let versions = serde_json::from_str::<Vec<ProjectVersion>>(
            response.text().map_err(Error::from)?.as_str(),
        )
        .map_err(Error::from)?;
        let version = versions
            .into_iter()
            .max_by(|lhs, rhs| lhs.date_published.cmp(&rhs.date_published))
            .ok_or_else(|| Error::ResponseEmpty { url })?;
        Ok(version)
    }

    /// Download a single file
    pub fn download_file(&self, version_file: &VersionFile) -> Result<Vec<u8>> {
        self.get(version_file.url.clone())?
            .bytes()
            .map(|x| x.into())
            .map_err(Error::from)
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
    pub game_versions: Vec<String>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct VersionFile {
    pub url: String,
    pub filename: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_project_version() {
        let client = Client::new();
        let game_version = "1.21.2".to_string();
        let loader = ModLoader::Minecraft;
        let version = client
            .get_project_version("faithful-32x", &game_version, loader)
            .expect("Client should get a project version");
        if !version.game_versions.contains(&game_version) || !version.loaders.contains(&loader) {
            panic!("Client should get the latest project version for a specific target {version:?}")
        }
    }

    #[test]
    fn test_download_files() {
        let client = Client::new();
        let game_version = "1.21.2".to_string();
        let loader = ModLoader::Fabric;
        let version = client
            .get_project_version("iris", &game_version, loader)
            .expect("Client should get a project version");
        if !version.game_versions.contains(&game_version) || !version.loaders.contains(&loader) {
            panic!("Client should get the latest project version for a specific target {version:?}")
        }
        let _files = client
            .download_version_files(&version)
            .expect("Client should be able to download files");
    }
}
