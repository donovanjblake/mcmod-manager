use crate::config;
use crate::error::{Error, Result};
use crate::labrinth;
use crate::types::{self, ModLink, ModLoader, ProjectId, ProjectSlug, VersionId};

/// Collects all mods and their dependencies according to the config
pub struct ModSolver<'a> {
    client: labrinth::Client,
    mod_config: &'a config::Config,
    mod_db: types::ModDB,
}

impl<'a> ModSolver<'a> {
    /// Construct a new mod solver for a config
    pub fn new(mod_config: &'a config::Config) -> Self {
        ModSolver {
            client: labrinth::Client::new(),
            mod_config,
            mod_db: types::ModDB::default(),
        }
    }

    /// Solve all the dependencies of the config, consuming self
    pub fn solve(mut self) -> Result<types::ModDB> {
        self.collect_required_projects()?;
        self.collect_optional_projects();
        Ok(self.mod_db)
    }

    /// Collect all the required versions from the config
    fn collect_required_projects(&mut self) -> Result<Vec<VersionId>> {
        let mut versions = Vec::<VersionId>::new();
        for project in self.mod_config.projects() {
            let mut collected = self.collect_project_and_dependencies(&project)?;
            versions.append(&mut collected);
        }
        Ok(versions)
    }

    /// Collect all the optional versions from the config
    fn collect_optional_projects(&mut self) -> Vec<VersionId> {
        let mut versions = Vec::<VersionId>::new();
        for project in self.mod_config.optional_projects() {
            let mut collected = match self.collect_project_and_dependencies(&project) {
                Ok(x) => x,
                Err(_) => continue,
            };
            versions.append(&mut collected);
        }
        versions
    }

    /// Collect a config project and its dependencies
    fn collect_project_and_dependencies(
        &mut self,
        project: &config::ConfigProject,
    ) -> Result<Vec<VersionId>> {
        let base_id = self.collect_config_project(project)?;
        let mut deps = self.collect_dependencies(&base_id).inspect_err(|_| {
            self.mod_db
                .remove(&types::ModLink::VersionId(base_id.clone()))
        })?;
        deps.push(base_id);
        Ok(deps)
    }

    /// Collect one project by its id
    fn collect_project_by_id(&mut self, project_id: &ProjectId) -> Result<ProjectId> {
        if let Some(project) = &mut self.mod_db.get_project_by_id(project_id) {
            return Ok(project.project_id.clone());
        }
        let project = self.client.get_project(project_id.as_str())?;
        let project_id = project.project_id.clone();
        self.mod_db.add_project(project);
        Ok(project_id)
    }

    /// Collect one project by its slug
    fn collect_project_by_slug(&mut self, project_slug: &ProjectSlug) -> Result<ProjectId> {
        if let Some(project) = &mut self.mod_db.get_project_by_slug(project_slug) {
            return Ok(project.project_id.clone());
        }
        let project = self.client.get_project(project_slug.as_str())?;
        let project_id = project.project_id.clone();
        self.mod_db.add_project(project);
        Ok(project_id)
    }

    /// Collect one version by its id
    fn collect_version(&mut self, version_id: &VersionId) -> Result<VersionId> {
        if let Some(version) = &mut self.mod_db.get_version(version_id) {
            return Ok(version.version_id.clone());
        }
        let version = self.client.get_version(version_id.as_str())?;
        let version_id = version.version_id.clone();
        self.mod_db.add_version(version);
        Ok(version_id)
    }

    /// Collect one project and a version by a project id
    fn collect_config_project(&mut self, project: &config::ConfigProject) -> Result<VersionId> {
        let project_id = match self.mod_db.get_project_by_slug(&project.name) {
            Some(x) => x.project_id.clone(),
            None => self.collect_project_by_slug(&project.name)?,
        };
        let version_id = match self
            .mod_db
            .get_preferred_by_id(&project_id)
            .map(|x| x.version_id.clone())
        {
            Some(x) => x,
            None => {
                let version = self.client.get_project_version_latest(
                    project.name.as_str(),
                    project.game_version,
                    project.loader,
                )?;
                let version_id = version.version_id.clone();
                self.mod_db.add_version(version);
                self.mod_db
                    .set_preferred_version(project_id, version_id.clone());
                version_id
            }
        };
        Ok(version_id)
    }

    /// Collect the appropriate version of a project
    fn collect_project_version(&mut self, project_id: &ProjectId) -> Result<VersionId> {
        let pid = self.collect_project_by_id(project_id)?;
        let mod_project =
            self.mod_db
                .get_project_by_id(&pid)
                .ok_or_else(|| Error::LocalCacheMiss {
                    key: project_id.to_string(),
                    msg: "Project was not added".into(),
                })?;
        if mod_project
            .loaders
            .contains(&self.mod_config.defaults.loader)
        {
            self.collect_config_project(&config::ConfigProject {
                name: mod_project.slug.clone(),
                game_version: self.mod_config.defaults.game_version,
                loader: self.mod_config.defaults.loader,
            })
        } else if mod_project.loaders.contains(&ModLoader::Minecraft) {
            self.collect_config_project(&config::ConfigProject {
                name: mod_project.slug.clone(),
                game_version: self.mod_config.defaults.game_version,
                loader: ModLoader::Minecraft,
            })
        } else if mod_project.loaders.contains(&ModLoader::Datapack) {
            self.collect_config_project(&config::ConfigProject {
                name: mod_project.slug.clone(),
                game_version: self.mod_config.defaults.game_version,
                loader: ModLoader::Datapack,
            })
        } else {
            todo!(
                "No idea how to resolve this one {}, {:?}",
                mod_project.slug,
                mod_project.loaders
            )
        }
    }

    /// Collect all the dependencies of a version. If one is missing, they are not collected.
    fn collect_dependencies(&mut self, version_id: &VersionId) -> Result<Vec<VersionId>> {
        let Some(version) = self.mod_db.get_version(version_id) else {
            return Err(Error::LocalCacheMiss {
                key: version_id.as_str().into(),
                msg: "Version not cached".into(),
            });
        };
        let deps = version.dependencies.clone();
        let mut found_deps = Vec::<VersionId>::new();
        for dep in &deps {
            if self.mod_db.contains_key(dep) {
                continue;
            }
            let collected = match dep {
                ModLink::ProjectId(x) => self.collect_project_version(x),
                ModLink::VersionId(x) => self.collect_version(x),
                ModLink::ProjectSlug(_) => {
                    unimplemented!("A dependency will never be a project slug");
                }
            };
            if collected.is_err() {
                for each in &found_deps {
                    self.mod_db.remove(&each.clone().into());
                }
            }
            let collected = collected?;
            let deps_res = self.collect_dependencies(&collected);
            let mut collected = match deps_res {
                Ok(mut x) => {
                    x.push(collected);
                    x
                }
                Err(e) => {
                    self.mod_db.remove(&collected.into());
                    for each in &found_deps {
                        self.mod_db.remove(&each.clone().into());
                    }
                    return Err(e);
                }
            };
            found_deps.append(&mut collected);
        }
        Ok(found_deps)
    }
}
