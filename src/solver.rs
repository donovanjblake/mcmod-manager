use crate::error::{Result, Error};
use crate::config;
use crate::types::{self, ProjectId, ProjectSlug, VersionId, ModLoader, ModLink};
use crate::labrinth;


/// Collects all mods and their dependencies according to the config
pub struct ModSolver<'a> {
    client: &'a labrinth::Client,
    mod_config: &'a config::Config,
    mod_db: &'a mut types::ModDB,
}

impl<'a> ModSolver<'a> {

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
        let version_id = match self.mod_db
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
                self.mod_db.set_preferred_version(project_id, version_id.clone());
                version_id
            }
        };
        Ok(version_id)
    }

    /// Collect the appropriate version of a project
    fn collect_project_version(&mut self, project_id: &ProjectId) -> Result<VersionId> {
        let pid = self.collect_project_by_id(project_id)?;
        let mod_project = self.mod_db.get_project_by_id(&pid).expect("weewoo");
        if mod_project.loaders.contains(&self.mod_config.defaults.loader) {
            self.collect_config_project(
                &config::ConfigProject {
                    name: mod_project.slug.clone(),
                    game_version: self.mod_config.defaults.game_version,
                    loader: self.mod_config.defaults.loader,
                },
            )
        } else if mod_project.loaders.contains(&ModLoader::Minecraft) {
            self.collect_config_project(
                &config::ConfigProject {
                    name: mod_project.slug.clone(),
                    game_version: self.mod_config.defaults.game_version,
                    loader: ModLoader::Minecraft,
                },
            )
        } else if mod_project.loaders.contains(&ModLoader::Datapack) {
            self.collect_config_project(
                &config::ConfigProject {
                    name: mod_project.slug.clone(),
                    game_version: self.mod_config.defaults.game_version,
                    loader: ModLoader::Datapack,
                },
            )
        } else {
            todo!(
                "No idea how to resolve this one {}, {:?}",
                mod_project.slug,
                mod_project.loaders
            )
        }
    }

    /// Collect all the dependencies of a version. If one is missing, they are not collected.
    fn collect_dependencies(
        &mut self, version_id: &VersionId,
    ) -> Result<Vec<types::ModLink>> {
        let Some(version) = self.mod_db.get_version(version_id) else {
            return Err(Error::LocalCacheMiss {
                key: version_id.as_str().into(),
                msg: "Version not cached".into(),
            });
        };
        let deps = version.dependencies.clone();
        let mut found_deps = Vec::<ModLink>::new();
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
                    self.mod_db.remove(each);
                }
            }
            let collected = collected?;
            let deps_res = self.collect_dependencies(&collected);
            let collected = ModLink::from(collected);
            let mut collected = match deps_res {
                Ok(mut x) => {
                    x.push(collected);
                    x
                }
                Err(e) => {
                    self.mod_db.remove(&collected);
                    for each in &found_deps {
                        self.mod_db.remove(each);
                    }
                    return Err(e);
                }
            };
            found_deps.append(&mut collected);
        }
        Ok(found_deps)
    }
}