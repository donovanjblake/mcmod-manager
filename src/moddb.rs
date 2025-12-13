use crate::error::{Error, Result};
use crate::types::{
    MinecraftVersion, ModLink, ModLoader, ModProject, ModVersion, ProjectId, ProjectSlug, VersionId,
};
use std::collections::HashMap;

/// An internal database of the projects and versions collected
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ModDB {
    /// A mapping of project ids to project data
    projects: HashMap<ProjectId, ModProject>,
    /// A mapping of version ids to version data
    versions: HashMap<VersionId, ModVersion>,
    /// A mapping of project slugs to project ids
    project_slugs: HashMap<ProjectSlug, ProjectId>,
}

impl ModDB {
    /// Create a database from a saved json
    pub fn from_json(json_str: &str) -> Result<Self> {
        let mod_db = serde_json::from_str(json_str)?;
        Ok(mod_db)
    }

    /// Update this ModDB by taking values from another ModDB
    pub fn extend(&mut self, other: ModDB) {
        self.projects.extend(other.projects);
        self.project_slugs.extend(other.project_slugs);
        self.versions.extend(other.versions);
    }

    /// Insert a project into the database, and return the previous project at the same project_id.
    pub fn add_project(&mut self, project: ModProject) -> Option<ModProject> {
        self.project_slugs
            .insert(project.slug.clone(), project.project_id);
        self.projects.insert(project.project_id, project)
    }

    /// Insert a version into the database, and return the previous version at the same version_id.
    pub fn add_version(&mut self, version: ModVersion) -> Option<ModVersion> {
        self.versions.insert(version.version_id, version)
    }

    /// Check if the given id exists in this database.
    pub fn contains_key(&self, mod_link: &ModLink) -> bool {
        match mod_link {
            ModLink::ProjectId(x) => self.projects.contains_key(x),
            ModLink::ProjectSlug(x) => self.project_slugs.contains_key(x),
            ModLink::VersionId(x) => self.versions.contains_key(x),
        }
    }

    /// Remove the given id from this database.
    pub fn remove(&mut self, mod_link: &ModLink) {
        match mod_link {
            ModLink::ProjectId(x) => {
                self.projects.remove(x);
            }
            ModLink::ProjectSlug(x) => {
                self.project_slugs.remove(x);
            }
            ModLink::VersionId(x) => {
                self.versions.remove(x);
            }
        }
    }

    /// Get a vector of all collected versions
    pub fn get_versions(&self) -> Vec<&ModVersion> {
        self.versions.values().collect()
    }

    /// Get the project of a given id
    pub fn get_project_by_id(&self, project_id: ProjectId) -> Option<&ModProject> {
        self.projects.get(&project_id)
    }

    /// Get the project of a given slug
    pub fn get_project_by_slug(&self, project_slug: &ProjectSlug) -> Option<&ModProject> {
        self.projects.get(self.project_slugs.get(project_slug)?)
    }

    /// Get the version of a given id
    pub fn get_version(&self, version_id: VersionId) -> Option<&ModVersion> {
        self.versions.get(&version_id)
    }

    /// Find the latest version of a project matching the given requirements and set it
    pub fn find_project_version_latest(
        &self,
        project_slug: &ProjectSlug,
        game_version: MinecraftVersion,
        mod_loader: ModLoader,
    ) -> Result<&ModVersion> {
        let project = self
            .get_project_by_slug(project_slug)
            .ok_or_else(|| Error::CacheMiss(project_slug.clone().into()))?;
        let mut latest: Option<(VersionId, chrono::NaiveDateTime)> = None;
        for version_id in &project.version_ids {
            let Some(version) = self.get_version(*version_id) else {
                continue;
            };
            if !version.game_versions.contains(&game_version)
                || !version.loaders.contains(&mod_loader)
            {
                continue;
            }
            if latest.is_none_or(|x| x.1 < version.date_published) {
                latest = Some((version.version_id, version.date_published))
            }
        }
        let latest_id = latest
            .ok_or_else(|| Error::NoMatchingVersion {
                project_slug: project_slug.clone(),
                game_version,
                mod_loader,
            })?
            .0;
        Ok(self
            .versions
            .get(&latest_id)
            .expect("The version was just added why does it not exist this is dumb."))
    }
}
