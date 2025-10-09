use crate::error::{Error, Result};
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

    fn get<U, P>(&self, url: U, params: &P) -> Result<rb::Response>
    where
        U: reqwest::IntoUrl,
        P: serde::Serialize + ?Sized,
    {
        self.client
            .get(url)
            .form(&params)
            .send()
            .map_err(Error::from)?
            .error_for_status()
            .map_err(Error::from)
    }

    pub fn get_project_version<S, T, U>(
        &self,
        project: S,
        game_version: T,
        loader: U,
    ) -> Result<Version>
    where
        S: std::fmt::Display,
        T: std::fmt::Display,
        U: std::fmt::Display,
    {
        let params = [
            ("game_versions", format!("[\"{game_version}\"]")),
            ("loaders", format!("[\"{loader}\"]")),
        ];
        let response = self.get(
            format!("{LABRINTH_URL}/v2/project/{project}/version"),
            &params,
        )?;
        let url = response.url().as_str().to_owned();
        let versions =
            serde_json::from_str::<Vec<Version>>(response.text().map_err(Error::from)?.as_str())
                .map_err(Error::from)?;
        let version = versions
            .into_iter()
            .max_by(|lhs, rhs| lhs.date_published.cmp(&rhs.date_published))
            .ok_or_else(|| Error::ResponseEmpty { url })?;
        Ok(version)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Version {
    #[serde(rename = "id")]
    version_id: String,
    name: String,
    files: Vec<VersionFile>,
    project_id: String,
    date_published: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct VersionFile {
    url: String,
    filename: String,
}
