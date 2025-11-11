use std::collections::HashSet;
use std::path::PathBuf;

use serde_json;

use crate::error::{Error, Result};
use crate::types::{self, ModProject, ModVersion, ProjectId, ProjectSlug, VersionId};

mod labrinth;

#[derive(Default)]
pub struct Client {
    fetched: HashSet<String>,
    mod_db: types::ModDB,
    labrinth_client: labrinth::Client,
}

impl Client {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_cache(mut self, cache_json: PathBuf) -> Result<Self> {
        let text = std::fs::read_to_string(cache_json)?;
        self.mod_db = serde_json::from_str(text.as_str())?;
        Ok(self)
    }

    pub fn write_cache(&self, cache_json: PathBuf) -> Result<()> {
        let text = serde_json::to_string(&self.mod_db)?;
        std::fs::write(cache_json, text)?;
        Ok(())
    }

    pub fn take_db(self) -> types::ModDB {
        self.mod_db
    }

    pub fn get_project_by_id(&self, project_id: &ProjectId) -> Option<&ModProject> {
        self.mod_db.get_project_by_id(project_id)
    }

    pub fn get_project_by_slug(&self, project_slug: &ProjectSlug) -> Option<&ModProject> {
        self.mod_db.get_project_by_slug(project_slug)
    }

    pub fn get_project_version_latest(
        &mut self,
        project_slug: &ProjectSlug,
        game_version: types::MinecraftVersion,
        loader: types::ModLoader,
    ) -> Result<&ModVersion> {
        let (project_id, project_slug) = {
            let project = self.fetch_project_by_slug(project_slug)?;
            (project.project_id.clone(), project.slug.clone())
        };
        let version_id = match self
            .mod_db
            .get_preferred_by_id(&project_id)
            .map(|x| x.version_id.clone())
        {
            Some(x) => x,
            None => {
                let version = self.labrinth_client.get_project_version_latest(
                    project_slug.as_str(),
                    game_version,
                    loader,
                )?;
                let version_id = version.version_id.clone();
                self.mod_db.add_version(version);
                self.mod_db
                    .set_preferred_version(project_id, version_id.clone());
                version_id
            }
        };
        self.fetch_version(&version_id)
    }

    pub fn get_version(&self, version_id: &VersionId) -> Option<&ModVersion> {
        self.mod_db.get_version(version_id)
    }

    pub fn fetch_project_by_id(&mut self, project_id: &ProjectId) -> Result<&ModProject> {
        if !self.fetched.contains(project_id.inner()) {
            let project = self.labrinth_client.get_project(project_id.as_str())?;
            self.fetched.insert(project_id.inner().clone());
            self.fetched.insert(project.slug.inner().clone());
            self.mod_db.add_project(project);
        }
        self.get_project_by_id(project_id)
            .ok_or_else(|| Error::LocalCacheMiss {
                key: project_id.to_string(),
                msg: "Fetch project failed".into(),
            })
    }

    pub fn fetch_project_by_slug(&mut self, project_slug: &ProjectSlug) -> Result<&ModProject> {
        if !self.fetched.contains(project_slug.inner()) {
            let project = self.labrinth_client.get_project(project_slug.as_str())?;
            self.fetched.insert(project.project_id.inner().clone());
            self.fetched.insert(project_slug.inner().clone());
            self.mod_db.add_project(project);
        }
        self.get_project_by_slug(project_slug)
            .ok_or_else(|| Error::LocalCacheMiss {
                key: project_slug.to_string(),
                msg: "Fetch project failed".into(),
            })
    }

    pub fn fetch_version(&mut self, version_id: &VersionId) -> Result<&ModVersion> {
        if !self.fetched.contains(version_id.inner()) {
            let version = self.labrinth_client.get_version(version_id.as_str())?;
            self.fetched.insert(version_id.inner().clone());
            self.mod_db.add_version(version);
        }
        self.get_version(version_id)
            .ok_or_else(|| Error::LocalCacheMiss {
                key: version_id.to_string(),
                msg: "Fetch version failed".into(),
            })
    }
}
