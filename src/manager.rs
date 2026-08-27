use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::mcmod_client;
use crate::types::{ModFile, ModLoader, VersionId};

pub struct ModFileManager {
    data_dir: PathBuf,
    dot_minecraft_dir: PathBuf,
}

impl ModFileManager {
    /// Construct a new mod file manager
    pub fn new(data_dir: PathBuf, dot_minecraft_dir: PathBuf) -> Self {
        if !data_dir.is_dir() {
            std::fs::create_dir(&data_dir)
                .unwrap_or_else(|e| panic!("{}: Could not create {}", e, data_dir.display()));
        }
        assert!(
            dot_minecraft_dir.is_dir(),
            "{} does not exist",
            dot_minecraft_dir.display()
        );
        ModFileManager {
            data_dir,
            dot_minecraft_dir,
        }
    }

    /// Remove all installed mods
    pub fn clear(&self) -> Result<()> {
        for x in ["resourcepacks", "shaderpacks", "mods", "datapacks"] {
            let root = self.dot_minecraft_dir.join(x);
            if !root.is_dir() {
                continue;
            }
            for x in root.read_dir().map_err(|_| Error::ReadPath(root.clone()))? {
                let x = x.map_err(|_| Error::ReadPath(root.clone()))?.path();
                if !x.is_file() {
                    continue;
                }
                std::fs::remove_file(&x).map_err(|_| Error::ReadPath(x))?;
            }
        }
        Ok(())
    }

    /// Construct the path to a cached download file
    fn cache_path(&self, version_id: VersionId, filename: &String) -> PathBuf {
        let version_id = version_id.to_string();
        self.data_dir
            .join(&version_id[0..2])
            .join(&version_id)
            .join(filename)
    }

    /// Return the location of a cached download file
    pub fn find_file(&self, version_id: VersionId, filename: &String) -> Option<PathBuf> {
        let version_id = version_id.to_string();
        let path = self
            .data_dir
            .join(&version_id[0..2])
            .join(&version_id)
            .join(filename);
        if path.is_file() { Some(path) } else { None }
    }

    /// Download a file to the data cache directory
    pub fn download_file(&self, version_id: VersionId, mod_file: &ModFile) -> Result<PathBuf> {
        let buffer = mcmod_client::download_file(&mod_file.url)?;
        let path = self.cache_path(version_id, &mod_file.name);
        std::fs::create_dir_all(
            path.parent()
                .unwrap_or_else(|| panic!("{:?} does not have parent", path.display())),
        )
        .map_err(|_| Error::CreatePath(path.clone()))?;
        std::fs::write(&path, buffer).map_err(|_| Error::WritePath(path.clone()))?;
        Ok(path)
    }

    /// Get a file from the data cache, downloading it if necessary
    pub fn get_file(&self, version_id: VersionId, mod_file: &ModFile) -> Result<PathBuf> {
        if let Some(path) = self.find_file(version_id, &mod_file.name) {
            return Ok(path);
        }
        self.download_file(version_id, mod_file)
    }

    /// Get the install path for a given loader
    fn install_path(&self, filename: &String, loader: Option<ModLoader>) -> PathBuf {
        self.dot_minecraft_dir
            .join(match loader {
                Some(ModLoader::Minecraft) => "resourcepacks",
                Some(ModLoader::Datapack) => "datapacks",
                Some(ModLoader::Iris | ModLoader::Optifine) => "shaderpacks",
                _ => "mods",
            })
            .join(filename)
    }

    /// Get and install the file to the proper path
    pub fn install_file(
        &self,
        version_id: VersionId,
        mod_file: &ModFile,
        loader: Option<ModLoader>,
    ) -> Result<()> {
        let src = self.get_file(version_id, mod_file)?;
        let dst = self.install_path(&mod_file.name, loader);
        std::fs::create_dir_all(
            dst.parent()
                .unwrap_or_else(|| panic!("{:?} does not have parent", dst.display())),
        )
        .map_err(|_| Error::CreatePath(dst.clone()))?;
        std::fs::copy(&src, &dst).map_err(|_| Error::ReadPath(src.clone()))?;
        Ok(())
    }
}
