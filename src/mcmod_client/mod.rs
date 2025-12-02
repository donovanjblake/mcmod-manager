use std::collections::HashSet;
use std::path::PathBuf;

use reqwest::blocking as rb;

use crate::types::{self, ModProject, ModVersion, ProjectId, ProjectSlug, VersionId};
use crate::error::{Error, Result};

mod labrinth;

#[derive(Default)]
pub struct ModClient {
    fetched: HashSet<String>,
    mod_db: types::ModDB,
    labrinth_client: labrinth::Client,
}

impl ModClient {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read_cache(&mut self, cache_json: &PathBuf) -> Result<()> {
        let text = std::fs::read_to_string(cache_json)?;
        let temp_db = types::ModDB::from_json(text)?;
        self.mod_db.update(temp_db);
    }

    pub fn write_cache(&self, cache_json: &PathBuf) -> Result<()> {
        let text = serde_json::to_string(&self.mod_db)?;
        std::fs::write(cache_json, text)?;
        Ok(())
    }

    pub fn fetch_project_version_latest(
        &mut self,
        project_slug: &ProjectSlug,
        game_version: types::MinecraftVersion,
        mod_loader: types::ModLoader,
    ) -> Result<VersionId> {
        if let Ok(version) = self.mod_db.find_project_version_latest(project_slug, game_version, mod_loader) {
            return Ok(version.version_id);
        }
        let _project = self.get_project_by_slug(project_slug)?;
        let version = self.labrinth_client.get_project_version_latest(
            project_slug,
            game_version,
            mod_loader,
        )?;
        let version_id = version.version_id;
        self.mod_db.add_version(version);
        Ok(version_id)
    }

    pub fn get_version(&self, version_id: VersionId) -> Option<&ModVersion> {
        self.mod_db.get_version(version_id)
    }

    pub fn fetch_project_by_id(&mut self, project_id: ProjectId) -> Result<&ModProject> {
        if !self.fetched.contains(&project_id.to_string()) {
            let project = self.labrinth_client.get_project(project_id.to_string().as_str())?;
            self.fetched.insert(project_id.to_string());
            self.fetched.insert(project.slug.inner().clone());
            self.mod_db.add_project(project);
        }
        self.mod_db.get_project_by_id(project_id)
            .ok_or_else(|| Error::CacheMissError(project_id.clone().into()))
    }

    pub fn get_project_by_slug(&mut self, project_slug: &ProjectSlug) -> Result<&ModProject> {
        if !self.fetched.contains(project_slug.inner()) {
            let project = self.labrinth_client.get_project(project_slug.as_str())?;
            self.fetched.insert(project.project_id.inner().to_string());
            self.fetched.insert(project_slug.inner().clone());
            self.mod_db.add_project(project);
        }
        self.mod_db.get_project_by_slug(project_slug)
            .ok_or_else(|| Error::CacheMissError(project_slug.clone().into()))
    }

    pub fn fetch_version(&mut self, version_id: &VersionId) -> Result<&ModVersion> {
        if !self.fetched.contains(&version_id.to_string()) {
            let version = self.labrinth_client.get_version(version_id.to_string().as_str())?;
            self.fetched.insert(version_id.inner().to_string());
            self.mod_db.add_version(version);
        }
        self.mod_db.get_version(*version_id)
            .ok_or_else(|| Error::CacheMissError(version_id.clone().into()))
    }

    pub fn validate_enums(&self) -> Result<Vec<crate::error::Error>> {
        Ok(self.labrinth_client.validate_enums()?)
    }
}

pub fn download_file(url: &String) -> Result<Vec<u8>> {
    Ok(rb::get(url)?.bytes().map(|x| x.into())?)
}
