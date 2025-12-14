use crate::config;
use crate::error::{Error, Result};
use crate::mcmod_client;
use crate::moddb::ModDB;
use crate::types::{self, ModLink, ModLoader, ProjectId, VersionId};

const CLIENT_CACHE_JSON: &str = "db_cache.json";

/// Collects all mods and their dependencies according to the config
pub struct ModSolver<'a> {
    db: ModDB,
    client: mcmod_client::ModClient,
    config: &'a config::Config,
}

impl<'a> ModSolver<'a> {
    /// Construct a new mod solver for a config
    pub fn new(mod_config: &'a config::Config) -> Self {
        let client_cache = mod_config.paths.data.join(CLIENT_CACHE_JSON);
        let mut mod_client = mcmod_client::ModClient::new();
        if client_cache.is_file() {
            let _ = mod_client.read_cache(&client_cache);
        }
        ModSolver {
            db: ModDB::default(),
            client: mod_client,
            config: mod_config,
        }
    }

    /// Solve all the dependencies of the config, consuming self
    pub fn solve(mut self) -> Result<ModDB> {
        self.collect_required_projects()?;
        self.collect_optional_projects();
        let path = &self.config.paths.data;
        if !path.is_dir() {
            std::fs::create_dir(path)?;
        }
        self.client
            .write_cache(&path.join(CLIENT_CACHE_JSON))
            .expect("Failed to write cache");
        Ok(self.db)
    }

    /// Collect all the required versions from the config
    fn collect_required_projects(&mut self) -> Result<Vec<VersionId>> {
        let mut versions = Vec::<VersionId>::new();
        for project in self.config.projects() {
            let mut collected = self.collect_project_and_dependencies(&project)?;
            versions.append(&mut collected);
        }
        Ok(versions)
    }

    /// Collect all the optional versions from the config
    fn collect_optional_projects(&mut self) -> Vec<VersionId> {
        let mut versions = Vec::<VersionId>::new();
        for project in self.config.optional_projects() {
            let Ok(mut collected) = self.collect_project_and_dependencies(&project) else {
                continue;
            };
            versions.append(&mut collected);
        }
        versions
    }

    /// Collect a config project and its dependencies
    pub fn collect_project_and_dependencies(
        &mut self,
        project: &config::ConfigProject,
    ) -> Result<Vec<VersionId>> {
        let base_id = self.collect_config_project(project)?;
        let mut deps = self
            .collect_dependencies(base_id)
            .inspect_err(|_| self.db.remove(&types::ModLink::VersionId(base_id)))?;
        deps.push(base_id);
        Ok(deps)
    }

    /// Collect one project by its id
    fn collect_project_by_id(&mut self, project_id: ProjectId) -> Result<ProjectId> {
        if let Some(project) = &mut self.db.get_project_by_id(project_id) {
            return Ok(project.project_id);
        }
        let project = self.client.fetch_project_by_id(project_id)?;
        let project_id = project.project_id;
        self.db.add_project(project.clone());
        Ok(project_id)
    }

    /// Collect one version by its id
    fn collect_version(&mut self, version_id: VersionId) -> Result<VersionId> {
        if let Some(version) = &mut self.db.get_version(version_id) {
            return Ok(version.version_id);
        }
        let version = self.client.fetch_version(version_id)?;
        let version_id = version.version_id;
        self.db.add_version(version.clone());
        Ok(version_id)
    }

    /// Collect one project and a version by a project id
    fn collect_config_project(&mut self, project: &config::ConfigProject) -> Result<VersionId> {
        let version_id = self.client.fetch_project_version_latest(
            &project.name,
            project.game_version,
            project.loader,
        )?;
        self.db.add_version(
            self.client
                .get_db()
                .get_version(version_id)
                .ok_or_else(|| Error::CacheMiss(version_id.into()))?
                .clone(),
        );
        Ok(version_id)
    }

