use std::collections::HashSet;
use std::path::PathBuf;

use reqwest::blocking as rb;

use crate::error::{Error, Result};
use crate::types::{self, ModDB, ModProject, ModVersion, ProjectId, ProjectSlug, VersionId};

mod labrinth;

/// A client for fetching mods from online or offline databases.
///
/// To use this class online, call fetch_* methods before calling get_* methods.
/// To use this class offline, do not call fetch_* methods.
#[derive(Default)]
pub struct ModClient {
    fetched: HashSet<String>,
    mod_db: ModDB,
    labrinth_client: labrinth::Client,
}

impl ModClient {
    /// Construct a new mod client
    pub fn new() -> Self {
        ModClient::default()
    }

    /// Read the database cache at the given path
    pub fn read_cache(&mut self, cache_json: &PathBuf) -> Result<()> {
        let text = std::fs::read_to_string(cache_json).map_err(|_| Error::ReadPath(cache_json.clone()))?;
        let temp_db = ModDB::from_json(text.as_str())?;
        self.mod_db.extend(temp_db);
        Ok(())
    }

    /// Dump a database cache to the given path
    pub fn write_cache(&self, cache_json: &PathBuf) -> Result<()> {
        let text = serde_json::to_string(&self.mod_db)?;
        let parent = cache_json
            .parent()
            .ok_or_else(|| Error::MissingPath(cache_json.clone()))?;
        if !parent.is_dir() {
            std::fs::create_dir_all(parent).map_err(|_| Error::CreatePath(parent.to_path_buf()))?;
        }
        std::fs::write(cache_json, text).map_err(|_| Error::ReadPath(cache_json.clone()))?;
        Ok(())
    }

    /// Get the internal mod database
    pub fn get_db(&self) -> &ModDB {
        &self.mod_db
    }

    /// Fetch the latest version of a mod, and return its id.
    pub fn fetch_project_version_latest(
        &mut self,
        project_slug: &ProjectSlug,
        game_version: &types::MinecraftVersion,
        mod_loader: types::ModLoader,
    ) -> Result<VersionId> {
        if let Ok(version) =
            self.mod_db
                .find_project_version_latest(project_slug, game_version, mod_loader)
        {
            return Ok(version.version_id);
        }
        let _project = self.fetch_project_by_slug(project_slug)?;
        let version = self.labrinth_client.get_project_version_latest(
            project_slug,
            game_version,
            mod_loader,
        )?;
        let version_id = version.version_id;
        self.mod_db.add_version(version);
        Ok(version_id)
    }

    /// Fetch a project from the online database and return its information.
    pub fn fetch_project_by_id(&mut self, project_id: ProjectId) -> Result<&ModProject> {
        if !self.fetched.contains(&project_id.to_string()) {
            let project = self
                .labrinth_client
                .get_project(project_id.to_string().as_str())?;
            self.fetched.insert(project_id.to_string());
            self.fetched.insert(project.slug.inner().clone());
            self.mod_db.add_project(project);
        }
        self.mod_db
            .get_project_by_id(project_id)
            .ok_or_else(|| Error::CacheMiss(project_id.into()))
    }

    /// Fetch a project from the online database and return its information.
    pub fn fetch_project_by_slug(&mut self, project_slug: &ProjectSlug) -> Result<&ModProject> {
        if !self.fetched.contains(project_slug.inner()) {
            let project = self.labrinth_client.get_project(project_slug.as_str())?;
            self.fetched.insert(project.project_id.inner().to_string());
            self.fetched.insert(project_slug.inner().clone());
            self.mod_db.add_project(project);
        }
        self.mod_db
            .get_project_by_slug(project_slug)
            .ok_or_else(|| Error::CacheMiss(project_slug.clone().into()))
    }

    /// Fetch a project version from the online database and return its information.
    pub fn fetch_version(&mut self, version_id: VersionId) -> Result<&ModVersion> {
        if !self.fetched.contains(&version_id.to_string()) {
            let version = self
                .labrinth_client
                .get_version(version_id.to_string().as_str())?;
            self.fetched.insert(version_id.inner().to_string());
            self.mod_db.add_version(version);
        }
        self.mod_db
            .get_version(version_id)
            .ok_or_else(|| Error::CacheMiss(version_id.into()))
    }

    /// Validate that all online data is supported.
    pub fn validate_structs(&self) -> Result<Vec<crate::error::Error>> {
        self.labrinth_client.validate_structs()
    }
}

pub fn download_file(url: &String) -> Result<Vec<u8>> {
    Ok(rb::get(url)?.bytes().map(Into::into)?)
}
