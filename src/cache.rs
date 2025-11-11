use std::path::PathBuf;

use crate::error::Result;
use crate::mcmod_client::Client;
use crate::types::*;

pub struct ModFileManager {
    data_dir: PathBuf,
    dot_minecraft_dir: PathBuf,
    client: Client,
}

impl ModFileManager {
    /// Construct a new mod file manager
    pub fn new(data_dir: PathBuf, dot_minecraft_dir: PathBuf) -> Self {
        if !data_dir.is_dir() {
            std::fs::create_dir(&data_dir)
                .unwrap_or_else(|e| panic!("{e:?}: Could not create {data_dir:?}"));
        }
        if !dot_minecraft_dir.is_dir() {
            panic!("{dot_minecraft_dir:?} does not exist");
        }
        ModFileManager {
            data_dir,
            dot_minecraft_dir,
            client: Default::default(),
        }
    }

    /// Construct the path to a cached download file
    fn cache_path(&self, version_id: &VersionId, filename: &String) -> PathBuf {
        self.data_dir
            .join(&version_id.as_str()[0..2])
            .join(&version_id.as_str()[2..])
            .join(filename)
    }

    /// Return the location of a cached download file
    pub fn find_file(&self, version_id: &VersionId, filename: &String) -> Option<PathBuf> {
        let path = self
            .data_dir
            .join(&version_id.as_str()[0..2])
            .join(&version_id.as_str()[2..])
            .join(filename);
        if !path.is_file() { None } else { Some(path) }
    }

    /// Download a file to the data cache directory
    pub fn download_file(&self, version_id: &VersionId, mod_file: &ModFile) -> Result<PathBuf> {
        let buffer = self.client.download_file(&mod_file.url)?;
        let path = self.cache_path(version_id, &mod_file.name);
        std::fs::create_dir_all(
            path.parent()
                .unwrap_or_else(|| panic!("{path:?} does not have parent")),
        )?;
        std::fs::write(&path, buffer)?;
        Ok(path)
    }

    /// Get a file from the data cache, downloading it if necessary
    pub fn get_file(&self, version_id: &VersionId, mod_file: &ModFile) -> Result<PathBuf> {
        if let Some(path) = self.find_file(version_id, &mod_file.name) {
            return Ok(path);
        }
        self.download_file(version_id, mod_file)
    }

    fn install_path(&self, filename: &String, loader: Option<ModLoader>) -> PathBuf {
        self.dot_minecraft_dir
            .join(match loader {
                Some(ModLoader::Minecraft) => "resourcepacks",
                Some(ModLoader::Datapack) => "datapacks",
                Some(ModLoader::Iris) | Some(ModLoader::Optifine) => "shaderpacks",
                _ => "mods",
            })
            .join(filename)
    }

    pub fn install_file(
        &self,
        version_id: &VersionId,
        mod_file: &ModFile,
        loader: Option<ModLoader>,
    ) -> Result<()> {
        let src = self.get_file(version_id, mod_file)?;
        let dst = self.install_path(&mod_file.name, loader);
        std::fs::create_dir_all(
            dst.parent()
                .unwrap_or_else(|| panic!("{dst:?} does not have parent")),
        )?;
        std::fs::copy(src, dst)?;
        Ok(())
    }
}