    /// Collect the appropriate version of a project
    fn collect_project_version(&mut self, project_id: ProjectId) -> Result<VersionId> {
        let pid = self.collect_project_by_id(project_id)?;
        let mod_project = self
            .db
            .get_project_by_id(pid)
            .ok_or_else(|| Error::CacheMiss(project_id.into()))?;
        if mod_project
            .loaders
            .contains(&self.config.defaults.loader)
        {
            self.collect_config_project(&config::ConfigProject {
                name: mod_project.slug.clone(),
                game_version: self.config.defaults.game_version,
                loader: self.config.defaults.loader,
            })
        } else if mod_project.loaders.contains(&ModLoader::Minecraft) {
            self.collect_config_project(&config::ConfigProject {
                name: mod_project.slug.clone(),
                game_version: self.config.defaults.game_version,
                loader: ModLoader::Minecraft,
            })
        } else if mod_project.loaders.contains(&ModLoader::Datapack) {
            self.collect_config_project(&config::ConfigProject {
                name: mod_project.slug.clone(),
                game_version: self.config.defaults.game_version,
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
    fn collect_dependencies(&mut self, version_id: VersionId) -> Result<Vec<VersionId>> {
        let Some(version) = self.db.get_version(version_id) else {
            return Err(Error::CacheMiss(version_id.into()));
        };
        let deps = version.dependencies.clone();
        let mut found_deps = Vec::<VersionId>::new();
        for dep in &deps {
            if self.db.contains_key(dep) {
                continue;
            }
            let collected = match dep {
                ModLink::ProjectId(x) => self.collect_project_version(*x),
                ModLink::VersionId(x) => self.collect_version(*x),
                ModLink::ProjectSlug(_) => {
                    unimplemented!("A dependency will never be a project slug");
                }
            };
            if collected.is_err() {
                for each in &found_deps {
                    self.db.remove(&(*each).into());
                }
            }
            let collected = collected?;
            let deps_res = self.collect_dependencies(collected);
            let mut collected = match deps_res {
                Ok(mut x) => {
                    x.push(collected);
                    x
                }
                Err(e) => {
                    self.db.remove(&collected.into());
                    for each in &found_deps {
                        self.db.remove(&(*each).into());
                    }
                    return Err(e);
                }
            };
            found_deps.append(&mut collected);
        }
        Ok(found_deps)
    }
}

pub struct ModSolverOffline<'a> {
    db: ModDB,
    client: mcmod_client::ModClient,
    config: &'a config::Config,
}

impl<'a> ModSolverOffline<'a> {
    /// Construct a new mod solver for a config
    pub fn new(mod_config: &'a config::Config) -> Result<Self> {
        let client_cache = mod_config.paths.data.join(CLIENT_CACHE_JSON);
        let mut mod_client = mcmod_client::ModClient::new();
        if client_cache.is_file() {
            mod_client.read_cache(&client_cache)?;
        }
        Ok(ModSolverOffline {
            db: ModDB::default(),
            client: mod_client,
            config: mod_config,
        })
    }

    /// Solve all the dependencies of the config, consuming self
    pub fn solve(mut self) -> Result<ModDB> {
        self.collect_required_projects()?;
        self.collect_optional_projects();
        self.client
            .write_cache(&self.config.paths.data.join(CLIENT_CACHE_JSON))
            .expect("Failed to write cache");
        Ok(self.db)
    }

    /// Collect all the required versions from the config
    fn collect_required_projects(&mut self) -> Result<Vec<VersionId>> {
        let mut versions = Vec::<VersionId>::new();
        for project in self.config.projects() {
            let mut collected = self.collect_project_and_dependencies(&project)?;
            versions.append(&mut collected);
        }
        Ok(versions)
    }

    /// Collect all the optional versions from the config
    fn collect_optional_projects(&mut self) -> Vec<VersionId> {
        let mut versions = Vec::<VersionId>::new();
        for project in self.config.optional_projects() {
            let Ok(mut collected) = self.collect_project_and_dependencies(&project) else {
                continue;
            };
            versions.append(&mut collected);
        }
        versions
    }

    /// Collect a config project and its dependencies
    pub fn collect_project_and_dependencies(
        &mut self,
        project: &config::ConfigProject,
    ) -> Result<Vec<VersionId>> {
        let base_id = self.collect_config_project(project)?;
        let mut deps = self
            .collect_dependencies(base_id)
            .inspect_err(|_| self.db.remove(&types::ModLink::VersionId(base_id)))?;
        deps.push(base_id);
        Ok(deps)
    }

    /// Collect one project by its id
    fn collect_project_by_id(&mut self, project_id: ProjectId) -> Result<ProjectId> {
        if let Some(project) = &mut self.db.get_project_by_id(project_id) {
            return Ok(project.project_id);
        }
        let project = self
            .client
            .get_db()
            .get_project_by_id(project_id)
            .ok_or_else(|| Error::CacheMiss(project_id.into()))?;
        let project_id = project.project_id;
        self.db.add_project(project.clone());
        Ok(project_id)
    }

    /// Collect one version by its id
    fn collect_version(&mut self, version_id: VersionId) -> Result<VersionId> {
        if let Some(version) = &mut self.db.get_version(version_id) {
            return Ok(version.version_id);
        }
        let version = self
            .client
            .get_db()
            .get_version(version_id)
            .ok_or_else(|| Error::CacheMiss(version_id.into()))?;
        let version_id = version.version_id;
        self.db.add_version(version.clone());
        Ok(version_id)
    }

    /// Get a config project from offline cache
    fn collect_config_project(&mut self, project: &config::ConfigProject) -> Result<VersionId> {
        let version = self
            .client
            .get_db()
            .find_project_version_latest(&project.name, project.game_version, project.loader)?
            .clone();
        let version_id = version.version_id;
        self.db.add_version(version);
        Ok(version_id)
    }

    /// Collect the appropriate version of a project
    fn collect_project_version(&mut self, project_id: ProjectId) -> Result<VersionId> {
        let pid = self.collect_project_by_id(project_id)?;
        let mod_project = self
            .db
            .get_project_by_id(pid)
            .ok_or_else(|| Error::CacheMiss(project_id.into()))?;
        if mod_project
            .loaders
            .contains(&self.config.defaults.loader)
        {
            self.collect_config_project(&config::ConfigProject {
                name: mod_project.slug.clone(),
                game_version: self.config.defaults.game_version,
                loader: self.config.defaults.loader,
            })
        } else if mod_project.loaders.contains(&ModLoader::Minecraft) {
            self.collect_config_project(&config::ConfigProject {
                name: mod_project.slug.clone(),
                game_version: self.config.defaults.game_version,
                loader: ModLoader::Minecraft,
            })
        } else if mod_project.loaders.contains(&ModLoader::Datapack) {
            self.collect_config_project(&config::ConfigProject {
                name: mod_project.slug.clone(),
                game_version: self.config.defaults.game_version,
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

    /// Collect all the dependencies of a version offline. If one is missing, they are not collected.
    fn collect_dependencies(&mut self, version_id: VersionId) -> Result<Vec<VersionId>> {
        let Some(version) = self.db.get_version(version_id) else {
            return Err(Error::CacheMiss(version_id.into()));
        };
        let deps = version.dependencies.clone();
        let mut found_deps = Vec::<VersionId>::new();
        for dep in &deps {
            if self.db.contains_key(dep) {
                continue;
            }
            let collected = match dep {
                ModLink::ProjectId(x) => self.collect_project_version(*x),
                ModLink::VersionId(x) => self.collect_version(*x),
                ModLink::ProjectSlug(_) => {
                    unimplemented!("A dependency will never be a project slug");
                }
            };
            if collected.is_err() {
                for each in &found_deps {
                    self.db.remove(&(*each).into());
                }
            }
            let collected = collected?;
            let deps_res = self.collect_dependencies(collected);
            let mut collected = match deps_res {
                Ok(mut x) => {
                    x.push(collected);
                    x
                }
                Err(e) => {
                    self.db.remove(&collected.into());
                    for each in &found_deps {
                        self.db.remove(&(*each).into());
                    }
                    return Err(e);
                }
            };
            found_deps.append(&mut collected);
        }
        Ok(found_deps)
    }
}
